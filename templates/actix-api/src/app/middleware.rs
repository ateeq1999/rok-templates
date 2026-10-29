// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The middleware every answer passes through: the security headers, CORS,
//! a panic turned into the wire standard's `500`, and `GLOBAL_RATE_LIMIT`
//! on every route but health. What they send is
//! `shared/security/headers.rs`'s; this file is actix-web's glue around it.
//! [`app`](super::app) wraps the application in them.

use std::panic::AssertUnwindSafe;

use actix_cors::Cors;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::{DefaultHeaders, Next};
use actix_web::{Error, ResponseError, web};
use futures_util::FutureExt;

use crate::error::AppError;
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
