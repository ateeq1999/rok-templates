// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Who is calling: the [`AuthUser`] and [`AdminUser`] extractors.
//!
//! Each verifies the HS256 JWT from `Authorization: Bearer <token>` with the
//! [`JwtKeys`] that `app/` puts in the request's extensions, then loads
//! the account with one query (`db::queries::session::session_state`, on
//! the `PgPool` in the same extensions): its `token_version`, whether it is
//! active, and its role. A missing or invalid token, a soft-deleted or
//! disabled account, or a token issued before the last `token_version` bump
//! is a `401`. The role comes from the database, not the token, so a
//! promotion or demotion applies on the next request; a caller who is not
//! an admin is a `403` for `AdminUser`. Both answer through [`AppError`].
//!
//! An API key stands in for the JWT: `Authorization: Bearer rok_…` (a key
//! is told apart by its prefix) or `X-API-Key`. It is checked by
//! [`api_key::authenticate`]: unknown, revoked or expired keys, and keys of
//! a deleted or disabled account, are a `401`; a `read` key on a method
//! that isn't safe, and any key on `/api/v1/auth` or `/api/v1/api-keys`, a
//! `403`. [`AuthUser::credential`] says which one the caller sent.
//!
//! Reading from the extensions, not the router's state, lets the same
//! extractors serve the generated routes (stateless, `features::router`)
//! and the hand-written ones (`Router<AppState>`).
//!
//! [`Allowed`] goes further: it runs a [`Guard`], an authorization rule
//! written once, and hands the handler its grant. It reads the
//! [`AppState`], so it serves the hand-written routes.
//!
//! [`SignedWebhook`] lets in a provider calling back instead of a person:
//! a [`WebhookVerifier`] checks the callback's signature over its raw body.

use std::marker::PhantomData;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{FromRef, FromRequest, FromRequestParts, OriginalUri, Request};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, header};
use sqlx::PgPool;
use uuid::Uuid;

use crate::app::AppState;
use crate::db::queries::session::{SessionStateRow, session_state};
pub use crate::db::types::Role;
use crate::error::AppError;
pub use crate::shared::security::api_key::Credential;
use crate::shared::security::api_key::{self, Presented};
pub use crate::shared::security::guard::{Guard, require};
pub use crate::shared::security::jwt::{Claims, JwtKeys};
pub use crate::shared::security::webhook::{Delivery, WebhookSecret, WebhookVerifier};
use crate::shared::telemetry;
use crate::shared::validation::Path;

/// A signed-in caller with a live, active account. Take it in a handler to
/// require one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthUser {
    /// The token's `sub`.
    pub id: Uuid,
    /// The account's role, from the database.
    pub role: Role,
    /// How the caller proved who they are: a session or an API key.
    pub credential: Credential,
}

/// A signed-in caller whose account has the `admin` role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminUser(pub AuthUser);

impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, AppError> {
        let wiring = |what: &str| {
            AppError::Internal(format!("{what} is not in the request extensions").into())
        };
        let pool = parts
            .extensions
            .get::<PgPool>()
            .ok_or_else(|| wiring("the `PgPool`"))?
            .clone();
        let user = match credentials(&parts.headers)? {
            Presented::Session(token) => {
                let keys = parts
                    .extensions
                    .get::<Arc<JwtKeys>>()
                    .ok_or_else(|| wiring("`JwtKeys`"))?;
                let claims = keys.decode(token)?;
                let account = session_state(&pool, claims.sub.into()).await?;
                admit(&claims, account)?
            }
            Presented::ApiKey(key) => {
                // A nested router sees its own part of the path; the rules
                // name whole paths.
                let path = parts
                    .extensions
                    .get::<OriginalUri>()
                    .map_or_else(|| parts.uri.path(), |original| original.path());
                let owner = api_key::authenticate(&pool, key, parts.method.as_str(), path).await?;
                AuthUser {
                    id: owner.id,
                    role: owner.role,
                    credential: Credential::ApiKey(owner.scope),
                }
            }
        };
        // The request's span names its caller from here on.
        telemetry::record_user(user.id);
        Ok(user)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        match user.role {
            Role::Admin => Ok(AdminUser(user)),
            Role::User => Err(AppError::Forbidden("this needs the admin role".to_owned())),
        }
    }
}

/// A caller the guard `G` let in, and its grant: how it was let in. Take
/// it in a handler to require the rule, ahead of the body extractor (the
/// last argument), so a caller who is refused gets the `403` before the
/// body is read:
///
/// ```text
/// Allowed(access): Allowed<PartnerAccess>   // the grant, matched on
/// _: Allowed<Finance>                       // a yes or no
/// ```
///
/// It reads the caller as [`AuthUser`] does (none is a `401`), `G`'s path
/// parameters as `Path<G::Path>` does (a malformed one is a `400`), then
/// asks `G` ([`require`]): a refusal is a `403`.
pub struct Allowed<G: Guard>(pub G::Grant);

