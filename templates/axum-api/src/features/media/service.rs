// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! `MediaService`: each caller's uploaded files. The bytes go through the
//! `Storage` port (`shared/storage`), the rows through this feature's
//! query module, `db::queries::media`. Every file is its owner's alone:
//! another caller's is a `404`, as if it didn't exist.
//!
//! A file arrives one of two ways. `upload` takes its bytes in the request.
//! `start_upload` hands out a presigned URL the client puts the bytes to
//! straight, and `complete_upload` then checks them, as `upload` does,
//! before the file appears (engine ADR 0024). An upload never completed is
//! removed by `sweep`.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::dto::{
    DownloadUrl, FILENAME_MAX, MediaContent, MediaResponse, StartUploadDto, UploadTicket,
};
use crate::db::media::models::{Media, MediaId};
use crate::db::queries::media as queries;
use crate::db::users::models::UserId;
use crate::error::AppError;
use crate::shared::jobs::{Job, JobError};
use crate::shared::pagination::Pagination;
use crate::shared::response::PaginatedResponse;
use crate::shared::storage::{Access, Storage};

/// The prefix of every key this feature stores under.
const KEY_PREFIX: &str = "media/";

/// How many of an object's first bytes are read to detect its type.
const SNIFF_BYTES: usize = 8 * 1024;

/// The `media` service, built by `app/`.
#[derive(Clone)]
pub struct MediaService {
    /// The pool the rows are read and written through.
    pub pool: PgPool,
    /// Where the bytes are.
    pub storage: Arc<dyn Storage>,
    /// `MEDIA_MAX_UPLOAD_BYTES`: the largest upload. The routes enforce it
    /// on the body; [`MediaService::upload`] checks it again.
    pub max_upload_bytes: usize,
    /// `MEDIA_ALLOWED_TYPES`: the types, detected from the bytes, an upload
    /// may have.
    pub allowed_types: Vec<String>,
    /// `MEDIA_UPLOAD_URL_TTL_SECS`: how long a direct-upload URL, and its
    /// upload intent, last.
    pub upload_url_ttl: Duration,
    /// `MEDIA_DOWNLOAD_URL_TTL_SECS`: how long a download URL lasts.
    pub download_url_ttl: Duration,
}

impl MediaService {
    /// Stores `bytes` for `owner`, named `filename`: the type is detected
    /// from the bytes (whatever the request claimed) and must be allowed
    /// (`415`); an empty body is a `400`, one over the limit a `413`, a
    /// name over 255 characters a `422`. The bytes are stored under a new
    /// key, then the row is written; when the row fails, the bytes are
    /// deleted again.
    pub async fn upload(
        &self,
        owner: UserId,
        bytes: Bytes,
        filename: Option<String>,
    ) -> Result<MediaResponse, AppError> {
        if bytes.is_empty() {
            return Err(AppError::BadRequest("the upload is empty".to_owned()));
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(self.too_large());
        }
        let filename = checked_filename(filename)?;
        let content_type = self.detected_type(&bytes)?;

        let id = MediaId::generate();
        let storage_key = format!("{KEY_PREFIX}{}", id.0.simple());
        let size_bytes = i64::try_from(bytes.len()).map_err(|_| self.too_large())?;
        self.storage.put(&storage_key, bytes, content_type).await?;
        let inserted = queries::insert_media(
            &self.pool,
            queries::InsertMediaParams {
                id,
                owner_id: owner,
                content_type: content_type.to_owned(),
                size_bytes,
                original_filename: filename,
                storage_key: storage_key.clone(),
            },
        )
        .await;
        match inserted {
            Ok(media) => Ok(media.into()),
            Err(error) => {
                self.forget(&storage_key).await;
                Err(error.into())
            }
        }
    }

