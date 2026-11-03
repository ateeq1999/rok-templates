// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The URLs the API signs, served at [`STORAGE_ROUTE`](super::STORAGE_ROUTE):
//! a framework's handler reads the token and the body, and these do the
//! rest. The token is the only credential: it grants one method on one key
//! until it expires, and a token that doesn't is a `403`, whatever is wrong
//! with it.

use bytes::Bytes;

use super::{Access, Grant, Storage, StoredObject};
use crate::error::AppError;

/// Stores `bytes` (of `content_type`) under the key an upload token grants.
/// The handler caps the body at `MEDIA_MAX_UPLOAD_BYTES`.
pub async fn put_signed(
    storage: &dyn Storage,
    token: &str,
    content_type: &str,
    bytes: Bytes,
) -> Result<(), AppError> {
    let grant = granted(storage, token, Access::Upload)?;
    storage.put(&grant.key, bytes, content_type).await
}

/// The object under the key a download token grants (`404` when gone).
pub async fn get_signed(storage: &dyn Storage, token: &str) -> Result<StoredObject, AppError> {
    let grant = granted(storage, token, Access::Download)?;
    storage
        .get(&grant.key)
        .await?
        .ok_or_else(|| AppError::NotFound("no such file".to_owned()))
}

/// What `token` grants for `access`, or `403`: also on a store that signs
/// its own URLs, whose objects the API never serves this way.
fn granted(storage: &dyn Storage, token: &str, access: Access) -> Result<Grant, AppError> {
    storage
        .api_signer()
        .and_then(|signer| signer.check(token, access.method(), chrono::Utc::now()))
        .ok_or_else(|| AppError::Forbidden("the link is invalid or has expired".to_owned()))
}
