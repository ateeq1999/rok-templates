//! Topics: a notice a feature publishes on its transaction reaches the
//! subscribers of its tenant once the transaction commits, and never when
//! it rolls back; a caller the subscribers' guard refuses gets a `403`.

mod support;

use std::future::poll_fn;
use std::pin::pin;
use std::time::Duration;

use actix_api::app::{AppState, app};
use actix_api::db::types::Role;
use actix_api::error::AppError;
use actix_api::features::live;
use actix_api::shared::auth::{AuthUser, Guard};
use actix_api::shared::live::publish;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::{StatusCode, header};
use actix_web::{App, test, web};
use serde_json::{Value, json};
use support::{Database, TestResult, account, state};
use uuid::Uuid;

/// `POST /api/v1/live/tickets` with `token`.
fn ticket(token: &str) -> test::TestRequest {
    test::TestRequest::post()
        .uri("/api/v1/live/tickets")
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
}

/// The status and JSON body of `response`.
async fn answer<B: MessageBody>(response: ServiceResponse<B>) -> (StatusCode, Value) {
    let status = response.status();
    let bytes = test::read_body(response).await;
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// The first notice on the stream `response` carries, as JSON.
async fn first_notice<B>(response: ServiceResponse<B>) -> Result<Value, Box<dyn std::error::Error>>
where
    B: MessageBody,
{
    let mut body = pin!(response.into_body());
    let mut text = String::new();
    loop {
        let chunk = tokio::time::timeout(
            Duration::from_secs(5),
            poll_fn(|cx| body.as_mut().poll_next(cx)),
        )
        .await?
        .ok_or("the stream ended")?
        .map_err(Into::into)?;
        text.push_str(&String::from_utf8_lossy(&chunk));
        if let Some(line) = text.lines().find_map(|line| line.strip_prefix("data:")) {
            return Ok(serde_json::from_str(line.trim())?);
        }
    }
}

#[actix_web::test]
async fn a_notice_reaches_its_tenants_subscriber_once_its_transaction_commits() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let live = state.live.clone();
    live.start(&db.pool).await?;
    let (id, token) = account(&db.pool, Role::User).await?;
    let app = test::init_service(app(state)).await;

    let (status, answer) =
        answer(test::call_service(&app, ticket(&token).to_request()).await).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let ticket = answer["ticket"].as_str().ok_or("no ticket")?;
    assert!(answer["expires_in"].as_u64().is_some(), "{answer}");
    let request = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/live?topics=orders.new,bills.paid&ticket={ticket}"
        ))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|value| value.as_bytes()),
        Some(&b"text/event-stream"[..])
    );

    // Another tenant's notice, a Topic not subscribed to, and a notice whose
    // transaction rolls back: none reaches the stream.
    let mut tx = db.pool.begin().await?;
    publish(&mut tx, Uuid::now_v7(), "orders.new", json!({"id": 1})).await?;
    publish(&mut tx, id, "orders.paid", json!({"id": 2})).await?;
    tx.commit().await?;
    let mut tx = db.pool.begin().await?;
    publish(&mut tx, id, "orders.new", json!({"id": 3})).await?;
    tx.rollback().await?;

    let mut tx = db.pool.begin().await?;
    publish(&mut tx, id, "bills.paid", json!({"id": 4})).await?;
    tx.commit().await?;

    let notice = first_notice(response).await?;
    assert_eq!(notice, json!({"topic": "bills.paid", "payload": {"id": 4}}));
    // The listener holds one of the pool's connections until the hub closes.
    live.close();
    db.drop().await
}

/// Lets nobody subscribe.
struct Nobody;

impl Guard for Nobody {
    type Grant = Uuid;
    type Path = ();

    async fn check(
        _caller: &AuthUser,
        _state: &AppState,
        _path: &(),
    ) -> Result<Option<Uuid>, AppError> {
        Ok(None)
    }
}

#[actix_web::test]
async fn a_caller_the_guard_refuses_gets_403_and_a_stream_needs_a_ticket() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let (_, token) = account(&db.pool, Role::User).await?;
    let refusing = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.pool.clone()))
            .app_data(state.keys.clone())
            .configure(live::routes::<Nobody>),
    )
    .await;
    let (status, answer) =
        answer(test::call_service(&refusing, ticket(&token).to_request()).await).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");

    let app = test::init_service(app(state)).await;
    for uri in [
        "/api/v1/live?topics=orders.new",
        "/api/v1/live?topics=orders.new&ticket=forged",
    ] {
        let request = test::TestRequest::get().uri(uri).to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
    db.drop().await
}
