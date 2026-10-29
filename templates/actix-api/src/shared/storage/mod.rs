// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Where uploaded bytes live: the [`Storage`] port (put, get and delete an
//! object by key, its content type kept), and [`ObjectStorage`], its one
//! implementation over the `object_store` crate, chosen by `STORAGE_URL`
//! ([`StorageUrl`]).
//!
//! Nothing here knows what an object is. A feature decides what may be
//! stored and generates the key (a fixed prefix and a random id, such as
//! `media/<uuid>`); [`is_valid_key`] is the backstop that keeps a key
//! inside the store.
//!
//! [`Storage::get`] answers the whole object in memory rather than a
//! stream: every upload is capped (`MEDIA_MAX_UPLOAD_BYTES`, 10 MiB by
//! default), and bytes keep the port free of any framework's body type.
//!
//! [`Storage::presign`] hands out [`PresignedUrl`]s, so a client can put or
//! get an object straight from the store (engine ADR 0024); a store that
//! can't sign has the API sign and serve them ([`ApiSigner`]).

mod object;
mod presign;
mod served;
mod url;

use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;

use crate::error::AppError;

pub use object::{ObjectStorage, Signing, open};
pub use presign::{Access, ApiSigner, Grant, PresignedUrl, STORAGE_ROUTE};
pub use served::{get_signed, put_signed};
pub use url::StorageUrl;

/// A stored object: its bytes and the content type it was put with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredObject {
    /// The object's bytes.
    pub bytes: Bytes,
    /// Its content type, such as `image/png`.
    pub content_type: String,
}

/// An object's size and its first bytes: enough to detect its type without
/// reading it whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peek {
    /// The object's size, in bytes.
    pub size: u64,
    /// Its first bytes, at most as many as asked for.
    pub start: Bytes,
}

/// The port features store bytes through (`AppState.storage`), by key.
/// A failure is an [`AppError::Internal`]; the caller sees a `500`.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Stores `bytes` under `key` with `content_type`, replacing any object
    /// the key held. A key [`is_valid_key`] refuses is an error.
    async fn put(&self, key: &str, bytes: Bytes, content_type: &str) -> Result<(), AppError>;

    /// The object under `key`, or `None` when there is none (or the key is
    /// not a valid one).
    async fn get(&self, key: &str) -> Result<Option<StoredObject>, AppError>;

    /// Deletes the object under `key`. A key holding nothing is no error.
    async fn delete(&self, key: &str) -> Result<(), AppError>;

    /// A URL granting `access` to the object under `key` for `ttl`: signed
    /// by the store, or by the API ([`Storage::api_signer`]).
    async fn presign(
        &self,
        key: &str,
        access: Access,
        ttl: Duration,
    ) -> Result<PresignedUrl, AppError>;

    /// The size and first `len` bytes of the object under `key`, or `None`
    /// when there is none.
    async fn peek(&self, key: &str, len: usize) -> Result<Option<Peek>, AppError>;

    /// The signer of the URLs the API serves for this store at
    /// [`STORAGE_ROUTE`], or `None` when the store signs its own.
    fn api_signer(&self) -> Option<&ApiSigner>;
}

/// Whether `key` may name an object: `/`-separated segments of
/// `A-Z a-z 0-9 . _ -`, none empty, `.` or `..`, so no key reaches outside
/// the store.
#[must_use]
pub fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_keys_inside_the_store_are_valid() {
        for good in ["a", "media/0123.png", "avatars/ab-cd_ef.jpg", "x/y/z"] {
            assert!(is_valid_key(good), "{good}");
        }
        for bad in [
            "",
            "/",
            "/abs",
            "a//b",
            "a/",
            "../secret",
            "a/../b",
            "./a",
            "a\\b",
            "a b",
            "a?b",
            "ü",
        ] {
            assert!(!is_valid_key(bad), "{bad}");
        }
    }
}
