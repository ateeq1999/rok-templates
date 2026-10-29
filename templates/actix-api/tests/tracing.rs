//! Every request runs in a span of its own and is answered its id, so a
//! caller can quote `x-request-id` and an operator can find its logs.

mod support;

use actix_web::http::{Method, StatusCode};
use sqlx::PgPool;
use support::{TestResult, send, state_with};

#[actix_web::test]
async fn every_answer_carries_its_request_id() -> TestResult {
    let state = state_with(&PgPool::connect_lazy("postgres://localhost/unused")?, &[])?;
    let first = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
    let second = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
    assert_eq!(first.status, StatusCode::OK);
    let id = |answer: &support::Raw| answer.header("x-request-id").map(str::to_owned);
    let (first, second) = (id(&first), id(&second));
    assert!(
        first.as_deref().is_some_and(|id| id.len() == 36),
        "{first:?}"
    );
    assert_ne!(first, second, "two requests share an id");
    Ok(())
}
