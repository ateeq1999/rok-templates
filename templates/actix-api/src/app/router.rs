// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`configure`]: the whole application's services and routes, mounted
//! over an [`AppState`].

use actix_web::web;
use utoipa_swagger_ui::SwaggerUi;

use super::AppState;
use crate::features::account::{self, AccountService};
use crate::features::api_keys::{self, ApiKeyService};
use crate::features::auth::{self, AuthService};
use crate::features::two_factor::{self, TwoFactorService};
use crate::features::{configure_all, media, register_services};
use crate::openapi::document;
use crate::shared::health;

/// Mounts the whole application on `cfg`: the pool, the JWT keys, the mail
/// outbox and templates as app data, every feature's service,
/// `/health/live` and `/health/ready`, the `OpenAPI` document at
/// `/api-docs/openapi.json` with Swagger UI at `/docs/`, and every feature's
/// routes: the hand-written `account`, `two_factor` and `auth` first, so
/// that `/users/me` is not taken for `/users/{id}` and the `/auth` scope
/// doesn't swallow `/auth/me` or `/auth/2fa`. Tests build the same
/// application with `actix_web::test::init_service(App::new().configure(...))`.
pub fn configure(cfg: &mut web::ServiceConfig, state: &AppState) {
    cfg.app_data(web::Data::new(state.pool.clone()))
        .app_data(state.keys.clone())
        .app_data(state.outbox.clone())
        .app_data(state.templates.clone());
    register_services(cfg, &state.pool);
    cfg.app_data(web::Data::new(AccountService::new(
        state.pool.clone(),
        state.storage.clone(),
    )))
    .app_data(web::Data::new(AuthService {
        pool: state.pool.clone(),
        keys: state.keys.clone().into_inner(),
        outbox: state.outbox.clone().into_inner(),
        templates: state.templates.clone().into_inner(),
        breach: state.breach.clone(),
        require_verified_email: state.require_verified_email,
        audit: state.audit.clone(),
    }))
    .app_data(web::Data::new(TwoFactorService {
        pool: state.pool.clone(),
        keys: state.keys.clone().into_inner(),
        outbox: state.outbox.clone().into_inner(),
        templates: state.templates.clone().into_inner(),
        totp: state.totp.clone(),
    }));
    cfg.app_data(web::Data::new(state.media_service()));
    health::routes(cfg);
    cfg.service(web::redirect("/docs", "/docs/").see_other())
        .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", document()));
    cfg.app_data(web::Data::new(ApiKeyService::new(state.pool.clone())));
    account::routes(cfg, &state.rate_limit);
    api_keys::routes(cfg);
    two_factor::routes(cfg, &state.rate_limit);
    auth::routes(cfg, &state.rate_limit);
    media::routes(cfg);
    configure_all(cfg);
}
