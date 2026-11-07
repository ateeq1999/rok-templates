//! `Allowed<G>`: a handler requiring a guard gets its grant, and a caller
//! the guard refuses is answered `403` before the body is read, so a
//! refused caller never learns the body's rules from a `422`.

mod support;

use axum::body::{Body, to_bytes};
use axum::extract::Extension;
use axum::http::{Method, Request, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use axum_api::app::AppState;
use axum_api::db::types::Role;
use axum_api::error::AppError;
use axum_api::shared::auth::{Allowed, AuthUser, Guard};
use serde::Deserialize;
use serde_json::{Value, json};
use support::{Answer, Database, TestResult, account, state};
use tower::ServiceExt;
use uuid::Uuid;

/// Lets a caller in at their own `{id}` only, granting it.
struct Own;

/// The path [`Own`] reads.
#[derive(Deserialize)]
struct ThingPath {
    id: Uuid,
}

impl Guard for Own {
    type Grant = Uuid;
    type Path = ThingPath;

    async fn check(
        caller: &AuthUser,
        _state: &AppState,
        path: &ThingPath,
    ) -> Result<Option<Uuid>, AppError> {
        Ok((caller.id == path.id).then_some(path.id))
    }
}

/// Lets admins in: a yes or no, with its own refusal.
struct Admins;

impl Guard for Admins {
    type Grant = ();
    type Path = ();
    const REFUSAL: &'static str = "this is for admins";

    async fn check(
        caller: &AuthUser,
        _state: &AppState,
        _path: &(),
    ) -> Result<Option<()>, AppError> {
        Ok((caller.role == Role::Admin).then_some(()))
    }
}

/// The body both routes take: a name is required.
#[derive(Deserialize)]
struct Rename {
    name: String,
}

async fn rename(Allowed(id): Allowed<Own>, Json(body): Json<Rename>) -> Json<Value> {
    Json(json!({"id": id, "name": body.name}))
}

async fn report(_: Allowed<Admins>, Json(body): Json<Rename>) -> Json<Value> {
    Json(json!({"name": body.name}))
}

/// Sends `POST uri` with `token` and the JSON `body` to the two routes,
/// over `state`, with the pool and keys `AuthUser` reads, as the
/// application's router puts them in every request.
async fn send(
    state: &AppState,
    token: Option<&str>,
    uri: &str,
    body: &Value,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/things/{id}", post(rename))
        .route("/report", post(report))
        .with_state(state.clone())
        .layer(Extension(state.pool.clone()))
        .layer(Extension(state.keys.clone()));
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let response = app
        .oneshot(request.body(Body::from(serde_json::to_vec(body)?))?)
        .await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    // axum's own `Json` rejection is plain text.
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok(Answer { status, body })
}

#[tokio::test]
async fn a_refused_caller_gets_403_before_the_body_is_read() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let (_, token) = account(&db.pool, Role::User).await?;
    let invalid = json!({"title": 1});

    // Someone else's id, and a body of the wrong shape: the 403, not a 422.
    let uri = format!("/things/{}", Uuid::now_v7());
    let answer = send(&state, Some(&token), &uri, &invalid).await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    assert_eq!(answer.code(), "forbidden");

    let answer = send(&state, Some(&token), "/report", &invalid).await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    assert_eq!(answer.body["error"]["message"], "this is for admins");

    // No caller is a 401, a malformed path parameter a 400.
    let answer = send(&state, None, &uri, &invalid).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    let answer = send(&state, Some(&token), "/things/not-a-uuid", &invalid).await?;
    assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{}", answer.body);
    db.drop().await
}

#[tokio::test]
async fn an_allowed_caller_gets_the_grant_and_the_body_is_read() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let (id, token) = account(&db.pool, Role::User).await?;
    let uri = format!("/things/{id}");

    let answer = send(&state, Some(&token), &uri, &json!({"name": "mine"})).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({"id": id, "name": "mine"}));

    // Let in, the body is read: a wrong one is the 422.
    let answer = send(&state, Some(&token), &uri, &json!({"title": 1})).await?;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );

    let (_, admin) = account(&db.pool, Role::Admin).await?;
    let answer = send(&state, Some(&admin), "/report", &json!({"name": "q3"})).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({"name": "q3"}));
    db.drop().await
}
