//! The auth flows through `AuthBloc`, the repository and the generated client, answered by a
//! mock `apps/api` on localhost, with the token kept in memory: no test touches the keychain.

use std::sync::Arc;

use desktop::data::models::user::User;
use desktop::data::providers::api::Api;
use desktop::data::providers::token_store::MemoryTokenStore;
use desktop::data::repositories::auth_repository::AuthRepositoryImpl;
use desktop::features::auth::bloc::auth_bloc::AuthBloc;
use desktop::features::auth::bloc::auth_event::AuthEvent;
use desktop::features::auth::bloc::auth_state::{AuthState, AuthStatus};
use rok_ui::bloc::test;
use rok_ui::http::testing::MockServer;
use rok_ui::http::Method;
use serde_json::json;

const REGISTER: &str = "/api/v1/auth/register";
const LOGIN: &str = "/api/v1/auth/login";
const VERIFY_CODE: &str = "/api/v1/auth/2fa/verify";
const ME: &str = "/api/v1/auth/me";

const EMAIL: &str = "ada@example.com";
const PASSWORD: &str = "correct horse battery";

/// The account `apps/api` answers with.
fn account(email_verified: bool) -> serde_json::Value {
    json!({
        "id": "0190c0de-0000-7000-8000-000000000001",
        "email": EMAIL,
        "display_name": "Ada",
        "email_verified": email_verified,
        "has_password": true,
        "is_active": true,
        "role": "user",
        "totp_enabled": false,
        "created_at": "2026-11-02T10:00:00Z",
    })
}

fn ada() -> User {
    User {
        id: "0190c0de-0000-7000-8000-000000000001".into(),
        name: "Ada".into(),
        email: EMAIL.into(),
    }
}

fn token(access_token: &str) -> serde_json::Value {
    json!({"access_token": access_token, "token_type": "Bearer", "expires_in": 900})
}

fn error(code: &str, message: &str) -> serde_json::Value {
    json!({"error": {"code": code, "message": message}})
}

/// The App as `main` builds it, over `server`, keeping its token in `store`.
fn app(server: &MockServer, store: &MemoryTokenStore) -> (Api, AuthBloc) {
    let api = Api::kept(server.url(), Arc::new(store.clone()));
    let bloc = AuthBloc::new(Arc::new(AuthRepositoryImpl::new(api.clone())));
    (api, bloc)
}

fn sign_in() -> AuthEvent {
    AuthEvent::SignInSubmitted {
        email: EMAIL.into(),
        password: PASSWORD.into(),
    }
}

fn last(states: &[AuthState]) -> &AuthState {
    states
        .last()
        .unwrap_or_else(|| panic!("no state was emitted"))
}

#[test]
fn signing_up_asks_to_verify_the_email() {
    let server = MockServer::start();
    server
        .on(Method::Post, REGISTER)
        .reply_json(201, &account(false));
    let store = MemoryTokenStore::new();
    let (_api, bloc) = app(&server, &store);

    let states = test::run(
        bloc,
        [AuthEvent::SignUpSubmitted {
            email: EMAIL.into(),
            password: PASSWORD.into(),
        }],
    );

    let state = last(&states);
    assert_eq!(state.status, AuthStatus::Success);
    assert_eq!(state.signed_up.as_deref(), Some(EMAIL));
    assert_eq!(state.user, None);
    assert_eq!(
        server.requests_to(REGISTER)[0].json(),
        json!({"email": EMAIL, "password": PASSWORD})
    );
    assert_eq!(store.token(), None);
}

#[test]
fn an_unverified_account_is_told_what_the_api_answers() {
    let server = MockServer::start();
    server.on(Method::Post, LOGIN).reply_json(
        403,
        &error("forbidden", "the email address is not verified yet"),
    );
    let store = MemoryTokenStore::new();
    let (_api, bloc) = app(&server, &store);

    let states = test::run(bloc, [sign_in()]);

    let state = last(&states);
    assert_eq!(state.status, AuthStatus::Failure);
    assert_eq!(
        state.error.as_ref().map(ToString::to_string).as_deref(),
        Some("the email address is not verified yet")
    );
    assert_eq!(store.token(), None);
}

#[test]
fn signing_in_keeps_the_token() {
    let server = MockServer::start();
    server
        .on(Method::Post, LOGIN)
        .reply_json(200, &token("t0k3n"));
    server.on(Method::Get, ME).reply_json(200, &account(true));
    let store = MemoryTokenStore::new();
    let (_api, bloc) = app(&server, &store);

    let states = test::run(bloc, [sign_in()]);

    let state = last(&states);
    assert_eq!(state.status, AuthStatus::Success);
    assert_eq!(state.user, Some(ada()));
    assert_eq!(store.token().as_deref(), Some("t0k3n"));
}

