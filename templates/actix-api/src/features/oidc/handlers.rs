// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` handlers: each parses the request, calls one
//! [`OidcService`] method, and answers. One function per endpoint.
//!
//! `start` and `callback` answer only redirects (Browser routes), which the
//! released `rok`'s contract reader still refuses (an operation needs a
//! `2xx`), so they stay out of the `OpenAPI` document until the next `rok`
//! release reads them; then they join it with `#[utoipa::path]`.

use actix_web::http::header;
use actix_web::{HttpResponse, web};

use super::dto::{CallbackQuery, IdentityResponse, LinkResponse};
use super::service::OidcService;
use crate::db::user_identities::models::UserIdentityId;
use crate::error::AppError;
use crate::shared::auth::AuthUser;
use crate::shared::response::ErrorBody;

/// `GET /api/v1/auth/oidc/{provider}/start`: `303` to the provider's
/// sign-in page.
pub async fn start(
    service: web::Data<OidcService>,
    provider: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let url = service.start(&provider).await?;
    Ok(see_other(&url))
}

/// `GET /api/v1/auth/oidc/{provider}/callback`: the provider's redirect
/// back, `303` on to the App (`{APP_URL}/auth/oidc#…`).
pub async fn callback(
    service: web::Data<OidcService>,
    provider: web::Path<String>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse, AppError> {
    let url = service.callback(&provider, query.into_inner()).await?;
    Ok(see_other(&url))
}

/// A `303 See Other` to `url`.
fn see_other(url: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, url))
        .finish()
}

/// `GET /api/v1/auth/oidc/identities`: the caller's Linked identities,
/// `200`.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oidc/identities",
    tag = "auth",
    operation_id = "list_identities",
    responses(
        (status = 200, description = "The caller's Linked identities, oldest first", body = Vec<IdentityResponse>),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No OpenID provider is configured", body = ErrorBody),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list_identities(
    service: web::Data<OidcService>,
    caller: AuthUser,
) -> Result<HttpResponse, AppError> {
    let identities = service.identities(caller.id.into()).await?;
    Ok(HttpResponse::Ok().json(identities))
}

/// `POST /api/v1/auth/oidc/{provider}/link`: where to send the browser to
/// link an identity at `provider` to the caller, `200`.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oidc/{provider}/link",
    tag = "auth",
    operation_id = "link_identity",
    params(("provider" = String, Path, description = "The provider's name, such as `google`")),
    responses(
        (status = 200, description = "The provider's authorization URL; its callback links the identity", body = LinkResponse),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such provider is configured", body = ErrorBody),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn link_identity(
    service: web::Data<OidcService>,
    caller: AuthUser,
    provider: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let link = service.link(caller.id.into(), &provider).await?;
    Ok(HttpResponse::Ok().json(link))
}

/// `DELETE /api/v1/auth/oidc/identities/{id}`: unlinks one of the caller's
/// identities, `204`.
#[utoipa::path(
    delete,
    path = "/api/v1/auth/oidc/identities/{id}",
    tag = "auth",
    operation_id = "unlink_identity",
    params(("id" = UserIdentityId, Path, description = "The Linked identity")),
    responses(
        (status = 204, description = "Unlinked"),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such identity of the caller's, or no provider configured", body = ErrorBody),
        (status = 422, description = "The account's last way in: no password and no other identity", body = ErrorBody),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn unlink_identity(
    service: web::Data<OidcService>,
    caller: AuthUser,
    path: web::Path<UserIdentityId>,
) -> Result<HttpResponse, AppError> {
    service.unlink(caller.id.into(), path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
