//! Draining: once the process is shutting down, `/health/ready` answers
//! `503` (the load balancer stops sending requests) while `/health/live`
//! and every other route still answer. The pool never connects.

mod support;

use actix_web::http::{Method, StatusCode};
use sqlx::PgPool;
use support::{TestResult, send, state_with};

#[actix_web::test]
async fn a_draining_process_is_unready_but_still_serves() -> TestResult {
    let state = state_with(&PgPool::connect_lazy("postgres://localhost/unused")?, &[])?;
    state.draining.start();
    let ready = send(&state, None, Method::GET, "/health/ready", &[], Vec::new()).await?;
    assert_eq!(ready.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(ready.json()?["error"]["code"], "service_unavailable");
    let live = send(&state, None, Method::GET, "/health/live", &[], Vec::new()).await?;
    assert_eq!(live.status, StatusCode::OK);
    let docs = send(
        &state,
        None,
        Method::GET,
        "/api-docs/openapi.json",
        &[],
        Vec::new(),
    )
    .await?;
    assert_eq!(docs.status, StatusCode::OK);
    Ok(())
}
