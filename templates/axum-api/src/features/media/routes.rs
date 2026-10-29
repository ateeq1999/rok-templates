// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `media` routes, under `/api/v1/media`, and the API-signed storage
//! URLs it hands out, under `/api/v1/storage`.

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};

use super::handlers;
use crate::app::AppState;

/// Every `media` endpoint, under `/api/v1/media`, and the storage URLs,
/// under `/api/v1/storage`. `POST /media` and `PUT /storage/{token}` read a
/// raw body of at most `max_upload_bytes` (`MEDIA_MAX_UPLOAD_BYTES`): the
/// route's own `DefaultBodyLimit`, in place of the 2 MiB `app.rs` sets for
/// every route.
pub fn routes(max_upload_bytes: usize) -> Router<AppState> {
    Router::new()
        .route(
            "/api/v1/media",
            get(handlers::list)
                .post(handlers::upload)
                .layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
        .route("/api/v1/media/uploads", post(handlers::start_upload))
        .route(
            "/api/v1/media/{id}",
            get(handlers::metadata).delete(handlers::remove),
        )
        .route(
            "/api/v1/media/{id}/complete",
            post(handlers::complete_upload),
        )
        .route("/api/v1/media/{id}/content", get(handlers::content))
        .route("/api/v1/media/{id}/url", get(handlers::download_url))
        .route(
            "/api/v1/storage/{token}",
            get(handlers::get_object)
                .put(handlers::put_object)
                .layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
}
