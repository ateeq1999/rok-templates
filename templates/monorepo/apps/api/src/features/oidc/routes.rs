// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` routes, under `/api/v1/auth/oidc`, behind the `/auth` rate
//! limit.

use axum::Router;
use axum::routing::{delete, get, post};

use super::handlers;
use crate::app::AppState;
use crate::shared::rate_limit::RateLimit;

/// Every `oidc` endpoint, under `/api/v1/auth/oidc`, each request counted
/// against `limit` like the rest of `/auth`. Its routes are more specific
/// than the `auth` router nested at `/api/v1/auth`, so they win.
pub fn routes(limit: &RateLimit) -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/auth/oidc/identities",
            get(handlers::list_identities),
        )
        .route(
            "/api/v1/auth/oidc/identities/{id}",
            delete(handlers::unlink_identity),
        )
        .route("/api/v1/auth/oidc/{provider}/start", get(handlers::start))
        .route(
            "/api/v1/auth/oidc/{provider}/callback",
            get(handlers::callback),
        )
        .route(
            "/api/v1/auth/oidc/{provider}/link",
            post(handlers::link_identity),
        )
        .layer(limit.clone())
}