    /// One page of `owner`'s files, newest first.
    pub async fn list(
        &self,
        owner: UserId,
        pagination: Pagination,
    ) -> Result<PaginatedResponse<MediaResponse>, AppError> {
        let total = queries::count_owned_media(&self.pool, owner).await?;
        let items =
            queries::list_owned_media(&self.pool, owner, pagination.limit(), pagination.offset())
                .await?;
        let items = items.into_iter().map(MediaResponse::from).collect();
        Ok(PaginatedResponse::new(items, pagination, total))
    }

    /// The file `id`'s metadata, when `owner` owns it (else `404`).
    pub async fn metadata(&self, owner: UserId, id: MediaId) -> Result<MediaResponse, AppError> {
        let media = queries::owned_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        Ok(media.into())
    }

    /// The file `id`'s bytes, when `owner` owns it (else `404`, also when
    /// its bytes are gone).
    pub async fn content(&self, owner: UserId, id: MediaId) -> Result<MediaContent, AppError> {
        let media = queries::owned_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        let Some(stored) = self.storage.get(&media.storage_key).await? else {
            tracing::warn!(media_id = %media.id, "a media row has no stored bytes");
            return Err(not_found());
        };
        Ok(MediaContent::new(media, stored.bytes))
    }

    /// Deletes the file `id`, when `owner` owns it (else `404`): the row,
    /// then its bytes (a failure there is logged, not answered).
    pub async fn delete(&self, owner: UserId, id: MediaId) -> Result<(), AppError> {
        let storage_key = queries::delete_owned_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        self.forget(&storage_key).await;
        Ok(())
    }

    /// Starts a direct upload for `owner`: checks what the file will be
    /// (its type allowed, `415`; its size within the limit, `413`; its name,
    /// `422`), writes its upload intent, and answers the presigned URL the
    /// client puts the bytes to. Nothing reads the file until
    /// [`complete_upload`](Self::complete_upload).
    pub async fn start_upload(
        &self,
        owner: UserId,
        upload: StartUploadDto,
    ) -> Result<UploadTicket, AppError> {
        if usize::try_from(upload.size_bytes).map_or(true, |size| size > self.max_upload_bytes) {
            return Err(self.too_large());
        }
        let filename = checked_filename(upload.filename)?;
        self.allowed(&upload.content_type)?;

        let id = MediaId::generate();
        let storage_key = format!("{KEY_PREFIX}{}", id.0.simple());
        let url = self
            .storage
            .presign(&storage_key, Access::Upload, self.upload_url_ttl)
            .await?;
        queries::insert_upload_intent(
            &self.pool,
            queries::InsertUploadIntentParams {
                id,
                owner_id: owner,
                content_type: upload.content_type.clone(),
                size_bytes: upload.size_bytes,
                original_filename: filename,
                storage_key,
                expires_at: url.expires_at,
            },
        )
        .await?;
        Ok(UploadTicket {
            media_id: id,
            method: url.method.to_owned(),
            upload_url: url.url,
            headers: BTreeMap::from([("Content-Type".to_owned(), upload.content_type)]),
            expires_at: url.expires_at,
        })
    }

    /// Completes `owner`'s direct upload `id` (else `404`, also once it
    /// expired): its bytes must be there (`409` while they aren't), as large
    /// as declared and of the declared type, detected from the bytes as
    /// `upload` detects it. A mismatch removes the upload and its bytes and
    /// answers `422`.
    pub async fn complete_upload(
        &self,
        owner: UserId,
        id: MediaId,
    ) -> Result<MediaResponse, AppError> {
        let pending = queries::pending_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        let Some(peek) = self.storage.peek(&pending.storage_key, SNIFF_BYTES).await? else {
            return Err(AppError::Conflict(
                "the file's bytes have not been uploaded yet".to_owned(),
            ));
        };
        let declared = u64::try_from(pending.size_bytes).unwrap_or(0);
        let detected = infer::get(&peek.start).map(|kind| kind.mime_type());
        let problem = if peek.size != declared {
            Some(format!(
                "the upload is {} bytes, not the {declared} declared",
                peek.size
            ))
        } else if detected != Some(pending.content_type.as_str()) {
            Some(format!(
                "the upload is not of the declared type {}",
                pending.content_type
            ))
        } else {
            None
        };
        if let Some(message) = problem {
            self.discard(owner, &pending).await?;
            return Err(AppError::Validation {
                message,
                details: None,
            });
        }
        let media = queries::complete_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        Ok(media.into())
    }

