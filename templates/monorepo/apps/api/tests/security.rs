//! The security baseline every answer gets: the security headers (HSTS
//! over HTTPS only), CORS from `CORS_ALLOWED_ORIGINS`, a panic answered
//! `500`, and `GLOBAL_RATE_LIMIT` on every route but health. No database:
//! the pool never connects.

mod support;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use axum::routing::get;
use serde_json::Value;
use sqlx::PgPool;
use support::{TestResult, send, state_with};
use tower::ServiceExt;

use axum_api::app::middleware::catch_panic;

fn pool() -> Result<PgPool, sqlx::Error> {
    PgPool::connect_lazy("postgres://localhost/unused")
}

#[tokio::test]
async fn every_answer_carries_the_security_headers() -> TestResult {
    let state = state_with(&pool()?, &[])?;
    let answer = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.header("x-content-type-options"), Some("nosniff"));
    assert_eq!(answer.header("x-frame-options"), Some("DENY"));
    assert_eq!(answer.header("referrer-policy"), Some("no-referrer"));
    assert_eq!(answer.header("strict-transport-security"), None);
    // An error answer too, and HSTS once `APP_URL` is HTTPS.
    let state = state_with(&pool()?, &[("APP_URL", "https://api.example.com")])?;
    let answer = send(&state, None, Method::GET, "/no/such/route", &[], Vec::new()).await?;
    assert_eq!(answer.header("x-content-type-options"), Some("nosniff"));
    assert!(
        answer
            .header("strict-transport-security")
            .is_some_and(|value| value.starts_with("max-age="))
    );
    Ok(())
}

/// A preflight from an allowed origin is answered for it; another origin
/// gets no `Access-Control-Allow-Origin`. Unset, no other origin is.
#[tokio::test]
async fn cors_allows_the_configured_origins_only() -> TestResult {
    let preflight = |origin: &'static str| {
        [
            ("origin", origin),
            ("access-control-request-method", "POST"),
            (
                "access-control-request-headers",
                "authorization,content-type",
            ),
        ]
    };
    let state = state_with(
        &pool()?,
        &[("CORS_ALLOWED_ORIGINS", "https://app.example.com")],
    )?;
    let path = "/api/v1/auth/login";
    let allowed = send(
        &state,
        None,
        Method::OPTIONS,
        path,
        &preflight("https://app.example.com"),
        Vec::new(),
    )
    .await?;
    assert_eq!(
        allowed.header("access-control-allow-origin"),
        Some("https://app.example.com")
    );
    let refused = send(
        &state,
        None,
        Method::OPTIONS,
        path,
        &preflight("https://evil.example.com"),
        Vec::new(),
    )
    .await?;
    assert_eq!(refused.header("access-control-allow-origin"), None);
    let same_origin = state_with(&pool()?, &[])?;
    let unset = send(
        &same_origin,
        None,
        Method::OPTIONS,
        path,
        &preflight("https://app.example.com"),
        Vec::new(),
    )
    .await?;
    assert_eq!(unset.header("access-control-allow-origin"), None);
    Ok(())
}

/// A handler that panics is answered the wire standard's `500`, not a
/// dropped connection.
#[tokio::test]
async fn a_panic_is_answered_500() -> TestResult {
    #[allow(clippy::panic)]
    async fn boom() -> &'static str {
        panic!("the handler broke")
    }
    let router = Router::new().route("/boom", get(boom)).layer(catch_panic());
    let response = router
        .oneshot(Request::get("/boom").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(body["error"]["code"], "internal_error");
    assert!(!body.to_string().contains("the handler broke"), "{body}");
    Ok(())
}

/// Past `GLOBAL_RATE_LIMIT`, any route but health answers `429`; health
/// never does.
#[tokio::test]
async fn the_global_rate_limit_spares_health() -> TestResult {
    let state = state_with(&pool()?, &[("GLOBAL_RATE_LIMIT", "1")])?;
    let docs = "/api-docs/openapi.json";
    let first = send(&state, None, Method::GET, docs, &[], Vec::new()).await?;
    assert_eq!(first.status, StatusCode::OK);
    let second = send(&state, None, Method::GET, docs, &[], Vec::new()).await?;
    assert_eq!(second.status, StatusCode::TOO_MANY_REQUESTS);
    assert!(second.header("retry-after").is_some());
    for _ in 0..3 {
        let live = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
        assert_eq!(live.status, StatusCode::OK);
    }
    Ok(())
}
