//! `/api/v1/jobs`, admins only: the queue listed by status and kind, a
//! dead job retried, a job cancelled (not while a worker runs it), each
//! retry and cancellation in the audit log.

mod support;

use axum::http::{Method, StatusCode};
use support::{Database, TestResult, admin, call, user};
use uuid::Uuid;

/// Writes a job of `kind` with `status` straight into the queue: its id.
async fn job(pool: &sqlx::PgPool, kind: &str, status: &str) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO jobs (id, kind, payload, status, attempts, last_error) \
         VALUES ($1, $2, '{}', $3, 5, 'it broke')",
    )
    .bind(id)
    .bind(kind)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(id)
}

/// The audit log's actions about the job `id`.
async fn audited(pool: &sqlx::PgPool, id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE details->>'job' = $1 ORDER BY created_at",
    )
    .bind(id.to_string())
    .fetch_all(pool)
    .await
}

#[tokio::test]
async fn an_admin_lists_the_jobs_by_status_and_kind() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    job(&db.pool, "mail.send", "dead").await?;
    job(&db.pool, "mail.send", "pending").await?;
    job(&db.pool, "report.build", "dead").await?;
    let token = admin(&db.pool).await?;
    let page = call(
        &db.pool,
        Some(&token),
        Method::GET,
        "/api/v1/jobs?status=dead",
        None,
    )
    .await?;
    assert_eq!(page.status, StatusCode::OK);
    assert_eq!(
        page.body["items"].as_array().map(Vec::len),
        Some(2),
        "{}",
        page.body
    );
    let one = call(
        &db.pool,
        Some(&token),
        Method::GET,
        "/api/v1/jobs?status=dead&kind=report.build",
        None,
    )
    .await?;
    let jobs = one.body["items"].as_array().cloned().unwrap_or_default();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["kind"], "report.build");
    assert_eq!(jobs[0]["status"], "dead");
    assert_eq!(jobs[0]["last_error"], "it broke");
    // The payload never leaves: a pending mail's holds its link.
    assert_eq!(jobs[0].get("payload"), None);
    db.drop().await
}

#[tokio::test]
async fn a_dead_job_is_retried_and_the_retry_audited() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let id = job(&db.pool, "mail.send", "dead").await?;
    let token = admin(&db.pool).await?;
    let retry = format!("/api/v1/jobs/{id}/retry");
    let answer = call(&db.pool, Some(&token), Method::POST, &retry, None).await?;
    assert_eq!(answer.status, StatusCode::NO_CONTENT);
    let (status, attempts): (String, i32) =
        sqlx::query_as("SELECT status, attempts FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&db.pool)
            .await?;
    assert_eq!((status.as_str(), attempts), ("pending", 0));
    assert_eq!(audited(&db.pool, id).await?, ["job.retried"]);
    // A pending job is no dead job to retry.
    let again = call(&db.pool, Some(&token), Method::POST, &retry, None).await?;
    assert_eq!(again.status, StatusCode::NOT_FOUND);
    db.drop().await
}

#[tokio::test]
async fn a_job_is_cancelled_but_not_while_a_worker_runs_it() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let token = admin(&db.pool).await?;
    // A worker holding its claim: the row is locked by another transaction.
    let running = job(&db.pool, "mail.send", "pending").await?;
    let mut worker = db.pool.begin().await?;
    sqlx::query("SELECT id FROM jobs WHERE id = $1 FOR UPDATE")
        .bind(running)
        .execute(&mut *worker)
        .await?;
    let path = format!("/api/v1/jobs/{running}");
    let busy = call(&db.pool, Some(&token), Method::DELETE, &path, None).await?;
    assert_eq!(busy.status, StatusCode::CONFLICT);
    worker.rollback().await?;
    let cancelled = call(&db.pool, Some(&token), Method::DELETE, &path, None).await?;
    assert_eq!(cancelled.status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs WHERE id = $1")
        .bind(running)
        .fetch_one(&db.pool)
        .await?;
    assert_eq!(left, 0);
    assert_eq!(audited(&db.pool, running).await?, ["job.cancelled"]);
    let gone = call(&db.pool, Some(&token), Method::DELETE, &path, None).await?;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    db.drop().await
}

#[tokio::test]
async fn the_queue_is_for_admins_only() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let id = job(&db.pool, "mail.send", "dead").await?;
    let token = user(&db.pool).await?;
    let list = call(&db.pool, Some(&token), Method::GET, "/api/v1/jobs", None).await?;
    assert_eq!(list.status, StatusCode::FORBIDDEN);
    let retry = format!("/api/v1/jobs/{id}/retry");
    let refused = call(&db.pool, Some(&token), Method::POST, &retry, None).await?;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let anonymous = call(&db.pool, None, Method::GET, "/api/v1/jobs", None).await?;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let unchanged: String = sqlx::query_scalar("SELECT status FROM jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&db.pool)
        .await?;
    assert_eq!(unchanged, "dead");
    db.drop().await
}
