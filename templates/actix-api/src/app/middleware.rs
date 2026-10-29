// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The middleware every answer passes through: the request's span and id,
//! the security headers, CORS,
//! a panic turned into the wire standard's `500`, and `GLOBAL_RATE_LIMIT`
//! on every route but health. What they send is
//! `shared/security/headers.rs`'s; this file is actix-web's glue around it.
//! [`app`](super::app) wraps the application in them.

use std::panic::AssertUnwindSafe;
use std::time::Instant;

use actix_cors::Cors;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::middleware::{DefaultHeaders, Next};
use actix_web::{Error, HttpMessage, ResponseError, web};
use futures_util::FutureExt;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RequestId, RootSpanBuilder};

use crate::error::AppError;
use crate::shared::metrics;
use crate::shared::rate_limit::{FORWARDED_FOR, RateLimit};
use crate::shared::security::headers::{
    self, CORS_EXPOSED_HEADERS, CORS_HEADERS, CORS_MAX_AGE, CORS_METHODS, CorsOrigins,
};

/// The security headers for `https`, set on every answer that doesn't
/// carry them already.
pub fn security_headers(https: bool) -> DefaultHeaders {
    headers::security_headers(https)
        .into_iter()
        .fold(DefaultHeaders::new(), |defaults, header| {
            defaults.add(header)
        })
}

/// CORS for `origins`: none but the API's own when unset; otherwise the
/// methods, headers and preflight cache of `shared/security/headers.rs`.
/// A request from another origin is never refused here: it just gets no
/// `Access-Control-Allow-Origin`, and the browser keeps the answer from
/// the page.
pub fn cors(origins: &CorsOrigins) -> Cors {
    let cors = Cors::default().block_on_origin_mismatch(false);
    let cors = match origins {
        CorsOrigins::SameOrigin => return cors,
        CorsOrigins::Any => cors.allow_any_origin(),
        CorsOrigins::List(origins) => origins
            .iter()
            .fold(cors, |cors, origin| cors.allowed_origin(origin)),
    };
    cors.allowed_methods(CORS_METHODS.iter().copied())
        .allowed_headers(CORS_HEADERS.iter().copied())
        .expose_headers(CORS_EXPOSED_HEADERS.iter().copied())
        .max_age(usize::try_from(CORS_MAX_AGE.as_secs()).ok())
}

/// The span every request runs in (`TracingLogger::<RequestSpan>`):
/// tracing-actix-web's root span (method, route, status, a `request_id`,
/// and the trace the request's `traceparent` names, when spans are
/// exported) with the caller's `user_id`, which `shared/auth.rs` records
/// once known.
pub struct RequestSpan;

impl RootSpanBuilder for RequestSpan {
    fn on_request_start(request: &ServiceRequest) -> Span {
        tracing_actix_web::root_span!(request, user_id = tracing::field::Empty)
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

/// Answers the request's id as `x-request-id`, the id its span carries, so
/// a caller can quote it (`.wrap(from_fn(request_id))`, inside the
/// `TracingLogger` that assigns it).
pub async fn request_id(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let id = req.extensions().get::<RequestId>().copied();
    let mut response = next.call(req).await?;
    if let Some(value) = id.and_then(|id| HeaderValue::from_str(&id.to_string()).ok()) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-request-id"), value);
    }
    Ok(response)
}

/// Counts and times every answered request by method, route and status
/// (`.wrap(from_fn(record_metrics))`), a failed one by its error's status;
/// a no-op until `METRICS_ADDR` installs the recorder. A path no route
/// matched counts as one `unmatched` route.
pub async fn record_metrics(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let method = req.method().to_string();
    let started = Instant::now();
    let outcome = next.call(req).await;
    let (route, status) = match &outcome {
        Ok(response) => (response.request().match_pattern(), response.status()),
        Err(error) => (None, error.as_response_error().status_code()),
    };
    metrics::record_request(
        &method,
        route.as_deref().unwrap_or(metrics::UNMATCHED_ROUTE),
        status.as_u16(),
        started.elapsed(),
    );
    outcome
}

/// Fails a request whose handler panicked with `AppError::Internal`, which
/// actix-web answers as the wire standard's `500 internal_error` (and logs),
/// instead of dropping the connection (`.wrap(from_fn(catch_panic))`).
///
/// It fails the call rather than building the answer: that needs the
/// request, and holding a copy of it while it is routed panics actix-web.
pub async fn catch_panic(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    match AssertUnwindSafe(next.call(req)).catch_unwind().await {
        Ok(response) => response.map(ServiceResponse::map_into_boxed_body),
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "no message".to_owned());
            Err(AppError::Internal(format!("a handler panicked: {message}").into()).into())
        }
    }
}

/// `GLOBAL_RATE_LIMIT`, as app data [`global_rate_limit`] reads: apart from
/// the `/auth` limit, which counts on its own.
pub struct GlobalRateLimit(pub RateLimit);

/// Counts every request but health against the [`GlobalRateLimit`] in the
/// app data, and answers the wire standard's `429` past it
/// (`.wrap(from_fn(global_rate_limit))`). A probe is never refused.
pub async fn global_rate_limit(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let checked = match req.app_data::<web::Data<GlobalRateLimit>>() {
        Some(limit) if !req.path().starts_with("/health/") => {
            let forwarded_for = req
                .headers()
                .get_all(FORWARDED_FOR)
                .filter_map(|value| value.to_str().ok());
            limit
                .0
                .check(forwarded_for, || req.peer_addr().map(|addr| addr.ip()))
        }
        _ => Ok(()),
    };
    match checked {
        Ok(()) => next
            .call(req)
            .await
            .map(ServiceResponse::map_into_boxed_body),
        Err(refused) => Ok(req.into_response(refused.error_response())),
    }
}
