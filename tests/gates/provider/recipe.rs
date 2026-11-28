//! The provider recipe at work: a charge made through the fake swapped
//! into `AppState`, its signed callback let in by
//! `SignedWebhook<PaymentsSignature>`, and a forged one refused. No
//! database: the pool never connects.

mod support;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::{Method, Request, StatusCode};
use axum::routing::post;
use axum_api::app::AppState;
use axum_api::error::AppError;
use axum_api::shared::auth::SignedWebhook;
use axum_api::shared::payments::{Charge, FakePayments, PaymentEvent, PaymentsSignature};
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Answer, TestResult, config, state_with};
use tower::ServiceExt;

pub async fn payment_callback(
    State(_state): State<AppState>,
    webhook: SignedWebhook<PaymentsSignature>,
) -> Result<Json<Value>, AppError> {
    let event: PaymentEvent = serde_json::from_slice(&webhook.body)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;
    // Mark the order paid here, through the feature's service.
    Ok(Json(json!({ "settled": event.charge_id })))
}

/// A state charging through `fake`, as a test sets it up.
fn state(fake: &Arc<FakePayments>) -> Result<AppState, Box<dyn std::error::Error>> {
    let pool = PgPool::connect_lazy("postgres://localhost/unused")?;
    Ok(state_with(&pool, &[])?.with_payments(fake.clone()))
}

/// Posts `body` with `headers` to the callback route over `state`.
async fn call_back(
    state: &AppState,
    headers: &[(&str, String)],
    body: &str,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/payments/callback", post(payment_callback))
        .with_state(state.clone());
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/payments/callback");
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

#[tokio::test]
async fn the_fakes_signed_callback_is_let_in_and_a_forged_one_is_not() -> TestResult {
    let fake = Arc::new(FakePayments::default());
    let state = state(&fake)?;
    let charge = Charge {
        amount: 1_500,
        currency: "usd".to_owned(),
        reference: "order-1".to_owned(),
    };
    let id = state.payments.charge(&charge).await?;
    assert_eq!(fake.charges(), [charge]);

    let (headers, body) = fake.settled(&id);
    let answer = call_back(&state, &headers, &body).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({ "settled": id }));

    // The same signature over another body, and another provider's.
    let forged = body.replace(&id, "ch_someone_elses");
    let answer = call_back(&state, &headers, &forged).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    let (headers, body) = FakePayments::default().settled(&id);
    let answer = call_back(&state, &headers, &body).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    Ok(())
}

#[test]
fn payments_transport_picks_the_transport() {
    assert!(config(&[]).is_ok(), "the fake is the default");
    assert!(
        config(&[("PAYMENTS_TRANSPORT", "http")]).is_err(),
        "http needs its URL, key and secret"
    );
    assert!(
        config(&[
            ("PAYMENTS_TRANSPORT", "http"),
            ("PAYMENTS_URL", "https://payments.example.com"),
            ("PAYMENTS_API_KEY", "key"),
            ("PAYMENTS_WEBHOOK_SECRET", "whsec_c2VjcmV0"),
        ])
        .is_ok()
    );
    assert!(config(&[("PAYMENTS_TRANSPORT", "carrier-pigeon")]).is_err());
}
