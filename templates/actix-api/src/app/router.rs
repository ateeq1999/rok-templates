// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`app`]: the whole application over an [`AppState`], its services and
//! routes ([`configure`]) wrapped in the middleware.

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::from_fn;
use actix_web::{App, web};
use tracing_actix_web::TracingLogger;
use utoipa_swagger_ui::SwaggerUi;

use super::AppState;
use super::middleware::{
    GlobalRateLimit, RequestSpan, catch_panic, cors, global_rate_limit, request_id,
    security_headers,
};
use crate::features::account::{self, AccountService};
use crate::features::api_keys::{self, ApiKeyService};
use crate::features::auth::{self, AuthService};
use crate::features::two_factor::{self, TwoFactorService};
use crate::features::{configure_all, media, register_services};
use crate::openapi::document;
use crate::shared::health;

/// The whole application over `state`: [`configure`]'s services and
/// routes, wrapped (outermost first) in the request's span and its
/// `x-request-id`, the security headers, CORS, a
/// panic answered `500`, and everything but health under
/// `state.global_rate_limit`. The server runs it in every worker; tests
/// build it with `actix_web::test::init_service(app(state))`.
pub fn app(
    state: AppState,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(GlobalRateLimit(
            state.global_rate_limit.clone(),
        )))
        .configure(|cfg| configure(cfg, &state))
        .wrap(from_fn(global_rate_limit))
        .wrap(from_fn(catch_panic))
        .wrap(cors(&state.cors_allowed_origins))
        .wrap(security_headers(state.https))
        .wrap(from_fn(request_id))
        .wrap(TracingLogger::<RequestSpan>::new())
}

/// Mounts the whole application on `cfg`: the pool, the JWT keys, the mail
/// outbox and templates as app data, every feature's service,
/// `/health/live` and `/health/ready`, the `OpenAPI` document at
/// `/api-docs/openapi.json` with Swagger UI at `/docs/`, and every feature's
/// routes: the hand-written `account`, `two_factor` and `auth` first, so
/// that `/users/me` is not taken for `/users/{id}` and the `/auth` scope
/// doesn't swallow `/auth/me` or `/auth/2fa`. [`app`] wraps it in the
/// middleware.
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
