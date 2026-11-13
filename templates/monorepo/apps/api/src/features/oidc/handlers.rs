// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` handlers: each parses the request, calls one
//! [`OidcService`] method, and answers. One function per endpoint.
//!
//! `start` and `callback` answer only redirects (Browser routes), which the
//! released `rok`'s contract reader still refuses (an operation needs a
//! `2xx`), so they stay out of the `OpenAPI` document until the next `rok`
//! release reads them; then they join it with `#[utoipa::path]`.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};

use super::dto::{
    CallbackQuery, ClientQuery, HandoffDto, IdentityResponse, LinkResponse, SignInProviderResponse,
    TokenResponse,
};
use super::service::OidcService;
use crate::db::user_identities::models::UserIdentityId;
use crate::error::AppError;
use crate::shared::auth::AuthUser;
use crate::shared::response::ErrorBody;
use crate::shared::validation::ValidatedJson;

/// `GET /api/v1/auth/oidc/{provider}/start`: `303` to the provider's
/// sign-in page. `?client=mobile` comes back to the mobile App
/// (`<APP_SCHEME>://auth/oidc?…`) instead of the web App.
pub async fn start(
    State(service): State<Arc<OidcService>>,
    Path(provider): Path<String>,
    Query(query): Query<ClientQuery>,
) -> Result<Response, AppError> {
    let url = service
        .start(&provider, query.client.unwrap_or_default())
        .await?;
    Ok(Redirect::to(&url).into_response())
}

/// `GET /api/v1/auth/oidc/{provider}/callback`: the provider's redirect
/// back, `303` on to the App (`{APP_URL}/auth/oidc#…`).
pub async fn callback(
    State(service): State<Arc<OidcService>>,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> Result<Response, AppError> {
    let url = service.callback(&provider, query).await?;
    Ok(Redirect::to(&url).into_response())
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
    State(service): State<Arc<OidcService>>,
    caller: AuthUser,
) -> Result<Response, AppError> {
    let identities = service.identities(caller.id.into()).await?;
    Ok(Json(identities).into_response())
}

/// `POST /api/v1/auth/oidc/{provider}/link`: where to send the browser to
/// link an identity at `provider` to the caller, `200`.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oidc/{provider}/link",
    tag = "auth",
    operation_id = "link_identity",
    params(
        ("provider" = String, Path, description = "The provider's name, such as `google`"),
        ClientQuery,
    ),
    responses(
        (status = 200, description = "The provider's authorization URL; its callback links the identity", body = LinkResponse),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 404, description = "No such provider is configured", body = ErrorBody),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn link_identity(
    State(service): State<Arc<OidcService>>,
    caller: AuthUser,
    Path(provider): Path<String>,
    Query(query): Query<ClientQuery>,
) -> Result<Response, AppError> {
    let client = query.client.unwrap_or_default();
    let link = service.link(caller.id.into(), &provider, client).await?;
    Ok(Json(link).into_response())
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
    State(service): State<Arc<OidcService>>,
    caller: AuthUser,
    Path(id): Path<UserIdentityId>,
) -> Result<Response, AppError> {
    service.unlink(caller.id.into(), id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `GET /api/v1/auth/oidc/providers`: the providers people may sign in
/// with, for the Apps' "Continue with …" buttons, `200`. None configured
/// is an empty list, so the Apps hide the buttons.
#[utoipa::path(
    get,
    path = "/api/v1/auth/oidc/providers",
    tag = "auth",
    operation_id = "list_sign_in_providers",
    responses(
        (status = 200, description = "The configured providers, by name; empty when none is", body = Vec<SignInProviderResponse>),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
)]
pub async fn list_sign_in_providers(
    State(service): State<Arc<OidcService>>,
) -> Result<Response, AppError> {
    Ok(Json(service.sign_in_providers()).into_response())
}

/// `POST /api/v1/auth/oidc/handoff`: trades the code of the mobile App's
/// deep link (`<APP_SCHEME>://auth/oidc?code=…`) for a session, `200`.
#[utoipa::path(
    post,
    path = "/api/v1/auth/oidc/handoff",
    tag = "auth",
    operation_id = "exchange_handoff",
    request_body = HandoffDto,
    responses(
        (status = 200, description = "A session", body = TokenResponse),
        (status = 400, description = "The code expired, was already used, or never existed", body = ErrorBody),
        (status = 404, description = "No OpenID provider is configured", body = ErrorBody),
        (status = 422, description = "No code", body = ErrorBody),
        (status = 429, description = "Too many requests from this IP", body = ErrorBody),
    ),
)]
pub async fn exchange_handoff(
    State(service): State<Arc<OidcService>>,
    body: ValidatedJson<HandoffDto>,
) -> Result<Response, AppError> {
    let session = service.exchange_handoff(&body.into_inner().code).await?;
    Ok(Json(session).into_response())
}
