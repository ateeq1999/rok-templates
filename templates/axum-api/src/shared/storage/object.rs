// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`ObjectStorage`]: the [`Storage`] port over the `object_store` crate,
//! on local files, S3 or memory.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::signer::Signer;
use object_store::{
    Attribute, AttributeValue, Attributes, ObjectStore, ObjectStoreExt, PutOptions, PutPayload,
};

use super::{
    Access, ApiSigner, Peek, PresignedUrl, Storage, StorageUrl, StoredObject, is_valid_key,
};
use crate::error::AppError;

/// The content type of an object whose store kept none, and whose bytes
/// say nothing either.
const UNKNOWN_TYPE: &str = "application/octet-stream";

/// How a store's presigned URLs are signed.
#[derive(Debug, Clone)]
pub struct Signing {
    /// The signer of the URLs the API serves, for a store that can't sign
    /// (local files, memory).
    pub api: ApiSigner,
    /// `STORAGE_PUBLIC_ENDPOINT`: the endpoint an S3-compatible store's URLs
    /// are signed for, when clients reach the store elsewhere than the API
    /// does (`http://minio:9000` inside Docker, `https://files.example.com`
    /// outside).
    pub public_endpoint: Option<String>,
}

/// Opens the store `url` names: a `file://` directory is created when
/// missing; an `s3://` bucket reads its settings from the `AWS_*`
/// variables. `app.rs` calls it once, at startup.
pub fn open(url: &StorageUrl, signing: Signing) -> Result<Arc<dyn Storage>, object_store::Error> {
    let storage = match url {
        StorageUrl::Memory => ObjectStorage::memory().with_api_signer(signing.api),
        StorageUrl::File(dir) => {
            std::fs::create_dir_all(dir).map_err(|source| object_store::Error::Generic {
                store: "LocalFileSystem",
                source: Box::new(source),
            })?;
            // Local files keep no attributes: the content type is
            // detected again when read.
            ObjectStorage {
                store: Arc::new(LocalFileSystem::new_with_prefix(dir)?),
                keeps_content_type: false,
                signer: UrlSigner::Api(signing.api),
            }
        }
        StorageUrl::S3(bucket) => {
            let s3 = Arc::new(
                AmazonS3Builder::from_env()
                    .with_bucket_name(bucket)
                    .build()?,
            );
            // Signed for the public endpoint, a URL is valid where the
            // client sends it: the signature covers the host.
            let signer: Arc<dyn Signer> = match &signing.public_endpoint {
                Some(endpoint) => Arc::new(
                    AmazonS3Builder::from_env()
                        .with_bucket_name(bucket)
                        .with_endpoint(endpoint)
                        .build()?,
                ),
                None => s3.clone(),
            };
            ObjectStorage {
                store: s3,
                keeps_content_type: true,
                signer: UrlSigner::Store(signer),
            }
        }
    };
    Ok(Arc::new(storage))
}

/// Who signs a store's presigned URLs.
#[derive(Debug, Clone)]
enum UrlSigner {
    /// The store itself: S3 and its kind.
    Store(Arc<dyn Signer>),
    /// The API, which serves the URLs at `STORAGE_ROUTE`.
    Api(ApiSigner),
}

/// The [`Storage`] port over an `object_store` store.
#[derive(Debug, Clone)]
pub struct ObjectStorage {
    store: Arc<dyn ObjectStore>,
    /// Whether the store keeps an object's content type (S3 and memory
    /// do, local files don't).
    keeps_content_type: bool,
    signer: UrlSigner,
}

impl ObjectStorage {
    /// A store in memory, empty, and lost when dropped: `memory://`, what
    /// the tests configure. Its URLs are signed under a random key until
    /// [`with_api_signer`](Self::with_api_signer) sets one.
    #[must_use]
    pub fn memory() -> Self {
        ObjectStorage {
            store: Arc::new(InMemory::new()),
            keeps_content_type: true,
            signer: UrlSigner::Api(ApiSigner::from_secret(&rand::random::<[u8; 32]>())),
        }
    }

    /// This store, with the API signing its URLs under `signer`.
    #[must_use]
    pub fn with_api_signer(mut self, signer: ApiSigner) -> Self {
        self.signer = UrlSigner::Api(signer);
        self
    }
}

/// `key` as an `object_store` path, when it is a valid key.
fn path(key: &str) -> Option<Path> {
    if is_valid_key(key) {
        Path::parse(key).ok()
    } else {
        None
    }
}

/// A failure of the store, answered as a `500`.
fn failed(error: object_store::Error) -> AppError {
    AppError::Internal(Box::new(error))
}

