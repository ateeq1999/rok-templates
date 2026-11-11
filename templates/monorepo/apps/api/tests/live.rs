//! Topics: a notice a feature publishes on its transaction reaches the
//! subscribers of its tenant once the transaction commits, and never when
//! it rolls back; a caller the subscribers' guard refuses gets a `403`.

mod support;

use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Extension;
use axum::http::{Method, Request, StatusCode, header};
use axum_api::app::{AppState, router};
use axum_api::db::types::Role;
use axum_api::error::AppError;
use axum_api::features::live;
use axum_api::shared::auth::{AuthUser, Guard};
use axum_api::shared::live::publish;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{Database, TestResult, account, state};
use tower::ServiceExt;
use uuid::Uuid;

/// A ticket for the caller of `token`, from `POST /api/v1/live/tickets`.
async fn ticket(
    app: Router,
    token: &str,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/live/tickets")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    Ok((
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    ))
}

/// The first notice on the stream `GET uri` answers, as JSON.
async fn first_notice(app: Router, uri: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|value| value.as_bytes()),
        Some(&b"text/event-stream"[..])
    );
    let mut body = response.into_body();
    let mut text = String::new();
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(5), body.frame())
            .await?
            .ok_or("the stream ended")??;
        if let Ok(data) = frame.into_data() {
            text.push_str(&String::from_utf8_lossy(&data));
        }
        if let Some(line) = text.lines().find_map(|line| line.strip_prefix("data:")) {
            return Ok(serde_json::from_str(line.trim())?);
        }
    }
}

#[tokio::test]
async fn a_notice_reaches_its_tenants_subscriber_once_its_transaction_commits() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let live = state.live.clone();
    live.start(&db.pool).await?;
    let (id, token) = account(&db.pool, Role::User).await?;
    let app = router(state);

    let (status, answer) = ticket(app.clone(), &token).await?;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let ticket = answer["ticket"].as_str().ok_or("no ticket")?;
    assert!(answer["expires_in"].as_u64().is_some(), "{answer}");
    let uri = format!("/api/v1/live?topics=orders.new,bills.paid&ticket={ticket}");
    let stream =
        tokio::spawn(async move { first_notice(app, &uri).await.map_err(|err| err.to_string()) });
    // Let the subscriber in before anything is published.
    tokio::time::sleep(Duration::from_millis(300)).await;

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

    let notice = stream.await?.map_err(|err| format!("the stream: {err}"))?;
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

#[tokio::test]
async fn a_caller_the_guard_refuses_gets_403_and_a_stream_needs_a_ticket() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let (_, token) = account(&db.pool, Role::User).await?;
    let refusing = live::routes::<Nobody>()
        .with_state(state.clone())
        .layer(Extension(state.pool.clone()))
        .layer(Extension(state.keys.clone()));
    let (status, answer) = ticket(refusing, &token).await?;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");

    let app = router(state);
    for uri in [
        "/api/v1/live?topics=orders.new",
        "/api/v1/live?topics=orders.new&ticket=forged",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{uri}");
    }
    db.drop().await
}
