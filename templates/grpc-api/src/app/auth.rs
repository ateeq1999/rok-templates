// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`authenticate`]: the caller check in front of the API's services. It
//! checks the credentials a request's metadata carries, as the axum
//! template's `AuthUser` extractor checks the headers, and puts the
//! [`AuthUser`] they name in the request's extensions, where the generated
//! services read it (`shared/grpc.rs`'s `caller` and `admin`). The health
//! and reflection services are added outside it
//! ([`services`](super::services)): they need no credentials.
//!
//! A bearer token (`authorization: Bearer <jwt>`) is verified with
//! [`AppState::keys`] (`JWT_SECRET`, and `JWT_SECRET_PREVIOUS` while it is
//! rotated out), then its account is loaded with one query
//! (`db::queries::session::session_state`): an expired or forged token, a
//! soft-deleted or disabled account, or a token issued before the last
//! `token_version` bump is `UNAUTHENTICATED`. The role comes from the
//! database, not the token.
//!
//! An API key (`authorization: Bearer rok_…` or `x-api-key: rok_…`) is
//! checked by [`api_key::authenticate`]: an unknown, revoked or expired
//! key, or one of a deleted or disabled account, is `UNAUTHENTICATED`; a
//! `read` key on an RPC that isn't a read ([`READS`]) `PERMISSION_DENIED`.
//!
//! A request with neither reaches the services without a caller: each one
//! that needs a caller answers `UNAUTHENTICATED`, so a service of your own
//! may serve anonymous callers. This API verifies credentials; it never
//! issues them: signing in and making API keys stay on the HTTP API.

use std::convert::Infallible;
use std::sync::Arc;

use sqlx::PgPool;
use tonic::Status;
use tonic::codegen::http::{HeaderMap, Request, Response, header};
use tonic::codegen::{BoxFuture, Context, Poll, Service};
use tonic::service::{AxumBody, Routes};
use tower::Layer;

use super::AppState;
use crate::db::queries::session::{SessionStateRow, session_state};
use crate::error::AppError;
use crate::shared::auth::{AuthUser, Claims, Credential, JwtKeys};
use crate::shared::security::api_key::{self, Presented};
use crate::shared::telemetry;

/// The RPCs a `read` API key may call, by method name: the generated
/// reads. Name your own read-only RPCs here; every other RPC is a write.
pub const READS: &[&str] = &["Get", "List"];

/// `api` behind the caller check, over the state's JWT keys and pool.
#[must_use]
pub fn authenticate(api: Routes, state: &AppState) -> Routes {
    let check = Check {
        keys: Arc::clone(&state.keys),
        pool: state.pool.clone(),
    };
    Routes::from(api.into_axum_router().layer(check))
}

/// What the caller check verifies credentials with, and the layer that
/// puts it in front of a service.
#[derive(Clone)]
struct Check {
    keys: Arc<JwtKeys>,
    pool: PgPool,
}

impl<S> Layer<S> for Check {
    type Service = Checked<S>;

    fn layer(&self, inner: S) -> Checked<S> {
        Checked {
            check: self.clone(),
            inner,
        }
    }
}

/// `inner` behind the caller check: a request with valid credentials
/// reaches it with their [`AuthUser`] in its extensions, one with none
/// without a caller, and one with invalid credentials not at all.
#[derive(Clone)]
struct Checked<S> {
    check: Check,
    inner: S,
}

impl<S> Service<Request<AxumBody>> for Checked<S>
where
    S: Service<Request<AxumBody>, Response = Response<AxumBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<AxumBody>;
    type Error = Infallible;
    type Future = BoxFuture<Response<AxumBody>, Infallible>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Infallible>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<AxumBody>) -> Self::Future {
        let check = self.check.clone();
        // The clone that was polled ready serves this request.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move {
            match check.caller(request.headers(), request.uri().path()).await {
                Ok(Some(user)) => {
                    // The request's span names its caller from here on.
                    telemetry::record_user(user.id);
                    request.extensions_mut().insert(user);
                }
                Ok(None) => {}
                Err(error) => return Ok(Status::from(error).into_http()),
            }
            inner.call(request).await
        })
    }
}

impl Check {
    /// The caller the credentials in `headers` name, calling the RPC at
    /// `path` (`/<package>.<Service>/<Method>`), or `None` when there are
    /// none.
    async fn caller(&self, headers: &HeaderMap, path: &str) -> Result<Option<AuthUser>, AppError> {
        let value = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
        let (authorization, key) = (
            value(header::AUTHORIZATION.as_str()),
            value(api_key::HEADER),
        );
        if authorization.is_none() && key.is_none() {
            return Ok(None);
        }
        let user = match api_key::presented(authorization, key)? {
            Presented::Session(token) => {
                let claims = self.keys.decode(token)?;
                let account = session_state(&self.pool, claims.sub.into()).await?;
                admit(&claims, account)?
            }
            Presented::ApiKey(key) => {
                let owner = api_key::authenticate(&self.pool, key, method(path), path).await?;
                AuthUser {
                    id: owner.id,
                    role: owner.role,
                    credential: Credential::ApiKey(owner.scope),
                }
            }
        };
        Ok(Some(user))
    }
}

/// The HTTP method [`api_key::check_reach`] judges the RPC at `path` by:
/// `GET` for a read ([`READS`]), `POST`, as gRPC sends every call, for the
/// rest.
fn method(path: &str) -> &'static str {
    let rpc = path.rsplit('/').next().unwrap_or_default();
    if READS.contains(&rpc) { "GET" } else { "POST" }
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
    use uuid::Uuid;

    use super::*;
    use crate::shared::auth::Role;

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
    fn a_stale_token_version_or_a_disabled_or_missing_account_is_rejected() {
        for state in [
            Some(state(2, true, Role::User)),
            Some(state(1, false, Role::User)),
            None,
        ] {
            let rejected = admit(&claims(1), state);
            assert!(
                matches!(rejected, Err(AppError::Unauthorized(_))),
                "{rejected:?}"
            );
        }
    }

    #[test]
    fn only_the_reads_are_judged_as_gets() {
        assert_eq!(method("/grpc_api.v1.UsersService/Get"), "GET");
        assert_eq!(method("/grpc_api.v1.UsersService/List"), "GET");
        assert_eq!(method("/grpc_api.v1.UsersService/Update"), "POST");
        assert_eq!(method("/grpc_api.v1.UsersService/GetOrCreate"), "POST");
    }
}
