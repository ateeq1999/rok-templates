//! The caller check (`app::authenticate`) as a client sees it: the bearer
//! tokens and API keys the axum template issues name the caller here too,
//! and a missing, expired, forged or revoked credential is
//! `UNAUTHENTICATED` (a call without any is `tests/server.rs`'s).
//!
//! The tokens that fail before the database (expired, forged, malformed)
//! run over a pool that never connects; the calls that look an account or
//! a key up need `DATABASE_URL` and skip, with a note, without it.

mod support;

use std::time::Duration;

use grpc_api::app;
use grpc_api::db::queries::api_keys::{InsertApiKeyParams, insert_api_key};
use grpc_api::db::types::{ApiKeyScope, Role};
use grpc_api::db::users::models::{NewUser, User};
use grpc_api::features::proto;
use grpc_api::features::proto::users_service_client::UsersServiceClient;
use grpc_api::shared::auth::{Claims, JwtKeys};
use grpc_api::shared::security::api_key::NewApiKey;
use jsonwebtoken::{EncodingKey, Header};
use sqlx::PgPool;
use support::{Database, TestResult, channel, config, offline};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::{Code, Request, Status};
use tonic_health::server::HealthReporter;
use uuid::Uuid;

/// The test application's `JWT_SECRET` (`support::config`).
const SECRET: &str = "a test secret of at least 32 bytes";

/// A channel to the whole application over `pool`, configured with
/// `overrides`: the API behind the caller check.
async fn application(
    pool: &PgPool,
    overrides: &[(&str, &str)],
) -> Result<Channel, Box<dyn std::error::Error>> {
    let state = app::AppState::new(pool.clone(), &config(overrides)?);
    channel(app::routes(&state, HealthReporter::new())?).await
}

/// A `Get` of the user `id`, carrying the `name: value` metadata.
fn get(
    id: Uuid,
    name: &'static str,
    value: &str,
) -> Result<Request<proto::GetUserRequest>, Status> {
    let mut request = Request::new(proto::GetUserRequest { id: id.to_string() });
    let value: MetadataValue<_> = value
        .parse()
        .map_err(|_| Status::internal("not a metadata value"))?;
    request.metadata_mut().insert(name, value);
    Ok(request)
}

/// A bearer `token` in `Authorization`.
fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

/// A token for `id` at `token_version`, signed with `secret`.
fn token(secret: &str, id: Uuid, token_version: i32) -> Result<String, Box<dyn std::error::Error>> {
    let keys = JwtKeys::new(secret, Duration::from_secs(900));
    Ok(keys.issue(id, Role::Admin, token_version)?)
}

/// A bearer token for `user`, as the axum template signs one at sign-in.
fn session(user: &User) -> Result<String, Box<dyn std::error::Error>> {
    Ok(bearer(&token(SECRET, user.id.into(), user.token_version)?))
}

/// The status a `Get` with `name: value` is answered with, over a pool
/// that never connects.
async fn refused(name: &'static str, value: &str) -> Result<Status, Box<dyn std::error::Error>> {
    let mut users = UsersServiceClient::new(application(&offline()?, &[]).await?);
    let answer = users.get(get(Uuid::now_v7(), name, value)?).await;
    Ok(answer.err().ok_or("the call was answered")?)
}

/// A new account with `role`, made through the data layer.
async fn account(pool: &PgPool, role: Role) -> Result<User, Box<dyn std::error::Error>> {
    let id = Uuid::now_v7();
    let user = User::create(
        pool,
        NewUser {
            id: Some(id.into()),
            email: format!("user-{}@test.invalid", id.simple()),
            display_name: Some("Ada".to_owned()),
            role: Some(role),
            is_active: None,
        },
    )
    .await?;
    Ok(user)
}

/// A new API key of `user` with `scope`: the raw key, as it is shown once.
async fn api_key(
    pool: &PgPool,
    user: &User,
    scope: ApiKeyScope,
) -> Result<String, Box<dyn std::error::Error>> {
    let key = NewApiKey::generate();
    insert_api_key(
        pool,
        InsertApiKeyParams {
            id: Uuid::now_v7().into(),
            user_id: user.id,
            name: "a script".to_owned(),
            key_prefix: key.prefix,
            key_hash: key.hash,
            scope,
            expires_at: None,
        },
    )
    .await?;
    Ok(key.raw)
}

/// An admin's own account, as `Get` answers it to a call with
/// `name: value`.
async fn got(
    database: &Database,
    user: &User,
    name: &'static str,
    value: &str,
    overrides: &[(&str, &str)],
) -> Result<proto::User, Box<dyn std::error::Error>> {
    let mut users = UsersServiceClient::new(application(&database.pool, overrides).await?);
    Ok(users
        .get(get(user.id.into(), name, value)?)
        .await?
        .into_inner())
}

