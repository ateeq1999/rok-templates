// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The layers every answer passes through: the request's span (`traceparent`),
//! the security headers, CORS, and a panic turned into the wire standard's
//! `500`. What they send is `shared/security/headers.rs`'s; this file is
//! axum's glue around it. [`router`](super::router) applies them.

use std::any::Any;
use std::sync::Arc;

use axum::extract::{MatchedPath, Request, State};
use axum::http::{HeaderName, HeaderValue, Method};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::error::AppError;
use crate::shared::security::headers::{
    self, CORS_EXPOSED_HEADERS, CORS_HEADERS, CORS_MAX_AGE, CORS_METHODS, CorsOrigins,
};
use crate::shared::telemetry;

/// The security headers, ready to set: [`headers::security_headers`] for
/// `https`.
pub type SecurityHeaders = Arc<[(HeaderName, HeaderValue)]>;

/// [`headers::security_headers`] for `https`, as header names and values.
#[must_use]
pub fn security_headers(https: bool) -> SecurityHeaders {
    headers::security_headers(https)
        .into_iter()
        .filter_map(|(name, value)| {
            Some((
                HeaderName::from_static(name),
                HeaderValue::from_str(&value).ok()?,
            ))
        })
        .collect()
}

/// Sets the security headers on every answer
/// (`from_fn_with_state(security_headers(https), set_security_headers)`),
/// but those the handler set itself.
pub async fn set_security_headers(
    State(headers): State<SecurityHeaders>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    for (name, value) in headers.iter() {
        response
            .headers_mut()
            .entry(name.clone())
            .or_insert_with(|| value.clone());
    }
    response
}

/// CORS for `origins`: none but the API's own when unset; otherwise the
/// methods, headers and preflight cache of `shared/security/headers.rs`.
pub fn cors(origins: &CorsOrigins) -> CorsLayer {
    let layer = match origins {
        CorsOrigins::SameOrigin => return CorsLayer::new(),
        CorsOrigins::Any => CorsLayer::new().allow_origin(AllowOrigin::any()),
        CorsOrigins::List(origins) => CorsLayer::new().allow_origin(
            origins
                .iter()
                .filter_map(|origin| HeaderValue::from_str(origin).ok())
                .collect::<Vec<_>>(),
        ),
    };
    layer
        .allow_methods(
            CORS_METHODS
                .iter()
                .filter_map(|method| Method::from_bytes(method.as_bytes()).ok())
                .collect::<Vec<_>>(),
        )
        .allow_headers(names(CORS_HEADERS))
        .expose_headers(names(CORS_EXPOSED_HEADERS))
        .max_age(CORS_MAX_AGE)
}

/// `names` as header names.
fn names(names: &[&'static str]) -> Vec<HeaderName> {
    names
        .iter()
        .map(|name| HeaderName::from_static(name))
        .collect()
}

/// The span a request runs in (`TraceLayer::make_span_with(request_span)`):
/// its method, its route (the matched pattern, such as `/api/v1/users/{id}`,
/// so a trace backend groups by it), its `x-request-id`, and the caller's
/// `user_id`, which `shared/auth.rs` records once known. It continues the
/// trace the request's `traceparent` names, when spans are exported.
pub fn request_span(request: &Request) -> Span {
    let header = |name: &str| {
        request
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    };
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| request.uri().path(), MatchedPath::as_str);
    let span = tracing::info_span!(
        "request",
        method = %request.method(),
        route,
        request_id = header("x-request-id").unwrap_or_default(),
        user_id = tracing::field::Empty,
    );
    // Without an exporter there is no trace to join: nothing to set.
    let _ = span.set_parent(telemetry::remote_parent(
        header("traceparent"),
        header("tracestate"),
    ));
    span
}

/// A layer that answers a request whose handler panicked with the wire
/// standard's `500 internal_error`, and logs the panic, instead of
/// dropping the connection.
#[must_use]
pub fn catch_panic() -> CatchPanicLayer<fn(Box<dyn Any + Send + 'static>) -> Response> {
    CatchPanicLayer::custom(panicked as fn(Box<dyn Any + Send + 'static>) -> Response)
}

/// The answer to a panic carrying `payload`. It takes the payload by value
/// because that is what `CatchPanicLayer` hands over.
#[allow(clippy::needless_pass_by_value)]
fn panicked(payload: Box<dyn Any + Send + 'static>) -> Response {
    let message = payload
        .downcast_ref::<&str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "no message".to_owned());
    AppError::Internal(format!("a handler panicked: {message}").into()).into_response()
}
