// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! `JobService`: what an admin sees and does with the job queue, through
//! this feature's query module, `db::queries::jobs`. Running jobs is
//! `shared/jobs/`'s; this is the window onto it.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgConnection, PgPool};

use super::dto::{JobResponse, ListJobsQuery};
use crate::db::jobs::models::JobId;
use crate::db::queries::jobs::{self as queries, ListJobsParams};
use crate::db::queries::queue;
use crate::db::users::models::UserId;
use crate::error::AppError;
use crate::shared::jobs::Queue;
use crate::shared::response::PaginatedResponse;

/// What an admin did to a job, for the audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobAuditEntry {
    /// The admin.
    pub actor: UserId,
    /// `job.retried` or `job.cancelled`.
    pub action: &'static str,
    /// The job.
    pub job: JobId,
    /// Its kind.
    pub kind: String,
}

/// The port this feature records what admins do through, on the change's
/// own connection so the entry commits with it. `app/ports.rs` implements
/// it onto the `audit_log` feature.
#[async_trait]
pub trait JobAudit: Send + Sync {
    /// Records `entry` on `conn`.
    async fn record_in(
        &self,
        conn: &mut PgConnection,
        entry: JobAuditEntry,
    ) -> Result<(), AppError>;
}

/// The `jobs` service, built by `app/`.
pub struct JobService {
    /// The pool the service reads and writes through.
    pub pool: PgPool,
    /// Wakes the workers when a job is retried.
    pub queue: Queue,
    /// Where retries and cancellations are recorded.
    pub audit: Arc<dyn JobAudit>,
}

impl JobService {
    /// One page of the jobs `query` asks for, newest first.
    pub async fn list(
        &self,
        query: &ListJobsQuery,
    ) -> Result<PaginatedResponse<JobResponse>, AppError> {
        let pagination = query.pagination();
        let status = query.status_filter();
        let total =
            queries::count_listed_jobs(&self.pool, status.as_deref(), query.kind.as_deref())
                .await?;
        let rows = queries::list_jobs(
            &self.pool,
            ListJobsParams {
                status,
                kind: query.kind.clone(),
                limit: pagination.limit(),
                offset: pagination.offset(),
            },
        )
        .await?;
        let items = rows.into_iter().map(JobResponse::from).collect();
        Ok(PaginatedResponse::new(items, pagination, total))
    }

    /// Puts the dead job `id` back in the queue, due now with its attempts
    /// at zero, for `actor`; `404` when `id` is no dead job.
    pub async fn retry(&self, actor: UserId, id: JobId) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        let kind = queries::retry_dead_job(&mut *tx, id)
            .await?
            .ok_or_else(|| AppError::NotFound("no dead job has this id".to_owned()))?;
        self.audit
            .record_in(
                &mut tx,
                JobAuditEntry {
                    actor,
                    action: "job.retried",
                    job: id,
                    kind,
                },
            )
            .await?;
        // Workers in other processes wake on the commit; this one's now.
        queue::notify_jobs(&mut *tx).await?;
        self.queue.commit(tx).await
    }

    /// Deletes the job `id`, pending or dead, for `actor`: `409` while a
    /// worker runs it (retry once it settles), `404` when there is none.
    pub async fn cancel(&self, actor: UserId, id: JobId) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await?;
        let Some(kind) = queries::cancel_job(&mut *tx, id).await? else {
            return Err(if queries::count_job(&mut *tx, id).await? > 0 {
                AppError::Conflict(
                    "a worker is running this job; try again once it settles".to_owned(),
                )
            } else {
                AppError::NotFound("no job has this id".to_owned())
            });
        };
        self.audit
            .record_in(
                &mut tx,
                JobAuditEntry {
                    actor,
                    action: "job.cancelled",
                    job: id,
                    kind,
                },
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
