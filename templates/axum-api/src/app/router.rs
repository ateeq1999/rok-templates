// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`router`]: the whole application over an [`AppState`], with its
//! middleware.

use std::time::Duration;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

use super::AppState;
use crate::error::AppError;
use crate::features;
use crate::features::{account, api_keys, auth, media, two_factor};
use crate::openapi::document;
use crate::shared::health;

/// The largest request body: 2 MiB. A JSON body over it is a `413`.
const BODY_LIMIT: usize = 2 * 1024 * 1024;

/// How long a request may take before it is answered `408`.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// The whole application over `state`: `/health/live` and
/// `/health/ready`, the `OpenAPI` document at `/api-docs/openapi.json` with
/// Swagger UI at `/docs/`, every feature's routes (the hand-written
/// `account`, `auth` and `two_factor`, their `/auth` routes behind
/// `state.rate_limit`), and the middleware: tracing, request ids
/// (`x-request-id`), CORS, a timeout and a body limit. Tests drive the same router.
///
/// The generated features' routes (`features::router`) are stateless: they
/// hold their services as extensions. The hand-written ones are a
/// `Router<AppState>`, given `state` here. Either kind reads the caller
/// (`AuthUser`) from the JWT keys and the pool [`router`] puts in every
/// request's extensions.
///
/// Mount a hand-written feature by merging its `routes()` into `hand_written`
/// below; wrap the routes that need it in `state.rate_limit.clone()` (a
/// `Layer`).
pub fn router(state: AppState) -> Router {
    let pool = state.pool.clone();
    let keys = state.keys.clone();
    let hand_written: Router = Router::new()
        .merge(account::routes(&state.rate_limit))
        .merge(auth::routes(&state.rate_limit))
        .merge(api_keys::routes())
        .merge(two_factor::routes(&state.rate_limit))
        .merge(media::routes(state.media.max_upload_bytes))
        .with_state(state);
    let extensions = pool.clone();
    Router::new()
        .merge(health::routes(&pool))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", document()))
        .merge(hand_written)
        .merge(features::router(&pool))
        .layer(middleware::from_fn(
            move |mut request: Request, next: Next| {
                request.extensions_mut().insert(keys.clone());
                request.extensions_mut().insert(extensions.clone());
                next.run(request)
            },
        ))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware::from_fn_with_state(REQUEST_TIMEOUT, time_out))
        // Same-origin only: allow your front end's origin here.
        .layer(CorsLayer::new())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

/// The request timeout [`router`] applies (`from_fn_with_state(limit,
/// time_out)`): a request still running after `limit` is dropped and
/// answered `408 request_timeout` with the wire standard's body.
pub async fn time_out(State(limit): State<Duration>, request: Request, next: Next) -> Response {
    match tokio::time::timeout(limit, next.run(request)).await {
        Ok(response) => response,
        Err(_) => AppError::RequestTimeout(format!("the request took longer than {limit:?}"))
            .into_response(),
    }
}
