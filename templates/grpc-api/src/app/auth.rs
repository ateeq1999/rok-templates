// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`authenticate`]: the caller check in front of the API's services. It
//! puts the [`AuthUser`](crate::shared::auth::AuthUser) the credentials
//! name in each request's extensions, where the generated services read it
//! (`shared/grpc.rs`); a request it puts none in reaches them without a
//! caller, so every service that needs one answers `UNAUTHENTICATED`. The
//! health and reflection services are added outside it
//! ([`services`](super::services)): they need no credentials.

use tonic::service::Routes;

use super::AppState;

/// `api` behind the caller check, over the state (its JWT keys and pool).
///
/// It checks nothing yet: no request gets a caller, so every service that
/// needs one answers `UNAUTHENTICATED`. Verify the bearer token
/// (`AppState::keys`) or the API key (`shared::security::api_key`) here, as
/// the axum template's `shared/auth.rs` does, and insert the caller.
#[must_use]
pub fn authenticate(api: Routes, _state: &AppState) -> Routes {
    api
}
