// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Presigned URLs (engine ADR 0024): short-lived URLs that let a client put
//! or get one object without its bytes passing through the API.
//!
//! An S3-compatible store signs them itself. A store that can't (local
//! files, memory) gets URLs the API signs and serves at
//! [`STORAGE_ROUTE`]: an HMAC-SHA256 over the method, the key and the
//! expiry, under a key derived from `JWT_SECRET` ([`ApiSigner`]). A client's
//! flow is the same either way.

use std::fmt;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

/// What a presigned URL lets its holder do with its object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Put the object's bytes: `PUT`.
    Upload,
    /// Get the object's bytes: `GET`.
    Download,
}

impl Access {
    /// The HTTP method the URL is for.
    #[must_use]
    pub fn method(self) -> &'static str {
        match self {
            Access::Upload => "PUT",
            Access::Download => "GET",
        }
    }
}

/// A short-lived URL for one object: its holder sends [`method`](Self::method)
/// to [`url`](Self::url) before [`expires_at`](Self::expires_at).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedUrl {
    /// `PUT` for an upload, `GET` for a download.
    pub method: &'static str,
    /// The URL: absolute from an S3-compatible store, relative to the API
    /// (`/api/v1/storage/…`) when the API signs it.
    pub url: String,
    /// When the URL stops working.
    pub expires_at: DateTime<Utc>,
}

/// Where the API serves the URLs it signs, with the token after it.
pub const STORAGE_ROUTE: &str = "/api/v1/storage/";

/// The label the storage signing key is derived from `JWT_SECRET` under, so
/// the key is never the token signing key itself.
const KEY_LABEL: &[u8] = b"rok storage url signing";

/// Signs and checks the URLs the API serves itself, for a store that can't
/// presign.
#[derive(Clone)]
pub struct ApiSigner {
    key: [u8; 32],
}

impl fmt::Debug for ApiSigner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiSigner(..)")
    }
}

/// What an API-signed token grants: `access` to `key` until `expires`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    /// The method the token is for.
    pub access: Access,
    /// The object's key.
    pub key: String,
    /// When the token stops working.
    pub expires_at: DateTime<Utc>,
}

impl ApiSigner {
    /// The signer whose key is derived from `secret` (`JWT_SECRET`).
    #[must_use]
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            key: hmac_sha256(secret, KEY_LABEL),
        }
    }

    /// A URL granting `access` to `key` for `ttl`, from `now`.
    #[must_use]
    pub fn sign(
        &self,
        key: &str,
        access: Access,
        ttl: Duration,
        now: DateTime<Utc>,
    ) -> PresignedUrl {
        let expires_at = now + chrono::Duration::from_std(ttl).unwrap_or(chrono::Duration::zero());
        let payload = format!("{}\n{key}\n{}", access.method(), expires_at.timestamp());
        let mac = hmac_sha256(&self.key, payload.as_bytes());
        let token = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(payload),
            URL_SAFE_NO_PAD.encode(mac)
        );
        PresignedUrl {
            method: access.method(),
            url: format!("{STORAGE_ROUTE}{token}"),
            expires_at,
        }
    }

    /// What `token` grants, when the API signed it, it is for `method` and
    /// it hasn't expired at `now`; `None` otherwise, without saying why.
    #[must_use]
    pub fn check(&self, token: &str, method: &str, now: DateTime<Utc>) -> Option<Grant> {
        let (payload, mac) = token.split_once('.')?;
        let payload = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let mac = URL_SAFE_NO_PAD.decode(mac).ok()?;
        let expected = hmac_sha256(&self.key, &payload);
        if !bool::from(expected.ct_eq(mac.as_slice())) {
            return None;
        }
        let payload = String::from_utf8(payload).ok()?;
        let mut parts = payload.splitn(3, '\n');
        let access = match parts.next()? {
            "PUT" => Access::Upload,
            "GET" => Access::Download,
            _ => return None,
        };
        let key = parts.next()?.to_owned();
        let expires_at = DateTime::from_timestamp(parts.next()?.parse().ok()?, 0)?;
        (access.method() == method && now < expires_at).then_some(Grant {
            access,
            key,
            expires_at,
        })
    }
}

/// HMAC-SHA256 of `message` under `key`.
fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as hmac::KeyInit>::new_from_slice(key)
        .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
    mac.update(message);
    mac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000, 0).unwrap_or_default()
    }

    #[test]
    fn a_signed_url_grants_its_method_on_its_key_until_it_expires() {
        let signer = ApiSigner::from_secret(b"a long enough secret for the tests");
        let url = signer.sign("media/abc", Access::Upload, Duration::from_secs(60), now());
        assert_eq!(url.method, "PUT");
        assert_eq!(url.expires_at, now() + chrono::Duration::seconds(60));
        let token = url.url.strip_prefix(STORAGE_ROUTE).unwrap_or_default();

        let grant = signer.check(token, "PUT", now());
        assert_eq!(
            grant,
            Some(Grant {
                access: Access::Upload,
                key: "media/abc".to_owned(),
                expires_at: url.expires_at,
            })
        );
        // Another method, a later time, another signer or a changed token:
        // nothing.
        assert_eq!(signer.check(token, "GET", now()), None);
        let later = now() + chrono::Duration::seconds(60);
        assert_eq!(signer.check(token, "PUT", later), None);
        let other = ApiSigner::from_secret(b"another secret, as long as the first");
        assert_eq!(other.check(token, "PUT", now()), None);
        let forged = token.replacen('A', "B", 1);
        assert_eq!(signer.check(&forged, "PUT", now()), None);
        assert_eq!(signer.check("not a token", "PUT", now()), None);
    }
}