    /// A short-lived URL `owner`'s file `id` can be fetched from without
    /// credentials (else `404`).
    pub async fn download_url(&self, owner: UserId, id: MediaId) -> Result<DownloadUrl, AppError> {
        let media = queries::owned_media(&self.pool, id, owner)
            .await?
            .ok_or_else(not_found)?;
        let url = self
            .storage
            .presign(&media.storage_key, Access::Download, self.download_url_ttl)
            .await?;
        Ok(DownloadUrl {
            url: url.url,
            expires_at: url.expires_at,
        })
    }

    /// Removes every upload intent past its expiry, with any bytes it got:
    /// how many.
    pub async fn sweep(&self) -> Result<usize, AppError> {
        let keys = queries::delete_expired_uploads(&self.pool).await?;
        for key in &keys {
            self.forget(key).await;
        }
        Ok(keys.len())
    }

    /// The type detected from `bytes`, when it is allowed (else `415`).
    fn detected_type(&self, bytes: &[u8]) -> Result<&'static str, AppError> {
        let Some(detected) = infer::get(bytes) else {
            return Err(AppError::UnsupportedMediaType(
                "the file type could not be recognized".to_owned(),
            ));
        };
        let content_type = detected.mime_type();
        self.allowed(content_type)?;
        Ok(content_type)
    }

    /// `415` unless `content_type` is one of [`MediaService::allowed_types`].
    fn allowed(&self, content_type: &str) -> Result<(), AppError> {
        if self
            .allowed_types
            .iter()
            .any(|allowed| allowed == content_type)
        {
            Ok(())
        } else {
            Err(AppError::UnsupportedMediaType(format!(
                "files of type {content_type} are not accepted"
            )))
        }
    }

    /// Removes the upload intent `pending` and its bytes.
    async fn discard(&self, owner: UserId, pending: &Media) -> Result<(), AppError> {
        queries::delete_pending_media(&self.pool, pending.id, owner).await?;
        self.forget(&pending.storage_key).await;
        Ok(())
    }

    /// Deletes the bytes under `key`, best effort: an orphaned object is
    /// harmless, so a failure is only logged.
    async fn forget(&self, key: &str) {
        if let Err(error) = self.storage.delete(key).await {
            tracing::warn!(%error, key, "could not delete stored bytes");
        }
    }

    /// The `413` for a body over [`MediaService::max_upload_bytes`].
    fn too_large(&self) -> AppError {
        AppError::too_large("file", self.max_upload_bytes)
    }
}

/// `filename` when it is a name worth keeping: blank is none, and one over
/// [`FILENAME_MAX`] characters a `422`.
fn checked_filename(filename: Option<String>) -> Result<Option<String>, AppError> {
    let filename = filename.filter(|name| !name.trim().is_empty());
    if filename
        .as_deref()
        .is_some_and(|name| name.chars().count() > FILENAME_MAX)
    {
        return Err(AppError::Validation {
            message: format!("the file name must be at most {FILENAME_MAX} characters"),
            details: None,
        });
    }
    Ok(filename)
}

/// The `404` for a file the caller doesn't have.
fn not_found() -> AppError {
    AppError::NotFound("no such file".to_owned())
}

/// The job removing every upload intent past its expiry, with any bytes it
/// got (`media.sweep_uploads`): `app/run.rs`'s schedule enqueues it every
/// five minutes.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SweepUploads;

#[async_trait]
impl Job for SweepUploads {
    const KIND: &'static str = "media.sweep_uploads";
    type Context = MediaService;

    async fn run(self, media: &MediaService) -> Result<(), JobError> {
        let removed = media.sweep().await?;
        if removed > 0 {
            tracing::info!(removed, "removed expired uploads");
        }
        Ok(())
    }
}
