//! The guards recipe at work: `Allowed<Finance>`, `Allowed<PartnerAccess>`
//! and `Allowed<PartnerAccess<Organizer>>` on routes, each let in or
//! refused (before the body is read) as its rule says.

mod support;

use axum::body::{Body, to_bytes};
use axum::extract::Extension;
use axum::http::{Method, Request, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use axum_api::app::AppState;
use axum_api::db::types::Role;
use axum_api::shared::auth::Allowed;
use axum_api::shared::guards::{Finance, Organizer, PartnerAccess, PartnerGrant};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{Answer, Database, TestResult, account, state};
use tower::ServiceExt;
use uuid::Uuid;

/// A body every route takes.
#[derive(Deserialize)]
pub struct Rename {
    pub name: String,
}

pub async fn payouts(_: Allowed<Finance>) -> Json<Value> {
    Json(json!({"payouts": []}))
}

pub async fn rename_partner(
    Allowed(access): Allowed<PartnerAccess>,
    Json(body): Json<Rename>,
) -> Json<Value> {
    let by = match access {
        PartnerGrant::Member(role) => format!("member:{role:?}"),
        PartnerGrant::Staff(role) => format!("staff:{role:?}"),
    };
    Json(json!({"name": body.name, "by": by}))
}

pub async fn create_event(
    Allowed(_access): Allowed<PartnerAccess<Organizer>>,
    Json(body): Json<Rename>,
) -> Json<Value> {
    Json(json!({"event": body.name}))
}

/// Sends `POST uri` with `token` and `body` to the recipe's routes.
async fn send(
    state: &AppState,
    token: &str,
    uri: &str,
    body: &Value,
) -> Result<Answer, Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/payouts", post(payouts))
        .route("/partners/{partner_id}", post(rename_partner))
        .route("/partners/{partner_id}/events", post(create_event))
        .with_state(state.clone())
        .layer(Extension(state.pool.clone()))
        .layer(Extension(state.keys.clone()));
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(serde_json::to_vec(body)?))?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok(Answer { status, body })
}

/// A partner of `kind`.
async fn partner(pool: &PgPool, kind: &str) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO partners (id, name, kind) VALUES ($1, 'p', $2::partner_kind)")
        .bind(id)
        .bind(kind)
        .execute(pool)
        .await?;
    Ok(id)
}

/// `user` a member of `partner` with `role`.
async fn member(pool: &PgPool, partner: Uuid, user: Uuid, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO partner_members (id, partner_id, user_id, role) VALUES ($1, $2, $3, $4::partner_role)",
    )
    .bind(Uuid::now_v7())
    .bind(partner)
    .bind(user)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

/// `user` staff with `role`.
async fn staff(pool: &PgPool, user: Uuid, role: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO staff_members (id, user_id, role) VALUES ($1, $2, $3::staff_role)")
        .bind(Uuid::now_v7())
        .bind(user)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}

#[tokio::test]
async fn each_guard_lets_in_or_refuses_as_its_rule_says() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (pool, state) = (&db.pool, state(&db.pool)?);
    let (owner, owner_token) = account(pool, Role::User).await?;
    let (finance, finance_token) = account(pool, Role::User).await?;
    let (support_staff, support_token) = account(pool, Role::User).await?;
    let (_, stranger_token) = account(pool, Role::User).await?;
    let (organizer, venue) = (
        partner(pool, "organizer").await?,
        partner(pool, "venue").await?,
    );
    member(pool, organizer, owner, "owner").await?;
    member(pool, venue, owner, "manager").await?;
    staff(pool, finance, "finance").await?;
    staff(pool, support_staff, "support").await?;
    let (valid, invalid) = (json!({"name": "n"}), json!({"title": 1}));

    // `Finance`: finance staff only.
    let answer = send(&state, &finance_token, "/payouts", &valid).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    for token in [&support_token, &owner_token] {
        let answer = send(&state, token, "/payouts", &valid).await?;
        assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    }

    // `PartnerAccess`: a member, with their role, or staff; refused before
    // the body is read.
    let at = format!("/partners/{venue}");
    let answer = send(&state, &owner_token, &at, &valid).await?;
    assert_eq!(answer.body, json!({"name": "n", "by": "member:Manager"}));
    let answer = send(&state, &support_token, &at, &valid).await?;
    assert_eq!(answer.body, json!({"name": "n", "by": "staff:Support"}));
    let answer = send(&state, &stranger_token, &at, &invalid).await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);

    // `PartnerAccess<Organizer>`: and the partner is an organizer.
    let answer = send(
        &state,
        &owner_token,
        &format!("/partners/{organizer}/events"),
        &valid,
    )
    .await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    let answer = send(
        &state,
        &owner_token,
        &format!("/partners/{venue}/events"),
        &invalid,
    )
    .await?;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    assert_eq!(answer.body["error"]["message"], "this isn't an organizer");
    db.drop().await
}