#[test]
fn a_kept_token_signs_in_at_start() {
    let server = MockServer::start();
    server.on(Method::Get, ME).reply_json(200, &account(true));
    let store = MemoryTokenStore::with_token("kept");
    let (api, bloc) = app(&server, &store);
    assert_eq!(api.session().token().as_deref(), Some("kept"));

    let states = test::run(bloc, [AuthEvent::ProfileRequested]);

    assert_eq!(last(&states).user, Some(ada()));
    assert_eq!(
        server.requests_to(ME)[0].header("authorization"),
        Some("Bearer kept")
    );
}

#[test]
fn signing_out_forgets_the_kept_token() {
    let server = MockServer::start();
    let store = MemoryTokenStore::with_token("kept");
    let (api, bloc) = app(&server, &store);

    let states = test::run(bloc, [AuthEvent::SignOutRequested]);

    assert_eq!(last(&states).user, None);
    assert_eq!(store.token(), None);
    assert_eq!(api.session().token(), None);
    assert!(!api.session().is_expired());
}

#[test]
fn a_rejected_token_is_forgotten() {
    let server = MockServer::start();
    server
        .on(Method::Get, ME)
        .reply_json(401, &error("unauthorized", "The session has expired."));
    let store = MemoryTokenStore::with_token("stale");
    let (api, bloc) = app(&server, &store);

    let states = test::run(bloc, [AuthEvent::ProfileRequested]);

    assert_eq!(last(&states).status, AuthStatus::Failure);
    assert_eq!(store.token(), None);
    assert!(api.session().is_expired());
}

/// `apps/api`'s answer to the right password of an account with two-factor on.
fn code_required(server: &MockServer) {
    server.on(Method::Post, LOGIN).reply_json(
        200,
        &json!({"pending_token": "p3nd1ng", "requires_totp": true}),
    );
}

#[test]
fn a_two_factor_account_signs_in_with_its_code() {
    let server = MockServer::start();
    code_required(&server);
    server
        .on(Method::Post, VERIFY_CODE)
        .reply_json(200, &token("t0k3n"));
    server.on(Method::Get, ME).reply_json(200, &account(true));
    let store = MemoryTokenStore::new();
    let (_api, bloc) = app(&server, &store);

    let states = test::run_settled(
        bloc,
        [
            sign_in(),
            AuthEvent::CodeSubmitted {
                code: "123456".into(),
            },
        ],
    );

    assert!(
        states
            .iter()
            .any(|state| state.code_required() && state.user.is_none()),
        "the code is asked for: {states:?}"
    );
    let state = last(&states);
    assert_eq!(state.user, Some(ada()));
    assert!(!state.code_required());
    assert_eq!(
        server.requests_to(VERIFY_CODE)[0].json(),
        json!({"pending_token": "p3nd1ng", "code": "123456"})
    );
    assert_eq!(store.token().as_deref(), Some("t0k3n"));
}

#[test]
fn a_wrong_code_is_asked_for_again() {
    let server = MockServer::start();
    code_required(&server);
    server
        .on(Method::Post, VERIFY_CODE)
        .reply_json(401, &error("unauthorized", "Invalid code."));
    let store = MemoryTokenStore::new();
    let (api, bloc) = app(&server, &store);

    let states = test::run_settled(
        bloc,
        [
            sign_in(),
            AuthEvent::CodeSubmitted {
                code: "000000".into(),
            },
        ],
    );

    let state = last(&states);
    assert_eq!(state.status, AuthStatus::Failure);
    assert!(state.code_required());
    assert_eq!(
        state.error.as_ref().map(ToString::to_string).as_deref(),
        Some("Invalid code.")
    );
    assert_eq!(store.token(), None);
    assert!(!api.session().is_expired());
}

#[test]
fn a_spent_sign_in_starts_over() {
    let server = MockServer::start();
    code_required(&server);
    server.on(Method::Post, VERIFY_CODE).reply_json(
        400,
        &error("invalid_token", "The link is invalid or has expired."),
    );
    let (_api, bloc) = app(&server, &MemoryTokenStore::new());

    let states = test::run_settled(
        bloc,
        [
            sign_in(),
            AuthEvent::CodeSubmitted {
                code: "123456".into(),
            },
        ],
    );

    let state = last(&states);
    assert_eq!(state.status, AuthStatus::Failure);
    assert!(!state.code_required());
}

#[test]
fn cancelling_the_code_goes_back_to_the_password() {
    let server = MockServer::start();
    code_required(&server);
    let (_api, bloc) = app(&server, &MemoryTokenStore::new());

    let states = test::run_settled(bloc, [sign_in(), AuthEvent::CodeCancelled]);

    let state = last(&states);
    assert!(!state.code_required());
    assert_eq!(state.status, AuthStatus::Initial);
    assert_eq!(server.requests_to(VERIFY_CODE).len(), 0);
}

#[test]
fn no_state_shows_the_token() {
    let server = MockServer::start();
    code_required(&server);
    let (_api, bloc) = app(&server, &MemoryTokenStore::new());

    let states = test::run(bloc, [sign_in()]);

    assert!(!format!("{states:?}").contains("p3nd1ng"), "{states:?}");
}
