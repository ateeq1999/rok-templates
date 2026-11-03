//! The hand-written streaming RPC, `UserEventsService.Subscribe`
//! (`proto/user_events.proto`), called through its client over an
//! in-process channel: an admin subscribes and receives one event per
//! `NOTIFY user_events, '<user id>'`, the user as it is now or none once
//! it is gone, also through the real caller check with a `read` API key;
//! a caller who is not an admin is turned away before the
//! database is reached.
//!
//! The subscriptions that reach the database need `DATABASE_URL` and skip,
//! with a note, without it.

mod support;

use std::time::Duration;

use grpc_api::app;
use grpc_api::db::queries::api_keys::{InsertApiKeyParams, insert_api_key};
use grpc_api::db::types::{ApiKeyScope, Role};
use grpc_api::db::users::models::{NewUser, User};
use grpc_api::features::proto;
use grpc_api::features::proto::user_events_service_client::UserEventsServiceClient;
use grpc_api::features::user_events::{CHANNEL, UserEventsService};
use grpc_api::shared::security::api_key::NewApiKey;
use sqlx::PgPool;
use support::{Database, TestResult, as_caller, channel, offline, scoped, state};
use tokio_stream::StreamExt;
use tonic::Code;
use tonic::transport::Channel;
use tonic_health::server::HealthReporter;
use uuid::Uuid;

/// How long a test waits for an event before it fails.
const PATIENCE: Duration = Duration::from_secs(10);

/// Held by each test that notifies: the channel is the database's, so one
/// test's notifications would reach another's subscriber.
static ALONE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// A client of the user events over `pool`, behind the tests' caller
/// check.
async fn events(
    pool: &PgPool,
) -> Result<UserEventsServiceClient<Channel>, Box<dyn std::error::Error>> {
    Ok(UserEventsServiceClient::new(
        channel(scoped(pool, HealthReporter::new())?).await?,
    ))
}

/// What a client sends to subscribe, as the caller `id` with `role`.
fn subscribe(id: Uuid, role: Role) -> tonic::Request<proto::SubscribeUserEventsRequest> {
    as_caller(proto::SubscribeUserEventsRequest {}, id, role)
}

/// `NOTIFY user_events, '<payload>'`, as any publisher sends it: a trigger,
/// another process sharing the database, or `psql`.
async fn notify(pool: &PgPool, payload: &str) -> TestResult {
    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(CHANNEL)
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(())
}

/// A subscription without a caller is `UNAUTHENTICATED`, and one by a
/// caller who is not an admin `PERMISSION_DENIED`: the users are
/// admin-only, as `UsersService` is.
#[tokio::test]
async fn only_an_admin_subscribes() -> TestResult {
    let mut client = events(&offline()?).await?;
    let status = client
        .subscribe(proto::SubscribeUserEventsRequest {})
        .await
        .err()
        .ok_or("a call without a caller subscribed")?;
    assert_eq!(status.code(), Code::Unauthenticated, "{status:?}");
    let status = client
        .subscribe(subscribe(Uuid::now_v7(), Role::User))
        .await
        .err()
        .ok_or("a user subscribed")?;
    assert_eq!(status.code(), Code::PermissionDenied, "{status:?}");
    Ok(())
}

/// An admin subscribes, and each `NOTIFY` naming a user is one event: the
/// user as it is now, then, once the user is deleted (and
/// `UserEventsService::publish` names it), the id alone. A
/// payload that names no user is skipped.
#[tokio::test]
async fn a_subscriber_receives_an_event_per_notification() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let _alone = ALONE.lock().await;
    let id = Uuid::now_v7();
    let user = User::create(
        &db.pool,
        NewUser {
            id: Some(id.into()),
            email: format!("user-{}@test.invalid", id.simple()),
            display_name: Some("Ada".to_owned()),
            role: Some(Role::User),
            is_active: None,
        },
    )
    .await?;
    let mut client = events(&db.pool).await?;
    let mut stream = client
        .subscribe(subscribe(Uuid::now_v7(), Role::Admin))
        .await?
        .into_inner();

    notify(&db.pool, "not-a-user-id").await?;
    notify(&db.pool, &id.to_string()).await?;
    let event = tokio::time::timeout(PATIENCE, stream.next())
        .await?
        .ok_or("the stream ended")??;
    assert_eq!(
        event.user_id,
        id.to_string(),
        "a payload naming no user is skipped"
    );
    let got = event.user.ok_or("the event holds no user")?;
    assert_eq!(got.email, user.email);
    assert_eq!(got.display_name.as_deref(), Some("Ada"));

    User::destroy(&db.pool, user.id).await?;
    UserEventsService::new(db.pool.clone())
        .publish(user.id)
        .await?;
    let event = tokio::time::timeout(PATIENCE, stream.next())
        .await?
        .ok_or("the stream ended")??;
    assert_eq!(event.user_id, id.to_string());
    assert!(event.user.is_none(), "a deleted user was sent: {event:?}");

    drop(stream);
    db.drop().await
}

/// Through the real caller check (`app::authenticate`), an admin's `read`
/// API key subscribes: `Subscribe` is a read (`app/auth.rs`'s `READS`).
#[tokio::test]
async fn a_read_api_key_subscribes() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let _alone = ALONE.lock().await;
    let id = Uuid::now_v7();
    let admin = User::create(
        &db.pool,
        NewUser {
            id: Some(id.into()),
            email: format!("admin-{}@test.invalid", id.simple()),
            display_name: None,
            role: Some(Role::Admin),
            is_active: None,
        },
    )
    .await?;
    let key = NewApiKey::generate();
    insert_api_key(
        &db.pool,
        InsertApiKeyParams {
            id: Uuid::now_v7().into(),
            user_id: admin.id,
            name: "a dashboard".to_owned(),
            key_prefix: key.prefix,
            key_hash: key.hash,
            scope: ApiKeyScope::Read,
            expires_at: None,
        },
    )
    .await?;
    let routes = app::routes(&state(&db.pool)?, HealthReporter::new())?;
    let mut client = UserEventsServiceClient::new(channel(routes).await?);
    let mut request = tonic::Request::new(proto::SubscribeUserEventsRequest {});
    request.metadata_mut().insert("x-api-key", key.raw.parse()?);
    let mut stream = client.subscribe(request).await?.into_inner();

    notify(&db.pool, &id.to_string()).await?;
    let event = tokio::time::timeout(PATIENCE, stream.next())
        .await?
        .ok_or("the stream ended")??;
    assert_eq!(event.user_id, id.to_string());

    drop(stream);
    db.drop().await
}
