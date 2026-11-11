// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Tickets`]: the short-lived token a subscription is opened with. A
//! browser's `EventSource` can't send an `Authorization` header, so a
//! signed-in caller first asks for a ticket (a guard decides its tenant),
//! then opens the stream with it in the query.

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::error::AppError;

/// What signs tickets apart from every other key derived from the secret.
const KEY_LABEL: &[u8] = b"rok live ticket signing";

/// What a ticket lets its holder do: open a subscription to `tenant`'s
/// Topics until `expires_at`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    /// The tenant the guard let the caller into.
    pub tenant: String,
    /// When the ticket stops opening subscriptions, in seconds since the
    /// Unix epoch. An open subscription outlives it.
    pub expires_at: u64,
}

/// Signs and checks tickets under a key derived from `JWT_SECRET`.
#[derive(Clone)]
pub struct Tickets {
    key: [u8; 32],
}

impl fmt::Debug for Tickets {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Tickets(..)")
    }
}

impl Tickets {
    /// How long a ticket opens subscriptions: long enough to connect.
    pub const TTL: Duration = Duration::from_secs(60);

    /// Tickets signed under a key derived from `secret`.
    #[must_use]
    pub fn from_secret(secret: &[u8]) -> Self {
        Tickets {
            key: hmac_sha256(secret, KEY_LABEL),
        }
    }

    /// A ticket to `tenant`'s Topics, good for [`Tickets::TTL`].
    ///
    /// # Errors
    ///
    /// When the clock is before the Unix epoch.
    pub fn issue(&self, tenant: &str) -> Result<String, AppError> {
        let ticket = Ticket {
            tenant: tenant.to_owned(),
            expires_at: now()? + Self::TTL.as_secs(),
        };
        let claims = serde_json::to_vec(&ticket).map_err(|err| AppError::Internal(err.into()))?;
        let claims = URL_SAFE_NO_PAD.encode(claims);
        let signature = URL_SAFE_NO_PAD.encode(hmac_sha256(&self.key, claims.as_bytes()));
        Ok(format!("{claims}.{signature}"))
    }

    /// What `token` grants, when this key signed it and it hasn't expired;
    /// otherwise a `401`.
    ///
    /// # Errors
    ///
    /// A `401` for a forged, malformed or expired ticket.
    pub fn check(&self, token: &str) -> Result<Ticket, AppError> {
        let refused = || AppError::Unauthorized("the ticket is invalid or expired".to_owned());
        let (claims, signature) = token.split_once('.').ok_or_else(refused)?;
        let signature = URL_SAFE_NO_PAD.decode(signature).map_err(|_| refused())?;
        let expected = hmac_sha256(&self.key, claims.as_bytes());
        if !bool::from(expected.as_slice().ct_eq(&signature)) {
            return Err(refused());
        }
        let claims = URL_SAFE_NO_PAD.decode(claims).map_err(|_| refused())?;
        let ticket: Ticket = serde_json::from_slice(&claims).map_err(|_| refused())?;
        if ticket.expires_at <= now()? {
            return Err(refused());
        }
        Ok(ticket)
    }
}

/// Seconds since the Unix epoch.
fn now() -> Result<u64, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(|err| AppError::Internal(Box::new(err)))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as hmac::KeyInit>::new_from_slice(key)
        .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
    mac.update(message);
    mac.finalize().into_bytes().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ticket_opens_its_tenant_only_under_its_key() -> Result<(), AppError> {
        let tickets = Tickets::from_secret(b"secret");
        let token = tickets.issue("tenant-a")?;
        assert_eq!(tickets.check(&token)?.tenant, "tenant-a");
        assert!(Tickets::from_secret(b"other").check(&token).is_err());
        let (claims, signature) = token.split_once('.').unwrap_or_default();
        let forged = format!("{}x.{signature}", &claims[..claims.len() - 1]);
        assert!(tickets.check(&forged).is_err());
        assert!(tickets.check("not-a-ticket").is_err());
        Ok(())
    }
}
