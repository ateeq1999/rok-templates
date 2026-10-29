// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `jobs` routes: `/api/v1/jobs`.

use actix_web::web;

use super::handlers;

/// Mounts the job queue, for admins (`AdminUser` on every handler).
pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/v1/jobs").route(web::get().to(handlers::list)))
        .service(web::resource("/api/v1/jobs/{id}").route(web::delete().to(handlers::cancel)))
        .service(web::resource("/api/v1/jobs/{id}/retry").route(web::post().to(handlers::retry)));
}
