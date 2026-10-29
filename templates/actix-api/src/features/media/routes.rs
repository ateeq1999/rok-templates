// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `media` routes, under `/api/v1/media`, and the API-signed storage
//! URLs it hands out, under `/api/v1/storage`.

use actix_web::web;

use super::handlers;
use crate::shared::validation::{path_config, query_config};

/// Mounts every `media` endpoint under `/api/v1/media`, and the storage
/// URLs under `/api/v1/storage`. `POST /media` and `PUT /storage/{token}`
/// read a raw body of at most `MEDIA_MAX_UPLOAD_BYTES` (`MediaService`'s
/// `max_upload_bytes`), after the request is authorized (see
/// [`crate::shared::body`]).
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/media")
            .app_data(path_config())
            .app_data(query_config())
            .service(
                web::resource("")
                    .route(web::get().to(handlers::list))
                    .route(web::post().to(handlers::upload)),
            )
            .route("/uploads", web::post().to(handlers::start_upload))
            .route("/{id}", web::get().to(handlers::metadata))
            .route("/{id}", web::delete().to(handlers::remove))
            .route("/{id}/complete", web::post().to(handlers::complete_upload))
            .route("/{id}/content", web::get().to(handlers::content))
            .route("/{id}/url", web::get().to(handlers::download_url)),
    );
    cfg.service(
        web::scope("/api/v1/storage")
            .app_data(path_config())
            .route("/{token}", web::put().to(handlers::put_object))
            .route("/{token}", web::get().to(handlers::get_object)),
    );
}
