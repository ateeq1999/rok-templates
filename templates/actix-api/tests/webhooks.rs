//! `SignedWebhook<V>`: a provider's callback reaches its handler, headers
//! and raw body, only once the verifier `V` accepts its signature; a
//! tampered or unsigned one is a `401`. No database: the pool never
//! connects.

mod support;

use actix_api::app::AppState;
use actix_api::shared::auth::{Delivery, SignedWebhook, WebhookSecret, WebhookVerifier};
use actix_web::http::StatusCode;
use actix_web::{App, HttpResponse, test, web};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Answer, TestResult, state_with};

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
async fn callback(webhook: SignedWebhook<Provider>) -> HttpResponse {
    HttpResponse::Ok().json(json!({
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
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .route("/callbacks/provider", web::post().to(callback)),
    )
    .await;
    let mut request = test::TestRequest::post()
        .uri("/callbacks/provider")
        .set_payload(body.to_owned());
    for (name, value) in headers {
        request = request.insert_header((*name, value.as_str()));
    }
    let response = test::call_service(&app, request.to_request()).await;
    let status = response.status();
    let bytes = test::read_body(response).await;
    let body: Value = serde_json::from_slice(&bytes)?;
    Ok(Answer { status, body })
}

/// The provider's headers for `body`, sent now.
fn signed(body: &str) -> Result<Vec<(&'static str, String)>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().timestamp();
    Ok(WebhookSecret::new(SECRET)?
        .headers("msg_1", now, body.as_bytes())
        .to_vec())
}

#[actix_web::test]
async fn a_correctly_signed_callback_reaches_its_handler() -> TestResult {
    let body = r#"{"type":"charge.settled"}"#;
    let answer = deliver(&signed(body)?, body).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({"id": "msg_1", "body": body}));
    Ok(())
}

#[actix_web::test]
async fn a_tampered_or_unsigned_callback_is_refused() -> TestResult {
    let headers = signed(r#"{"amount":100}"#)?;
    let answer = deliver(&headers, r#"{"amount":900}"#).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    assert_eq!(answer.code(), "unauthorized");

    let answer = deliver(&[], r#"{"amount":100}"#).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    Ok(())
}
