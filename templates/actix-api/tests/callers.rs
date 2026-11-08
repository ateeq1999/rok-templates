//! `AuthUser` and `AdminUser` check the caller before the body is read: a
//! caller without a valid token gets the `401`, and one who is not an admin
//! the `403`, whatever the body, so a bad body's `422` never shows a
//! route's rules to callers who can't use it. actix-web polls a handler's
//! extractors together, so each holds the body back (`shared::body`),
//! whichever order the handler takes them in.

mod support;

use actix_api::app::AppState;
use actix_api::app::middleware::hold_body;
use actix_api::db::types::Role;
use actix_api::shared::auth::{AdminUser, AuthUser};
use actix_api::shared::validation::ValidatedJson;
use actix_web::http::{Method, StatusCode, header};
use actix_web::middleware::from_fn;
use actix_web::{App, HttpResponse, test, web};
use serde::Deserialize;
use serde_json::{Value, json};
use support::{Answer, Database, TestResult, account, call, token};
use uuid::Uuid;

/// A well-formed token of no account: a `401` only once the database has
/// been asked, while a body could already have been read.
fn ghost() -> Result<String, Box<dyn std::error::Error>> {
    token(Uuid::now_v7(), Role::Admin, 0)
}

/// A body of the wrong shape for every route here: `422` once read.
fn invalid() -> Value {
    json!({"display_name": 5, "name": 5})
}

#[actix_web::test]
async fn a_generated_admin_route_checks_the_caller_before_the_body() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (id, user) = account(&db.pool, Role::User).await?;
    let (_, admin) = account(&db.pool, Role::Admin).await?;
    let uri = format!("/api/v1/users/{id}");

    let answer = call(&db.pool, None, Method::PATCH, &uri, Some(invalid())).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    let answer = call(
        &db.pool,
        Some(&ghost()?),
        Method::PATCH,
        &uri,
        Some(invalid()),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);

    let answer = call(&db.pool, Some(&user), Method::PATCH, &uri, Some(invalid())).await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    assert_eq!(answer.code(), "forbidden");

    // An admin is let in, and the body is read: the 422.
    let answer = call(&db.pool, Some(&admin), Method::PATCH, &uri, Some(invalid())).await?;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    db.drop().await
}

/// The body the hand-written routes take: a name is required.
#[derive(Debug, Default, Deserialize, rok_data::Input)]
struct Rename {
    name: String,
}

/// The body before the admin.
async fn admin_after(body: ValidatedJson<Rename>, _: AdminUser) -> HttpResponse {
    HttpResponse::Ok().json(json!({"name": body.into_inner().name}))
}

/// The body before the caller.
async fn caller_after(body: ValidatedJson<Rename>, user: AuthUser) -> HttpResponse {
    HttpResponse::Ok().json(json!({"id": user.id, "name": body.into_inner().name}))
}

/// The caller before the body.
async fn caller_first(user: AuthUser, body: ValidatedJson<Rename>) -> HttpResponse {
    HttpResponse::Ok().json(json!({"id": user.id, "name": body.into_inner().name}))
}

/// Sends `POST uri` with `token` and the JSON `body` to the hand-written
/// routes over `state`, with the app data `AuthUser` reads, gated by
/// `hold_body` when `gated` (as `app` is).
async fn send(
    state: &AppState,
    gated: bool,
    token: Option<&str>,
    uri: &str,
    body: &Value,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let routes = |app: App<_>| {
        app.app_data(web::Data::new(state.pool.clone()))
            .app_data(state.keys.clone())
            .route("/admin-after", web::post().to(admin_after))
            .route("/caller-after", web::post().to(caller_after))
            .route("/caller-first", web::post().to(caller_first))
    };
    let mut request = test::TestRequest::post().uri(uri).set_json(body);
    if let Some(token) = token {
        request = request.insert_header((header::AUTHORIZATION, format!("Bearer {token}")));
    }
    let response = if gated {
        let app = test::init_service(routes(App::new()).wrap(from_fn(hold_body))).await;
        test::call_service(&app, request.to_request()).await
    } else {
        let app = test::init_service(routes(App::new())).await;
        test::call_service(&app, request.to_request()).await
    };
    let status = response.status();
    let bytes = test::read_body(response).await;
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok(Answer { status, body })
}

#[actix_web::test]
async fn the_caller_is_checked_first_whichever_order_the_handler_takes() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = support::state(&db.pool)?;
    let (_, user) = account(&db.pool, Role::User).await?;
    let (_, admin) = account(&db.pool, Role::Admin).await?;
    let ghost = ghost()?;
    let bad = invalid();

    for uri in ["/admin-after", "/caller-after", "/caller-first"] {
        let answer = send(&state, true, None, uri, &bad).await?;
        assert_eq!(
            answer.status,
            StatusCode::UNAUTHORIZED,
            "{uri}: {}",
            answer.body
        );
        let answer = send(&state, true, Some(&ghost), uri, &bad).await?;
        assert_eq!(
            answer.status,
            StatusCode::UNAUTHORIZED,
            "{uri}: {}",
            answer.body
        );
    }
    let answer = send(&state, true, Some(&user), "/admin-after", &bad).await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);

    // Let in, the body is read: a bad one is the 422, a good one is used.
    for (uri, token) in [
        ("/admin-after", &admin),
        ("/caller-after", &user),
        ("/caller-first", &user),
    ] {
        let answer = send(&state, true, Some(token), uri, &bad).await?;
        assert_eq!(
            answer.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{uri}: {}",
            answer.body
        );
        let answer = send(&state, true, Some(token), uri, &json!({"name": "q3"})).await?;
        assert_eq!(answer.status, StatusCode::OK, "{uri}: {}", answer.body);
        assert_eq!(answer.body["name"], "q3");
    }
    db.drop().await
}

#[actix_web::test]
async fn without_the_middleware_the_body_after_the_caller_is_held_back() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = support::state(&db.pool)?;
    let (_, user) = account(&db.pool, Role::User).await?;

    let answer = send(&state, false, Some(&ghost()?), "/caller-first", &invalid()).await?;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{}", answer.body);
    let answer = send(&state, false, Some(&user), "/caller-first", &invalid()).await?;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    let answer = send(
        &state,
        false,
        Some(&user),
        "/caller-first",
        &json!({"name": "q3"}),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    db.drop().await
}
