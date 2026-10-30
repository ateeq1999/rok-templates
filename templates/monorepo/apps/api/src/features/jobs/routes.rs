// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `jobs` routes: `/api/v1/jobs`.

use axum::Router;
use axum::routing::{delete, get, post};

use super::handlers;
use crate::app::AppState;

/// The job queue, for admins (`AdminUser` on every handler).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v1/jobs", get(handlers::list))
        .route("/api/v1/jobs/{id}", delete(handlers::cancel))
        .route("/api/v1/jobs/{id}/retry", post(handlers::retry))
}
