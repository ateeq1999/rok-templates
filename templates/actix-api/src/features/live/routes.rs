// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` routes: `/api/v1/live`.

use std::fmt::Display;

use actix_web::web;

use super::handlers;
use crate::shared::auth::Guard;

/// Mounts tickets for the callers `G` lets in, its grant their tenant, and
/// the stream a ticket opens. `app/` mounts `routes::<Subscriber>`.
pub fn routes<G>(cfg: &mut web::ServiceConfig)
where
    G: Guard<Path = ()> + 'static,
    G::Grant: Display,
{
    cfg.service(web::resource("/api/v1/live").route(web::get().to(handlers::subscribe)))
        .service(
            web::resource("/api/v1/live/tickets").route(web::post().to(handlers::ticket::<G>)),
        );
}
