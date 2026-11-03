//! The generated services, called through the generated clients over an
//! in-process channel, the caller put in place by the tests' stand-in for
//! the caller check: `users` gets, lists, updates and deletes, `audit_log`
//! gets and lists, and the statuses a client sees.
//!
//! The calls that stop before the data layer run over a pool that never
//! connects; the ones that reach it need `DATABASE_URL` and skip, with a
//! note, without it.

mod support;

use grpc_api::db::audit_log_entry::models::{AuditLogEntry, NewAuditLogEntry};
use grpc_api::db::types::Role;
use grpc_api::db::users::models::{NewUser, User};
use grpc_api::features::proto;
use grpc_api::features::proto::audit_log_entry_service_client::AuditLogEntryServiceClient;
use grpc_api::features::proto::users_service_client::UsersServiceClient;
use sqlx::PgPool;
use support::{Database, TestResult, as_caller, channel, offline, scoped};
use tonic::Code;
use tonic::transport::Channel;
use tonic_health::server::HealthReporter;
use uuid::Uuid;

/// A channel to the services over `pool`, behind the tests' caller check.
async fn services(pool: &PgPool) -> Result<Channel, Box<dyn std::error::Error>> {
    channel(scoped(pool, HealthReporter::new())?).await
}

/// A new account with `role`, made through the data layer (the sign-up
/// flows that make accounts stay on the HTTP API).
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

/// A `users` call by a caller who is not an admin is `PERMISSION_DENIED`:
/// the feature is admin-only.
#[tokio::test]
async fn a_caller_who_is_not_an_admin_is_permission_denied() -> TestResult {
    let mut users = UsersServiceClient::new(services(&offline()?).await?);
    let request = as_caller(
        proto::ListUsersRequest::default(),
        Uuid::now_v7(),
        Role::User,
    );
    let status = users
        .list(request)
        .await
        .err()
        .ok_or("a user listed users")?;
    assert_eq!(status.code(), Code::PermissionDenied, "{status:?}");
    Ok(())
}

/// A field that holds no value of its column's type is `INVALID_ARGUMENT`,
/// naming the field, before the data layer is reached.
#[tokio::test]
async fn a_malformed_id_is_invalid_argument() -> TestResult {
    let mut users = UsersServiceClient::new(services(&offline()?).await?);
    let request = as_caller(
        proto::GetUserRequest {
            id: "not-a-uuid".to_owned(),
        },
        Uuid::now_v7(),
        Role::Admin,
    );
    let status = users
        .get(request)
        .await
        .err()
        .ok_or("a malformed id was read")?;
    assert_eq!(status.code(), Code::InvalidArgument, "{status:?}");
    assert_eq!(status.message(), "`id` is not a UUID");
    Ok(())
}

/// An admin gets, lists (with the page's fields), updates and deletes an
/// account; a deleted one is `NOT_FOUND`.
#[tokio::test]
async fn an_admin_gets_lists_updates_and_deletes_users() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&db.pool, Role::Admin).await?;
    let user = account(&db.pool, Role::User).await?;
    let admin_id = Uuid::from(admin.id);
    let id = Uuid::from(user.id).to_string();
    let mut users = UsersServiceClient::new(services(&db.pool).await?);

    let got = users
        .get(as_caller(
            proto::GetUserRequest { id: id.clone() },
            admin_id,
            Role::Admin,
        ))
        .await?
        .into_inner();
    assert_eq!(got.id, id);
    assert_eq!(got.email, user.email);
    assert_eq!(got.display_name.as_deref(), Some("Ada"));
    assert_eq!(got.role(), proto::user::Role::User);

    let page = users
        .list(as_caller(
            proto::ListUsersRequest {
                page: Some(1),
                per_page: Some(1),
            },
            admin_id,
            Role::Admin,
        ))
        .await?
        .into_inner();
    assert_eq!((page.total, page.page, page.per_page), (2, 1, 1));
    assert_eq!(page.items.len(), 1);

    let updated = users
        .update(as_caller(
            proto::UpdateUserRequest {
                id: id.clone(),
                display_name: Some("  Grace  ".to_owned()),
                ..proto::UpdateUserRequest::default()
            },
            admin_id,
            Role::Admin,
        ))
        .await?
        .into_inner();
    assert_eq!(
        updated.display_name.as_deref(),
        Some("Grace"),
        "the update's rules trim the name"
    );
    assert_eq!(
        updated.email, user.email,
        "a field left out keeps its value"
    );

    users
        .delete(as_caller(
            proto::DeleteUserRequest { id: id.clone() },
            admin_id,
            Role::Admin,
        ))
        .await?;
    let status = users
        .get(as_caller(
            proto::GetUserRequest { id },
            admin_id,
            Role::Admin,
        ))
        .await
        .err()
        .ok_or("a deleted account was found")?;
    assert_eq!(status.code(), Code::NotFound, "{status:?}");
    db.drop().await
}

/// An admin gets and lists the audit log the application writes.
#[tokio::test]
async fn an_admin_reads_the_audit_log() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let admin = account(&db.pool, Role::Admin).await?;
    let entry = AuditLogEntry::create(
        &db.pool,
        NewAuditLogEntry {
            id: None,
            actor_user_id: admin.id,
            target_user_id: None,
            action: "user.disabled".to_owned(),
            details: None,
        },
    )
    .await?;
    let admin_id = Uuid::from(admin.id);
    let mut log = AuditLogEntryServiceClient::new(services(&db.pool).await?);

    let id = Uuid::from(entry.id).to_string();
    let got = log
        .get(as_caller(
            proto::GetAuditLogEntryRequest { id: id.clone() },
            admin_id,
            Role::Admin,
        ))
        .await?
        .into_inner();
    assert_eq!(got.id, id);
    assert_eq!(got.actor_user_id, admin_id.to_string());
    assert_eq!(got.action, "user.disabled");

    let page = log
        .list(as_caller(
            proto::ListAuditLogEntryRequest::default(),
            admin_id,
            Role::Admin,
        ))
        .await?
        .into_inner();
    assert_eq!(page.total, 1);
    assert_eq!(
        page.items.first().map(|item| item.id.as_str()),
        Some(id.as_str())
    );
    db.drop().await
}