impl<G: Guard> Allowed<G> {
    /// The grant.
    #[must_use]
    pub fn into_inner(self) -> G::Grant {
        self.0
    }
}

impl<G: Guard<Grant: std::fmt::Debug>> std::fmt::Debug for Allowed<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Allowed").field(&self.0).finish()
    }
}

impl<G, S> FromRequestParts<S> for Allowed<G>
where
    G: Guard,
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AppError> {
        let caller = AuthUser::from_request_parts(parts, state).await?;
        let Path(path) = Path::<G::Path>::from_request_parts(parts, state).await?;
        let grant = require::<G>(&caller, &AppState::from_ref(state), &path).await?;
        Ok(Allowed(grant))
    }
}

/// A provider's callback whose signature the verifier `V` accepted: its
/// headers and its raw body, for the handler to parse. Take it in a
/// handler instead of a body extractor (it reads the body itself):
///
/// ```text
/// webhook: SignedWebhook<PaymentsSignature>
/// let event: PaymentEvent = serde_json::from_slice(&webhook.body)?;
/// ```
///
/// A callback `V` refuses is a `401`, and the handler is not called. A
/// body over the route's `DefaultBodyLimit` is a `413`.
pub struct SignedWebhook<V: WebhookVerifier> {
    /// The callback's headers.
    pub headers: HeaderMap,
    /// The callback's body, exactly as it arrived.
    pub body: Bytes,
    verifier: PhantomData<fn() -> V>,
}

impl<V: WebhookVerifier> std::fmt::Debug for SignedWebhook<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedWebhook")
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

impl<V, S> FromRequest<S> for SignedWebhook<V>
where
    V: WebhookVerifier,
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, AppError> {
        let headers = req.headers().clone();
        let body = Bytes::from_request(req, state).await.map_err(|rejection| {
            if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
                AppError::PayloadTooLarge(rejection.body_text())
            } else {
                AppError::BadRequest(rejection.body_text())
            }
        })?;
        let named: Vec<(&str, &str)> = headers
            .iter()
            .filter_map(|(name, value)| Some((name.as_str(), value.to_str().ok()?)))
            .collect();
        if !V::verify(&AppState::from_ref(state), &Delivery::new(&named, &body)) {
            return Err(AppError::Unauthorized(
                "the callback's signature is not valid".to_owned(),
            ));
        }
        Ok(SignedWebhook {
            headers,
            body,
            verifier: PhantomData,
        })
    }
}

/// The credential `headers` present: a bearer token, or an API key.
fn credentials(headers: &HeaderMap) -> Result<Presented<'_>, AppError> {
    let value = |name| headers.get(name).and_then(|value| value.to_str().ok());
    api_key::presented(
        value(header::AUTHORIZATION.as_str()),
        value(api_key::HEADER),
    )
}

/// The caller of a valid token, given its account's `state` (`None` for no
/// live account).
fn admit(claims: &Claims, state: Option<SessionStateRow>) -> Result<AuthUser, AppError> {
    let revoked = || AppError::Unauthorized("the token is no longer valid".to_owned());
    let state = state.ok_or_else(revoked)?;
    if !state.is_active || state.token_version != claims.tv {
        return Err(revoked());
    }
    Ok(AuthUser {
        id: claims.sub,
        role: state.role,
        credential: Credential::Session,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims(tv: i32) -> Claims {
        Claims {
            sub: Uuid::now_v7(),
            role: Role::User,
            tv,
            exp: u64::MAX,
        }
    }

    fn state(token_version: i32, is_active: bool, role: Role) -> SessionStateRow {
        SessionStateRow {
            token_version,
            is_active,
            role,
        }
    }

    #[test]
    fn a_live_account_is_admitted_with_its_role_from_the_database() -> Result<(), AppError> {
        let claims = claims(2);
        let user = admit(&claims, Some(state(2, true, Role::Admin)))?;
        assert_eq!(
            user,
            AuthUser {
                id: claims.sub,
                role: Role::Admin,
                credential: Credential::Session,
            }
        );
        Ok(())
    }

    #[test]
    fn a_stale_token_version_is_rejected() {
        let rejected = admit(&claims(1), Some(state(2, true, Role::User)));
        assert!(
            matches!(rejected, Err(AppError::Unauthorized(_))),
            "{rejected:?}"
        );
    }

    #[test]
    fn a_disabled_or_missing_account_is_rejected() {
        let rejected = admit(&claims(1), Some(state(1, false, Role::User)));
        assert!(
            matches!(rejected, Err(AppError::Unauthorized(_))),
            "{rejected:?}"
        );
        let rejected = admit(&claims(1), None);
        assert!(
            matches!(rejected, Err(AppError::Unauthorized(_))),
            "{rejected:?}"
        );
    }
}
