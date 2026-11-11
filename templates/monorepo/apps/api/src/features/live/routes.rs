// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` routes: `/api/v1/live`.

use std::fmt::Display;

use axum::Router;
use axum::routing::{get, post};

use super::handlers;
use crate::app::AppState;
use crate::shared::auth::Guard;

/// Tickets for the callers `G` lets in, its grant their tenant, and the
/// stream a ticket opens. `app/` mounts `routes::<Subscriber>()`.
pub fn routes<G>() -> Router<AppState>
where
    G: Guard<Path = ()> + Send + 'static,
    G::Grant: Display,
{
    Router::new()
        .route("/api/v1/live", get(handlers::subscribe))
        .route("/api/v1/live/tickets", post(handlers::ticket::<G>))
}
