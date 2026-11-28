// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Signed webhooks: a provider calling back (a payment settled, a parcel
//! delivered) is let in only once its signature is the provider's.
//!
//! A [`WebhookVerifier`] is that check, written once per provider, as a
//! unit struct named after it. An HTTP handler takes `SignedWebhook<V>`
//! (`shared/auth.rs`), which reads the request's headers and raw body,
//! asks `V`, and hands the handler both, or answers `401` (the gRPC
//! template serves no HTTP routes, so it has the verifier but no
//! extractor):
//!
//! ```text
//! pub async fn payment_settled(
//!     State(state): State<AppState>,
//!     webhook: SignedWebhook<PaymentsSignature>,
//! ) -> Result<StatusCode, AppError>
//! ```
//!
//! The verifier is yours: each provider signs its own way. A provider
//! following [Standard Webhooks](https://www.standardwebhooks.com/) is
//! checked with a [`WebhookSecret`]; it signs too, so a test's fake
//! provider signs its callbacks exactly as the real one does. The
//! template ships no provider (the guide's "A provider that calls back"
//! recipe shows one).

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use crate::app::AppState;

/// What a verifier is shown of a callback: its headers and its body, the
/// bytes exactly as they arrived (a signature covers those, not a
/// re-encoding of them).
#[derive(Debug, Clone, Copy)]
pub struct Delivery<'a> {
    headers: &'a [(&'a str, &'a str)],
    body: &'a [u8],
}

impl<'a> Delivery<'a> {
    /// A callback of `headers` (name and value) and `body`.
    #[must_use]
    pub fn new(headers: &'a [(&'a str, &'a str)], body: &'a [u8]) -> Self {
        Delivery { headers, body }
    }

    /// The value of the header `name` (any case), the first when it
    /// repeats.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&'a str> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| *value)
    }

    /// The raw body.
    #[must_use]
    pub fn body(&self) -> &'a [u8] {
        self.body
    }
}

/// A provider's signature check. Implement it on a unit struct named after
/// the provider (`PaymentsSignature`), reading the secret from the state.
pub trait WebhookVerifier {
    /// Whether `delivery` is signed by the provider. `false` is a `401`,
    /// and the handler is not called.
    fn verify(state: &AppState, delivery: &Delivery<'_>) -> bool;
}

/// How far a callback's timestamp may be from now, either way, in seconds:
/// an older one is a replay.
pub const TOLERANCE_SECS: i64 = 5 * 60;

/// The header carrying a Standard Webhooks callback's id.
pub const ID_HEADER: &str = "webhook-id";
/// The header carrying its timestamp, in seconds since the Unix epoch.
pub const TIMESTAMP_HEADER: &str = "webhook-timestamp";
/// The header carrying its signatures: `v1,<base64>`, space-separated.
pub const SIGNATURE_HEADER: &str = "webhook-signature";

/// A secret that isn't `whsec_` and base64.
#[derive(Debug, thiserror::Error)]
#[error("a webhook secret is `whsec_` followed by base64")]
pub struct InvalidWebhookSecret;

/// A [Standard Webhooks](https://www.standardwebhooks.com/) secret: it
/// signs a callback (HMAC-SHA256 over `{id}.{timestamp}.{body}`) and
/// checks one.
#[derive(Clone)]
pub struct WebhookSecret {
    key: Zeroizing<Vec<u8>>,
}

impl fmt::Debug for WebhookSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WebhookSecret(..)")
    }
}

impl WebhookSecret {
    /// The secret the provider gave, `whsec_<base64>` (the prefix may be
    /// left out).
    ///
    /// # Errors
    ///
    /// [`InvalidWebhookSecret`] when the rest isn't base64, or is empty.
    pub fn new(secret: &str) -> Result<Self, InvalidWebhookSecret> {
        let encoded = secret.trim();
        let encoded = encoded.strip_prefix("whsec_").unwrap_or(encoded);
        let key = STANDARD.decode(encoded).map_err(|_| InvalidWebhookSecret)?;
        if key.is_empty() {
            return Err(InvalidWebhookSecret);
        }
        Ok(WebhookSecret {
            key: Zeroizing::new(key),
        })
    }

    /// A new random secret, `whsec_` and 32 random bytes: a fake
    /// provider's, in a test.
    #[must_use]
    pub fn random() -> Self {
        WebhookSecret {
            key: Zeroizing::new(rand::random::<[u8; 32]>().to_vec()),
        }
    }

