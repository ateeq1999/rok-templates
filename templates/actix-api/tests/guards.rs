//! `Allowed<G>`: a handler requiring a guard gets its grant, and a caller
//! the guard refuses is answered `403` before the body is read, so a
//! refused caller never learns the body's rules from a `422`. actix-web
//! polls a handler's extractors together; `Allowed` holds the body back
//! (`shared::body`).

mod support;

use actix_api::app::AppState;
use actix_api::db::types::Role;
use actix_api::error::AppError;
use actix_api::shared::auth::{Allowed, AuthUser, Guard};
use actix_api::shared::validation::{ValidatedJson, path_config};
use actix_web::http::{StatusCode, header};
use actix_web::{App, HttpResponse, test, web};
use serde::Deserialize;
use serde_json::{Value, json};
use support::{Answer, Database, TestResult, account, state};
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
#[derive(Debug, Default, Deserialize, rok_data::Input)]
struct Rename {
    name: String,
}

async fn rename(Allowed(id): Allowed<Own>, body: ValidatedJson<Rename>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"id": id, "name": body.into_inner().name}))
}

async fn report(_: Allowed<Admins>, body: ValidatedJson<Rename>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"name": body.into_inner().name}))
}

/// Sends `POST uri` with `token` and the JSON `body` to the two routes,
/// over `state`, with the app data `AuthUser` and `Allowed` read, as
/// `app::configure` registers it.
async fn send(
    state: &AppState,
    token: Option<&str>,
    uri: &str,
    body: &Value,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(state.pool.clone()))
            .app_data(state.keys.clone())
            .app_data(path_config())
            .route("/things/{id}", web::post().to(rename))
            .route("/report", web::post().to(report)),
    )
    .await;
    let mut request = test::TestRequest::post().uri(uri).set_json(body);
    if let Some(token) = token {
        request = request.insert_header((header::AUTHORIZATION, format!("Bearer {token}")));
    }
    let response = test::call_service(&app, request.to_request()).await;
    let status = response.status();
    let bytes = test::read_body(response).await;
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok(Answer { status, body })
}

#[actix_web::test]
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

#[actix_web::test]
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
