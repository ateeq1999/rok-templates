//! `SignedWebhook<V>`: a provider's callback reaches its handler, headers
//! and raw body, only once the verifier `V` accepts its signature; a
//! tampered or unsigned one is a `401`. No database: the pool never
//! connects.

mod support;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode};
use axum::routing::post;
use axum_api::app::AppState;
use axum_api::shared::auth::{Delivery, SignedWebhook, WebhookSecret, WebhookVerifier};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Answer, TestResult, state_with};
use tower::ServiceExt;

/// The provider's secret, as its dashboard would show it.
const SECRET: &str = "whsec_c2VjcmV0LWZvci10aGUtdGVzdHMtb25seQ==";

/// The test provider's signature check.
struct Provider;

impl WebhookVerifier for Provider {
    fn verify(_state: &AppState, delivery: &Delivery<'_>) -> bool {
        WebhookSecret::new(SECRET).is_ok_and(|secret| secret.verify(delivery))
    }
}

/// Echoes what the handler was handed: a header and the body.
async fn callback(webhook: SignedWebhook<Provider>) -> axum::Json<Value> {
    axum::Json(json!({
        "id": webhook.headers.get("webhook-id").and_then(|id| id.to_str().ok()),
        "body": String::from_utf8_lossy(&webhook.body),
    }))
}

/// Posts `body` with `headers` to the callback route.
async fn deliver(
    headers: &[(&str, String)],
    body: &str,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let state = state_with(&PgPool::connect_lazy("postgres://localhost/unused")?, &[])?;
    let app = Router::new()
        .route("/callbacks/provider", post(callback))
        .with_state(state);
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/callbacks/provider");
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let response = app
        .oneshot(request.body(Body::from(body.to_owned()))?)
        .await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    Ok(Answer {
        status,
        body: serde_json::from_slice(&bytes)?,
    })
}

/// The provider's headers for `body`, sent now.
fn signed(body: &str) -> Result<Vec<(&'static str, String)>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().timestamp();
    Ok(WebhookSecret::new(SECRET)?
        .headers("msg_1", now, body.as_bytes())
        .to_vec())
}

#[tokio::test]
async fn a_correctly_signed_callback_reaches_its_handler() -> TestResult {
    let body = r#"{"type":"charge.settled"}"#;
    let answer = deliver(&signed(body)?, body).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({"id": "msg_1", "body": body}));
    Ok(())
}

#[tokio::test]
async fn a_tampered_or_unsigned_callback_is_refused() -> TestResult {
    let headers = signed(r#"{"amount":100}"#)?;
    let answer = deliver(&headers, r#"{"amount":900}"#).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    assert_eq!(answer.code(), "unauthorized");

    let answer = deliver(&[], r#"{"amount":100}"#).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    Ok(())
}
