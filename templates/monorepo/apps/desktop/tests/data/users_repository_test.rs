//! `UsersRepositoryImpl`: the API's users, a page at a time, through `clients/rust`.

use desktop::data::models::page::Page;
use desktop::data::models::user::User;
use desktop::data::providers::api::Api;
use desktop::data::repositories::users_repository::{UsersRepository, UsersRepositoryImpl};
use rok_ui::http::testing::MockServer;
use rok_ui::http::{Method, Session};
use rok_ui::runtime::block_on;
use serde_json::json;

const ID: &str = "0190c0de-0000-7000-8000-000000000002";

/// A user row, as `apps/api` answers it.
fn row(id: &str, email: &str, display_name: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "email": email,
        "display_name": display_name,
        "email_verified": true,
        "has_password": true,
        "is_active": true,
        "role": "user",
        "totp_enabled": false,
        "created_at": "2026-11-02T10:00:00Z",
    })
}

fn signed_in(server: &MockServer) -> UsersRepositoryImpl {
    UsersRepositoryImpl::new(Api::new(server.url(), Session::with_token("t0k3n")))
}

#[test]
fn a_page_of_users_is_the_page_the_api_answers() {
    let server = MockServer::start();
    server.on(Method::Get, "/api/v1/users").reply_json(
        200,
        &json!({
            "items": [row(ID, "grace@example.com", None)],
            "page": 2,
            "per_page": 20,
            "total": 21,
        }),
    );

    let page = block_on(signed_in(&server).list(2));

    assert_eq!(
        page,
        Ok(Page {
            items: vec![User {
                id: ID.into(),
                name: String::new(),
                email: "grace@example.com".into(),
            }],
            page: 2,
            per_page: 20,
            total: 21,
        })
    );
    let request = &server.requests_to("/api/v1/users")[0];
    assert_eq!(request.query, "page=2");
    assert_eq!(request.header("authorization"), Some("Bearer t0k3n"));
}

#[test]
fn one_user_is_the_row_the_api_answers() {
    let server = MockServer::start();
    server
        .on(Method::Get, &format!("/api/v1/users/{ID}"))
        .reply_json(200, &row(ID, "grace@example.com", Some("Grace")));

    let user = block_on(signed_in(&server).get(ID.into()));

    assert_eq!(
        user,
        Ok(User {
            id: ID.into(),
            name: "Grace".into(),
            email: "grace@example.com".into(),
        })
    );
}
