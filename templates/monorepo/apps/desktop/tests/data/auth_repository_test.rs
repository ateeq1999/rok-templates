//! `AuthRepositoryImpl`: signing in and out of `apps/api` through `clients/rust`.

use desktop::data::models::sign_in::SignIn;
use desktop::data::models::user::User;
use desktop::data::providers::api::Api;
use desktop::data::repositories::auth_repository::{AuthRepository, AuthRepositoryImpl};
use gpui::SharedString;
use rok_ui::form::{self, ServerErrorOptions};
use rok_ui::http::testing::MockServer;
use rok_ui::http::{Method, Session};
use rok_ui::runtime::block_on;
use serde_json::json;

const LOGIN: &str = "/api/v1/auth/login";
const ME: &str = "/api/v1/auth/me";
const VERIFY_CODE: &str = "/api/v1/auth/2fa/verify";

/// The account `apps/api` answers `GET /auth/me` with.
fn account() -> serde_json::Value {
    json!({
        "id": "0190c0de-0000-7000-8000-000000000001",
        "email": "ada@example.com",
        "display_name": "Ada",
        "email_verified": true,
        "has_password": true,
        "is_active": true,
        "role": "user",
        "totp_enabled": false,
        "created_at": "2026-11-02T10:00:00Z",
    })
}

fn signed_in(server: &MockServer) {
    server.on(Method::Post, LOGIN).reply_json(
        200,
        &json!({"access_token": "t0k3n", "token_type": "Bearer", "expires_in": 900}),
    );
    server.on(Method::Get, ME).reply_json(200, &account());
}

#[test]
fn signing_in_keeps_the_token_and_answers_the_account() {
    let server = MockServer::start();
    signed_in(&server);
    let session = Session::new();
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), session.clone()));

    let user = block_on(repository.sign_in("ada@example.com".into(), "correct horse".into()));

    assert_eq!(
        user,
        Ok(SignIn::SignedIn(User {
            id: "0190c0de-0000-7000-8000-000000000001".into(),
            name: "Ada".into(),
            email: "ada@example.com".into(),
        }))
    );
    assert_eq!(
        server.requests_to(LOGIN)[0].json(),
        json!({"email": "ada@example.com", "password": "correct horse"})
    );
    assert_eq!(
        server.requests_to(ME)[0].header("authorization"),
        Some("Bearer t0k3n")
    );
    assert_eq!(session.token().as_deref(), Some("t0k3n"));
}

#[test]
fn a_two_factor_account_is_signed_in_by_its_code() {
    let server = MockServer::start();
    server.on(Method::Post, LOGIN).reply_json(
        200,
        &json!({"pending_token": "p3nd1ng", "requires_totp": true}),
    );
    server.on(Method::Post, VERIFY_CODE).reply_json(
        200,
        &json!({"access_token": "t0k3n", "token_type": "Bearer", "expires_in": 900}),
    );
    server.on(Method::Get, ME).reply_json(200, &account());
    let session = Session::new();
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), session.clone()));

    let answer = block_on(repository.sign_in("ada@example.com".into(), "correct horse".into()));
    let Ok(SignIn::CodeRequired(pending)) = answer else {
        panic!("the code is not asked for: {answer:?}");
    };
    assert_eq!(session.token(), None, "no session without the code");
    let user = block_on(repository.verify_code(pending, "123456".into())).expect("signed in");

    assert_eq!(user.email, "ada@example.com");
    assert_eq!(
        server.requests_to(VERIFY_CODE)[0].json(),
        json!({"pending_token": "p3nd1ng", "code": "123456"})
    );
    assert_eq!(session.token().as_deref(), Some("t0k3n"));
}

#[test]
fn a_rejected_sign_in_lands_on_the_forms_fields() {
    let server = MockServer::start();
    server.on(Method::Post, LOGIN).reply_json(
        422,
        &json!({"error": {
            "code": "validation_failed",
            "message": "The request is invalid.",
            "details": {"password": [{
                "code": "length",
                "message": "Use 8 to 128 characters.",
                "params": {"min": 8, "max": 128},
            }]},
        }}),
    );
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), Session::new()));

    let error = block_on(repository.sign_in("ada@example.com".into(), "short".into()))
        .expect_err("a 422 is an error");

    assert_eq!(
        (error.status, error.code.as_str()),
        (422, "validation_failed")
    );
    let shown = form::to_server_errors(&error, &ServerErrorOptions::new());
    assert_eq!(
        shown.fields,
        [(
            SharedString::from("password"),
            SharedString::from("Use 8 to 128 characters.")
        )]
    );
}

#[test]
fn wrong_credentials_are_an_answer_not_an_expired_session() {
    let server = MockServer::start();
    server.on(Method::Post, LOGIN).reply_json(
        401,
        &json!({"error": {"code": "unauthorized", "message": "Invalid email or password."}}),
    );
    let session = Session::with_token("earlier");
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), session.clone()));

    let error = block_on(repository.sign_in("ada@example.com".into(), "wrong pass".into()))
        .expect_err("a 401 is an error");

    assert_eq!(error.message, "Invalid email or password.");
    assert!(!session.is_expired());
}

#[test]
fn a_rejected_token_expires_the_session() {
    let server = MockServer::start();
    server.on(Method::Get, ME).reply_json(
        401,
        &json!({"error": {"code": "unauthorized", "message": "The session has expired."}}),
    );
    let session = Session::with_token("stale");
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), session.clone()));

    let error = block_on(repository.me()).expect_err("a 401 is an error");

    assert_eq!(error.status, 401);
    assert!(session.is_expired());
    assert_eq!(
        server.requests_to(ME)[0].header("authorization"),
        Some("Bearer stale")
    );
}

#[test]
fn signing_out_forgets_the_token() {
    let server = MockServer::start();
    signed_in(&server);
    let session = Session::new();
    let repository = AuthRepositoryImpl::new(Api::new(server.url(), session.clone()));
    block_on(repository.sign_in("ada@example.com".into(), "correct horse".into()))
        .expect("signed in");

    block_on(repository.sign_out()).expect("signed out");
    server.clear_requests();
    let _ = block_on(repository.me());

    assert_eq!(session.token(), None);
    assert!(!session.is_expired());
    assert_eq!(server.requests_to(ME)[0].header("authorization"), None);
}