#[async_trait]
impl Storage for ObjectStorage {
    async fn put(&self, key: &str, bytes: Bytes, content_type: &str) -> Result<(), AppError> {
        let Some(location) = path(key) else {
            return Err(AppError::Internal(
                format!("`{key}` is not a valid storage key").into(),
            ));
        };
        let mut attributes = Attributes::new();
        if self.keeps_content_type {
            attributes.insert(
                Attribute::ContentType,
                AttributeValue::from(content_type.to_owned()),
            );
        }
        let options = PutOptions {
            attributes,
            ..PutOptions::default()
        };
        self.store
            .put_opts(&location, PutPayload::from(bytes), options)
            .await
            .map_err(failed)?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<StoredObject>, AppError> {
        let Some(location) = path(key) else {
            return Ok(None);
        };
        let result = match self.store.get(&location).await {
            Ok(result) => result,
            Err(object_store::Error::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(failed(error)),
        };
        let kept = result
            .attributes
            .get(&Attribute::ContentType)
            .map(|value| value.as_ref().to_owned());
        let bytes = result.bytes().await.map_err(failed)?;
        let content_type = kept.unwrap_or_else(|| {
            infer::get(&bytes)
                .map_or(UNKNOWN_TYPE, |kind| kind.mime_type())
                .to_owned()
        });
        Ok(Some(StoredObject {
            bytes,
            content_type,
        }))
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        let Some(location) = path(key) else {
            return Ok(());
        };
        match self.store.delete(&location).await {
            Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(error) => Err(failed(error)),
        }
    }

    async fn presign(
        &self,
        key: &str,
        access: Access,
        ttl: Duration,
    ) -> Result<PresignedUrl, AppError> {
        let Some(location) = path(key) else {
            return Err(AppError::Internal(
                format!("`{key}` is not a valid storage key").into(),
            ));
        };
        let now = chrono::Utc::now();
        match &self.signer {
            UrlSigner::Api(signer) => Ok(signer.sign(key, access, ttl, now)),
            UrlSigner::Store(signer) => {
                let method = match access {
                    Access::Upload => reqwest::Method::PUT,
                    Access::Download => reqwest::Method::GET,
                };
                let url = signer
                    .signed_url(method, &location, ttl)
                    .await
                    .map_err(failed)?;
                let ttl = chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::zero());
                Ok(PresignedUrl {
                    method: access.method(),
                    url: url.to_string(),
                    expires_at: now + ttl,
                })
            }
        }
    }

    async fn peek(&self, key: &str, len: usize) -> Result<Option<Peek>, AppError> {
        let Some(location) = path(key) else {
            return Ok(None);
        };
        let meta = match self.store.head(&location).await {
            Ok(meta) => meta,
            Err(object_store::Error::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(failed(error)),
        };
        let end = meta.size.min(u64::try_from(len).unwrap_or(u64::MAX));
        let start = if end == 0 {
            Bytes::new()
        } else {
            self.store
                .get_range(&location, 0..end)
                .await
                .map_err(failed)?
        };
        Ok(Some(Peek {
            size: meta.size,
            start,
        }))
    }

    fn api_signer(&self) -> Option<&ApiSigner> {
        match &self.signer {
            UrlSigner::Api(signer) => Some(signer),
            UrlSigner::Store(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first bytes of a PNG file: enough for `infer`.
    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

    async fn round_trip(storage: &dyn Storage) -> Result<(), AppError> {
        assert_eq!(storage.get("media/one").await?, None);
        storage
            .put("media/one", Bytes::from_static(PNG), "image/png")
            .await?;
        assert_eq!(
            storage.get("media/one").await?,
            Some(StoredObject {
                bytes: Bytes::from_static(PNG),
                content_type: "image/png".to_owned(),
            })
        );
        storage.delete("media/one").await?;
        assert_eq!(storage.get("media/one").await?, None);
        // Deleting what isn't there is no error.
        storage.delete("media/one").await?;
        Ok(())
    }

    #[tokio::test]
    async fn memory_keeps_bytes_and_their_type_until_deleted() -> Result<(), AppError> {
        round_trip(&ObjectStorage::memory()).await
    }

    #[tokio::test]
    async fn a_directory_keeps_bytes_and_detects_their_type_again()
    -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("storage-{}", uuid::Uuid::now_v7().simple()));
        let signing = Signing {
            api: ApiSigner::from_secret(b"a secret for the storage tests"),
            public_endpoint: None,
        };
        let storage = open(&StorageUrl::File(dir.join("nested")), signing)?;
        round_trip(storage.as_ref()).await?;
        // A type the bytes don't show is not kept.
        storage
            .put("notes/a.txt", Bytes::from_static(b"hello"), "text/plain")
            .await?;
        let read = storage.get("notes/a.txt").await?;
        assert_eq!(
            read.map(|object| object.content_type).as_deref(),
            Some(UNKNOWN_TYPE)
        );
        std::fs::remove_dir_all(dir)?;
        Ok(())
    }

    /// A store that can't sign has the API sign its URLs; `peek` reads an
    /// object's size and first bytes.
    #[tokio::test]
    async fn memory_urls_are_api_signed_and_objects_peek() -> Result<(), AppError> {
        let storage = ObjectStorage::memory();
        let url = storage
            .presign("media/one", Access::Upload, Duration::from_secs(60))
            .await?;
        assert_eq!(url.method, "PUT");
        let token = url.url.strip_prefix(super::super::STORAGE_ROUTE);
        let signer = storage.api_signer();
        let grant = token
            .zip(signer)
            .and_then(|(token, signer)| signer.check(token, "PUT", chrono::Utc::now()));
        assert_eq!(grant.map(|grant| grant.key).as_deref(), Some("media/one"));

        assert_eq!(storage.peek("media/one", 4).await?, None);
        storage
            .put("media/one", Bytes::from_static(PNG), "image/png")
            .await?;
        assert_eq!(
            storage.peek("media/one", 4).await?,
            Some(Peek {
                size: PNG.len() as u64,
                start: Bytes::from_static(&PNG[..4]),
            })
        );
        Ok(())
    }

    #[tokio::test]
    async fn a_key_outside_the_store_is_refused() -> Result<(), AppError> {
        let storage = ObjectStorage::memory();
        let put = storage
            .put("../escape", Bytes::from_static(PNG), "image/png")
            .await;
        assert!(matches!(put, Err(AppError::Internal(_))));
        assert_eq!(storage.get("../escape").await?, None);
        storage.delete("../escape").await?;
        Ok(())
    }
}
