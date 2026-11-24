//! A unique or foreign-key violation is a `409 conflict` that says which
//! constraint it broke, so a feature can answer a constraint its own way.

mod support;

use sqlx::PgPool;
use support::{Database, TestResult};
use uuid::Uuid;

use axum_api::error::{AppError, ConflictKind};

/// The error of inserting an account with `email`, or `None` when it saved.
async fn insert_user(pool: &PgPool, email: &str) -> Option<AppError> {
    sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
        .bind(Uuid::now_v7())
        .bind(email)
        .execute(pool)
        .await
        .err()
        .map(AppError::from)
}

#[tokio::test]
async fn a_unique_violation_is_a_conflict_naming_its_constraint() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    assert!(insert_user(&db.pool, "ada@test.invalid").await.is_none());
    let error = insert_user(&db.pool, "ADA@test.invalid")
        .await
        .ok_or("the second account with the email saved")?;
    assert!(matches!(
        error,
        AppError::Conflict {
            kind: ConflictKind::Unique,
            ..
        }
    ));
    assert_eq!(error.constraint(), Some("users_email_key"));
    assert_eq!(error.code(), "conflict");
    assert_eq!(
        error.to_string(),
        "the value is already taken (constraint `users_email_key`)"
    );
    db.drop().await
}

#[tokio::test]
async fn a_foreign_key_violation_is_a_conflict_naming_its_constraint() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let error =
        sqlx::query("INSERT INTO audit_log (id, actor_user_id, action) VALUES ($1, $2, 'x')")
            .bind(Uuid::now_v7())
            .bind(Uuid::now_v7())
            .execute(&db.pool)
            .await
            .err()
            .map(AppError::from)
            .ok_or("an entry by no account saved")?;
    assert!(matches!(
        error,
        AppError::Conflict {
            kind: ConflictKind::ForeignKey,
            ..
        }
    ));
    assert_eq!(error.constraint(), Some("audit_log_actor_user_id_fkey"));
    assert_eq!(
        error.to_string(),
        "the change breaks a reference (constraint `audit_log_actor_user_id_fkey`)"
    );
    db.drop().await
}

/// A conflict a feature raises itself, with no constraint behind it.
#[test]
fn a_conflict_with_the_current_state_names_no_constraint() {
    let error = AppError::conflict("a worker is running this job");
    assert!(matches!(
        error,
        AppError::Conflict {
            kind: ConflictKind::State,
            ..
        }
    ));
    assert_eq!(error.constraint(), None);
    assert_eq!(error.code(), "conflict");
    assert_eq!(error.to_string(), "a worker is running this job");
}
