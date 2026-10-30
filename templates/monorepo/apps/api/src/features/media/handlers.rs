// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `media` handlers: each parses the request, calls one
//! [`MediaService`] method, and answers.

use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::extract::rejection::BytesRejection;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::dto::{
    DownloadUrl, ListMediaQuery, MediaResponse, StartUploadDto, UploadMediaQuery, UploadTicket,
};
use super::service::MediaService;
use crate::error::AppError;
use crate::shared::auth::AuthUser;
use crate::shared::body;
use crate::shared::response::{ErrorBody, PaginatedResponse};
use crate::shared::storage;
use crate::shared::validation::{Path, Query, ValidatedJson};

/// The header an upload may name its file in, when the query string
/// doesn't (`?filename=`).
const FILENAME_HEADER: &str = "x-filename";

/// The `Content-Security-Policy` of served files: nothing loads, and a
/// document opened anyway runs sandboxed.
const SANDBOX: &str = "default-src 'none'; sandbox";

/// `GET /api/v1/media`: one page of the caller's files, newest first, `200`.
#[utoipa::path(
    get,
    path = "/api/v1/media",
    tag = "media",
    operation_id = "list",
    params(ListMediaQuery),
    responses(
        (status = 200, description = "The caller's files, newest first", body = PaginatedResponse<MediaResponse>),
        (status = 400, description = "A malformed query", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Query(query): Query<ListMediaQuery>,
) -> Result<Response, AppError> {
    let page = service.list(caller.id.into(), query.pagination()).await?;
    Ok(Json(page).into_response())
}

/// `POST /api/v1/media`: stores the raw body as one of the caller's files,
/// `201`. The file name is `?filename=`, or else the `X-Filename` header.
#[utoipa::path(
    post,
    path = "/api/v1/media",
    tag = "media",
    operation_id = "upload",
    params(UploadMediaQuery),
    request_body(
        content_type = "application/octet-stream",
        description = "The raw file bytes. The type is detected from the bytes; the request's Content-Type is ignored"
    ),
    responses(
        (status = 201, description = "Stored", body = MediaResponse),
        (status = 400, description = "An empty body", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 413, description = "Larger than MEDIA_MAX_UPLOAD_BYTES", body = ErrorBody),
        (status = 415, description = "Unrecognized file type, or one that is not allowed", body = ErrorBody),
        (status = 422, description = "A file name over 255 characters", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn upload(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Query(query): Query<UploadMediaQuery>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, AppError> {
    let bytes = body::limited(body, service.max_upload_bytes, "file")?;
    let filename = query.filename.or_else(|| {
        headers
            .get(FILENAME_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    });
    let media = service.upload(caller.id.into(), bytes, filename).await?;
    Ok((StatusCode::CREATED, Json(media)).into_response())
}

/// `GET /api/v1/media/{id}`: one of the caller's files, `200`.
#[utoipa::path(
    get,
    path = "/api/v1/media/{id}",
    tag = "media",
    operation_id = "metadata",
    params(("id" = uuid::Uuid, Path, description = "Media id")),
    responses(
        (status = 200, description = "File metadata", body = MediaResponse),
        (status = 400, description = "A malformed id", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such file, or it belongs to someone else", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn metadata(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    let media = service.metadata(caller.id.into(), id.into()).await?;
    Ok(Json(media).into_response())
}

/// `DELETE /api/v1/media/{id}`: deletes one of the caller's files, `204`.
#[utoipa::path(
    delete,
    path = "/api/v1/media/{id}",
    tag = "media",
    operation_id = "remove",
    params(("id" = uuid::Uuid, Path, description = "Media id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 400, description = "A malformed id", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such file, or it belongs to someone else", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn remove(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    service.delete(caller.id.into(), id.into()).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `GET /api/v1/media/{id}/content`: the bytes of one of the caller's
/// files, `200`, with the detected `Content-Type`. Only an image is shown
/// inline; anything else downloads, and `nosniff` and a sandboxing CSP
/// keep the browser from treating it as anything else.
#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/content",
    tag = "media",
    operation_id = "content",
    params(("id" = uuid::Uuid, Path, description = "Media id")),
    responses(
        (status = 200, description = "The file's bytes, with the detected Content-Type"),
        (status = 400, description = "A malformed id", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such file, or it belongs to someone else", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn content(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Response, AppError> {
    let content = service.content(caller.id.into(), id.into()).await?;
    let headers = [
        (header::CONTENT_TYPE, content.content_type),
        (header::CONTENT_DISPOSITION, content.disposition),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        (header::CONTENT_SECURITY_POLICY, SANDBOX.to_owned()),
    ];
    Ok((headers, content.bytes).into_response())
}

/// `POST /api/v1/media/uploads`: starts a direct upload, `201`. The client
/// sends the bytes with `method` to `upload_url`, with `headers`, then
/// completes it; until then the file is no one's to read.
#[utoipa::path(
    post,
    path = "/api/v1/media/uploads",
    tag = "media",
    operation_id = "start_upload",
    request_body = StartUploadDto,
    responses(
        (status = 201, description = "Where to put the bytes, and until when", body = UploadTicket),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 413, description = "Larger than MEDIA_MAX_UPLOAD_BYTES", body = ErrorBody),
        (status = 415, description = "A type that is not allowed", body = ErrorBody),
        (status = 422, description = "An invalid body, or a file name over 255 characters", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn start_upload(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    body: ValidatedJson<StartUploadDto>,
) -> Result<Response, AppError> {
    let ticket = service
        .start_upload(caller.id.into(), body.into_inner())
        .await?;
    Ok((StatusCode::CREATED, Json(ticket)).into_response())
}

/// `POST /api/v1/media/{id}/complete`: completes a direct upload, `200`,
/// once its bytes are up and match what was declared.
#[utoipa::path(
    post,
    path = "/api/v1/media/{id}/complete",
    tag = "media",
    operation_id = "complete_upload",
    params(("id" = uuid::Uuid, Path, description = "Media id")),
    responses(
        (status = 200, description = "The file, now readable", body = MediaResponse),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such upload, or it expired or belongs to someone else", body = ErrorBody),
        (status = 409, description = "The bytes have not been uploaded yet", body = ErrorBody),
        (status = 422, description = "The bytes are not the size or type declared; the upload is removed", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn complete_upload(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<MediaResponse>, AppError> {
    let media = service.complete_upload(caller.id.into(), id.into()).await?;
    Ok(Json(media))
}

/// `GET /api/v1/media/{id}/url`: a short-lived URL one of the caller's
/// files can be fetched from without credentials, `200`.
#[utoipa::path(
    get,
    path = "/api/v1/media/{id}/url",
    tag = "media",
    operation_id = "download_url",
    params(("id" = uuid::Uuid, Path, description = "Media id")),
    responses(
        (status = 200, description = "Where to fetch the file, and until when", body = DownloadUrl),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such file, or it belongs to someone else", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn download_url(
    State(service): State<Arc<MediaService>>,
    caller: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<DownloadUrl>, AppError> {
    let url = service.download_url(caller.id.into(), id.into()).await?;
    Ok(Json(url))
}

/// `PUT /api/v1/storage/{token}`: stores the raw body under the key an
/// API-signed upload URL grants, `204`. The token is the credential.
#[utoipa::path(
    put,
    path = "/api/v1/storage/{token}",
    tag = "media",
    operation_id = "put_object",
    params(("token" = String, Path, description = "The signed token")),
    request_body(content_type = "application/octet-stream", description = "The file's bytes"),
    responses(
        (status = 204, description = "Stored"),
        (status = 403, description = "An invalid or expired token", body = ErrorBody),
        (status = 413, description = "Larger than MEDIA_MAX_UPLOAD_BYTES", body = ErrorBody),
    ),
)]
pub async fn put_object(
    State(service): State<Arc<MediaService>>,
    Path(token): Path<String>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<StatusCode, AppError> {
    let bytes = body::limited(body, service.max_upload_bytes, "file")?;
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream");
    storage::put_signed(service.storage.as_ref(), &token, content_type, bytes).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/v1/storage/{token}`: the bytes under the key an API-signed
/// download URL grants, `200`, served as `content` serves them.
#[utoipa::path(
    get,
    path = "/api/v1/storage/{token}",
    tag = "media",
    operation_id = "get_object",
    params(("token" = String, Path, description = "The signed token")),
    responses(
        (status = 200, description = "The file's bytes, with their Content-Type"),
        (status = 403, description = "An invalid or expired token", body = ErrorBody),
        (status = 404, description = "The file is gone", body = ErrorBody),
    ),
)]
pub async fn get_object(
    State(service): State<Arc<MediaService>>,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    let object = storage::get_signed(service.storage.as_ref(), &token).await?;
    let headers = [
        (header::CONTENT_TYPE, object.content_type),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        (header::CONTENT_SECURITY_POLICY, SANDBOX.to_owned()),
    ];
    Ok((headers, object.bytes).into_response())
}
