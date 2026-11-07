//! The development seed (`app::seed`, `cargo run --bin seed`): two verified
//! accounts that sign in, new passwords on every run, and `--promote`.

mod support;

use axum::http::{Method, StatusCode};
use axum_api::app::seed::{self, ADMIN_EMAIL, USER_EMAIL};
use axum_api::db::types::Role;
use serde_json::json;
use support::{Answer, Database, TestResult, account, call};

/// Signs in as `email` with `password` (`POST /auth/login`).
async fn sign_in(
    pool: &sqlx::PgPool,
    email: &str,
    password: &str,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let body = json!({"email": email, "password": password});
    call(pool, None, Method::POST, "/api/v1/auth/login", Some(body)).await
}

#[tokio::test]
async fn the_seeded_accounts_sign_in_with_their_roles() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let seeded = seed::seed(&db.pool).await?;
    let found: Vec<(&str, Role)> = seeded
        .iter()
        .map(|account| (account.email.as_str(), account.role))
        .collect();
    assert_eq!(
        found,
        [(ADMIN_EMAIL, Role::Admin), (USER_EMAIL, Role::User)]
    );
    for (account, role) in seeded.iter().zip(["admin", "user"]) {
        let answer = sign_in(&db.pool, &account.email, &account.password).await?;
        assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
        let token = answer.body["access_token"].as_str().unwrap_or_default();
        let me = call(&db.pool, Some(token), Method::GET, "/api/v1/auth/me", None).await?;
        assert_eq!(me.status, StatusCode::OK, "{}", me.body);
        assert_eq!(me.body["role"], role);
        assert_eq!(me.body["email_verified"], true, "{}", me.body);
    }

    // Again: the same accounts, new passwords; the old ones no longer work.
    let again = seed::seed(&db.pool).await?;
    let (old, new) = (&seeded[0], &again[0]);
    assert_eq!(new.email, old.email);
    assert_ne!(new.password, old.password);
    let answer = sign_in(&db.pool, &old.email, &old.password).await?;
    assert_ne!(answer.status, StatusCode::OK, "{}", answer.body);
    let answer = sign_in(&db.pool, &new.email, &new.password).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    db.drop().await
}

#[tokio::test]
async fn promote_makes_an_existing_account_an_admin() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (id, token) = account(&db.pool, Role::User).await?;
    let email = format!("caller-{}@test.invalid", id.simple());
    let me = call(&db.pool, Some(&token), Method::GET, "/api/v1/auth/me", None).await?;
    assert_eq!(me.body["role"], "user", "{}", me.body);

    // Whatever its case.
    let promoted = seed::promote(&db.pool, &email.to_uppercase()).await?;
    assert_eq!(promoted.as_deref(), Some(email.as_str()));
    let me = call(&db.pool, Some(&token), Method::GET, "/api/v1/auth/me", None).await?;
    assert_eq!(me.body["role"], "admin", "{}", me.body);

    assert_eq!(seed::promote(&db.pool, "nobody@test.invalid").await?, None);
    db.drop().await
}
