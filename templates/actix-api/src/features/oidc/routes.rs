// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` routes, under `/api/v1/auth/oidc`, behind the `/auth` rate
//! limit.

use actix_web::web;

use super::handlers;
use crate::shared::rate_limit::RateLimit;

/// Mounts every `oidc` endpoint under `/api/v1/auth/oidc`, each request
/// counted against `limit` like the rest of `/auth`. `app/` mounts it
/// before the `auth` scope, so that scope doesn't swallow it.
pub fn routes(cfg: &mut web::ServiceConfig, limit: &RateLimit) {
    cfg.service(
        web::scope("/api/v1/auth/oidc")
            .wrap(limit.clone())
            .route("/identities", web::get().to(handlers::list_identities))
            .route(
                "/identities/{id}",
                web::delete().to(handlers::unlink_identity),
            )
            .route("/{provider}/start", web::get().to(handlers::start))
            .route("/{provider}/callback", web::get().to(handlers::callback))
            .route("/{provider}/link", web::post().to(handlers::link_identity)),
    );
}