    /// The headers a callback with `id`, sent at `timestamp` (seconds
    /// since the Unix epoch), carries for `body`: what the provider sends,
    /// and what a fake provider sends in a test.
    #[must_use]
    pub fn headers(&self, id: &str, timestamp: i64, body: &[u8]) -> [(&'static str, String); 3] {
        let signature = STANDARD.encode(self.mac(id, timestamp, body));
        [
            (ID_HEADER, id.to_owned()),
            (TIMESTAMP_HEADER, timestamp.to_string()),
            (SIGNATURE_HEADER, format!("v1,{signature}")),
        ]
    }

    /// Whether `delivery` is signed with this secret and was sent within
    /// [`TOLERANCE_SECS`] of now.
    #[must_use]
    pub fn verify(&self, delivery: &Delivery<'_>) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX)
            });
        self.verify_at(delivery, now)
    }

    /// [`verify`](Self::verify), as at `now` (seconds since the Unix
    /// epoch). One matching signature is enough: a provider rotating its
    /// secret sends one per secret.
    #[must_use]
    pub fn verify_at(&self, delivery: &Delivery<'_>, now: i64) -> bool {
        let (Some(id), Some(timestamp), Some(signatures)) = (
            delivery.header(ID_HEADER),
            delivery.header(TIMESTAMP_HEADER),
            delivery.header(SIGNATURE_HEADER),
        ) else {
            return false;
        };
        let Ok(timestamp) = timestamp.trim().parse::<i64>() else {
            return false;
        };
        if now.abs_diff(timestamp) > TOLERANCE_SECS.unsigned_abs() {
            return false;
        }
        let expected = self.mac(id, timestamp, delivery.body());
        signatures
            .split_whitespace()
            .filter_map(|signature| signature.strip_prefix("v1,"))
            .filter_map(|signature| STANDARD.decode(signature).ok())
            .any(|signature| bool::from(signature.ct_eq(&expected)))
    }

    /// HMAC-SHA256 over `{id}.{timestamp}.{body}`.
    fn mac(&self, id: &str, timestamp: i64, body: &[u8]) -> [u8; 32] {
        let mut mac = <Hmac<Sha256> as hmac::KeyInit>::new_from_slice(&self.key)
            .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
        mac.update(format!("{id}.{timestamp}.").as_bytes());
        mac.update(body);
        mac.finalize().into_bytes().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_700_000_000;

    /// `secret`'s headers for `body`, signed at `at`, borrowed as a
    /// delivery's.
    fn signed(secret: &WebhookSecret, at: i64, body: &[u8]) -> [(&'static str, String); 3] {
        secret.headers("msg_1", at, body)
    }

    fn verify(secret: &WebhookSecret, headers: &[(&str, String)], body: &[u8]) -> bool {
        let headers: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        secret.verify_at(&Delivery::new(&headers, body), NOW)
    }

    #[test]
    fn a_correctly_signed_body_is_accepted() {
        let secret = WebhookSecret::random();
        let body = br#"{"type":"charge.settled"}"#;
        assert!(verify(&secret, &signed(&secret, NOW, body), body));
    }

    #[test]
    fn a_tampered_body_or_another_secret_is_refused() {
        let secret = WebhookSecret::random();
        let body = br#"{"amount":100}"#;
        let headers = signed(&secret, NOW, body);
        assert!(!verify(&secret, &headers, br#"{"amount":900}"#));
        assert!(!verify(&WebhookSecret::random(), &headers, body));
    }

    #[test]
    fn a_stale_or_unsigned_callback_is_refused() {
        let secret = WebhookSecret::random();
        let body = b"{}";
        let stale = signed(&secret, NOW - TOLERANCE_SECS - 1, body);
        assert!(!verify(&secret, &stale, body));
        assert!(!verify(&secret, &signed(&secret, NOW, body)[..2], body));
    }

    #[test]
    fn one_matching_signature_of_several_is_enough() {
        let (old, new) = (WebhookSecret::random(), WebhookSecret::random());
        let body = b"{}";
        let [id, at, signature] = signed(&new, NOW, body);
        let both = format!("{} {}", old.headers("msg_1", NOW, body)[2].1, signature.1);
        assert!(verify(&new, &[id, at, (SIGNATURE_HEADER, both)], body));
    }

    #[test]
    fn a_secret_is_read_with_or_without_its_prefix() -> Result<(), InvalidWebhookSecret> {
        let key = STANDARD.encode([7u8; 24]);
        let body = b"{}";
        let prefixed = WebhookSecret::new(&format!("whsec_{key}"))?;
        let bare = WebhookSecret::new(&key)?;
        assert!(verify(&bare, &signed(&prefixed, NOW, body), body));
        assert!(WebhookSecret::new("whsec_not base64!").is_err());
        assert!(WebhookSecret::new("whsec_").is_err());
        Ok(())
    }
}