/// A token past its `exp` is `UNAUTHENTICATED`, before the account is
/// looked up.
#[tokio::test]
async fn an_expired_token_is_unauthenticated() -> TestResult {
    let claims = Claims {
        sub: Uuid::now_v7(),
        role: Role::Admin,
        tv: 0,
        exp: 1,
    };
    let expired = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )?;
    let status = refused("authorization", &bearer(&expired)).await?;
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
    Ok(())
}

/// A token signed with another secret, or no token at all, is
/// `UNAUTHENTICATED`.
#[tokio::test]
async fn a_forged_or_malformed_token_is_unauthenticated() -> TestResult {
    let forged = token("another secret, also of 32 bytes", Uuid::now_v7(), 1)?;
    for value in [
        bearer(&forged),
        bearer("not.a.token"),
        "Basic abc".to_owned(),
    ] {
        let status = refused("authorization", &value).await?;
        assert_eq!(status.code(), Code::Unauthenticated, "{value}: {status:?}");
    }
    Ok(())
}

/// A valid token names its account as the caller: an admin gets a user.
#[tokio::test]
async fn a_valid_token_names_the_caller() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&database.pool, Role::Admin).await?;
    let value = session(&admin)?;
    let user = got(&database, &admin, "authorization", &value, &[]).await?;
    assert_eq!(user.id, Uuid::from(admin.id).to_string());
    database.drop().await
}

/// A token signed with the secret being rotated out
/// (`JWT_SECRET_PREVIOUS`) is still accepted.
#[tokio::test]
async fn a_token_signed_with_the_previous_secret_is_accepted() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&database.pool, Role::Admin).await?;
    let value = session(&admin)?;
    let rotated = [
        ("JWT_SECRET", "the new secret, at least 32 bytes long"),
        ("JWT_SECRET_PREVIOUS", SECRET),
    ];
    let user = got(&database, &admin, "authorization", &value, &rotated).await?;
    assert_eq!(user.id, Uuid::from(admin.id).to_string());
    database.drop().await
}

/// The role comes from the database, not the token: a user whose token
/// says admin is `PERMISSION_DENIED` on the admin-only service.
#[tokio::test]
async fn the_role_comes_from_the_database() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let user = account(&database.pool, Role::User).await?;
    let value = session(&user)?;
    let refused = got(&database, &user, "authorization", &value, &[]).await;
    let status = refused.err().ok_or("a user got a user")?;
    let status = status.downcast_ref::<Status>().ok_or("not a status")?;
    assert_eq!(status.code(), Code::PermissionDenied, "{status:?}");
    database.drop().await
}

/// A token of an account that no longer exists is `UNAUTHENTICATED`.
#[tokio::test]
async fn a_token_of_a_missing_account_is_unauthenticated() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let value = bearer(&token(SECRET, Uuid::now_v7(), 1)?);
    let mut users = UsersServiceClient::new(application(&database.pool, &[]).await?);
    let refused = users
        .get(get(Uuid::now_v7(), "authorization", &value)?)
        .await;
    let status = refused.err().ok_or("a missing account was admitted")?;
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
    database.drop().await
}

/// An API key names its owner as the caller, in `Authorization` or in
/// `x-api-key`.
#[tokio::test]
async fn an_api_key_names_its_owner() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&database.pool, Role::Admin).await?;
    let key = api_key(&database.pool, &admin, ApiKeyScope::Write).await?;
    for (name, value) in [("authorization", bearer(&key)), ("x-api-key", key.clone())] {
        let user = got(&database, &admin, name, &value, &[]).await?;
        assert_eq!(user.id, Uuid::from(admin.id).to_string(), "{name}");
    }
    database.drop().await
}

/// An unknown API key is `UNAUTHENTICATED`.
#[tokio::test]
async fn an_unknown_api_key_is_unauthenticated() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let key = NewApiKey::generate().raw;
    let mut users = UsersServiceClient::new(application(&database.pool, &[]).await?);
    let refused = users.get(get(Uuid::now_v7(), "x-api-key", &key)?).await;
    let status = refused.err().ok_or("an unknown key was admitted")?;
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
    database.drop().await
}

/// A `read` key reaches the reads (`Get`, `List`) but not the writes: an
/// `Update` is `PERMISSION_DENIED`.
#[tokio::test]
async fn a_read_key_may_only_read() -> TestResult {
    let Some(database) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&database.pool, Role::Admin).await?;
    let key = api_key(&database.pool, &admin, ApiKeyScope::Read).await?;
    let user = got(&database, &admin, "x-api-key", &key, &[]).await?;
    assert_eq!(user.id, Uuid::from(admin.id).to_string());
    let mut users = UsersServiceClient::new(application(&database.pool, &[]).await?);
    let mut update = Request::new(proto::UpdateUserRequest {
        id: Uuid::from(admin.id).to_string(),
        ..Default::default()
    });
    update.metadata_mut().insert("x-api-key", key.parse()?);
    let status = users
        .update(update)
        .await
        .err()
        .ok_or("a read key updated a user")?;
    assert_eq!(status.code(), Code::PermissionDenied, "{status:?}");
    database.drop().await
}
