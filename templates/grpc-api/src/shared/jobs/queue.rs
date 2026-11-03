// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Queue`]: where flows put their jobs, on their own transaction.

use std::sync::Arc;
use std::time::Duration;

use sqlx::{PgConnection, Postgres, Transaction};
use tokio::sync::Notify;

use super::Job;
use crate::db::jobs::models::JobId;
use crate::db::queries::queue::{self as queries, EnqueueJobParams};
use crate::error::AppError;
use crate::shared::telemetry;

/// Where flows enqueue jobs. Cheap to clone: every clone wakes the same
/// workers.
#[derive(Clone, Default)]
pub struct Queue {
    wake: Arc<Notify>,
}

/// When and how a job is enqueued, beyond "now": [`Queue::enqueue_with`].
#[derive(Debug, Clone, Default)]
pub struct Enqueue {
    /// How long from now the job is due; zero, now.
    pub delay: Duration,
    /// Set, no second job with the same key is enqueued while one exists
    /// (pending or dead): [`Queue::enqueue_with`] answers `false`.
    pub unique_key: Option<String>,
}

impl Queue {
    /// A queue whose workers are not woken yet.
    #[must_use]
    pub fn new() -> Self {
        Queue::default()
    }

    /// Writes `job`, due now, on `conn`, the flow's transaction (`&mut tx`).
    /// It runs only once that transaction commits.
    pub async fn enqueue<J: Job>(&self, conn: &mut PgConnection, job: &J) -> Result<(), AppError> {
        self.enqueue_with(conn, job, Enqueue::default()).await?;
        Ok(())
    }

    /// Writes `job` on `conn` as `options` say: whether it was written
    /// (`false`: its `unique_key` is taken). It carries the current trace,
    /// so its run continues it.
    pub async fn enqueue_with<J: Job>(
        &self,
        conn: &mut PgConnection,
        job: &J,
        options: Enqueue,
    ) -> Result<bool, AppError> {
        let payload = serde_json::to_value(job).map_err(|err| AppError::Internal(Box::new(err)))?;
        self.enqueue_value(conn, J::KIND, payload, options).await
    }

    /// [`enqueue_with`](Self::enqueue_with) for a payload already JSON: what
    /// the schedule enqueues its ticks with.
    pub(super) async fn enqueue_value(
        &self,
        conn: &mut PgConnection,
        kind: &str,
        payload: serde_json::Value,
        options: Enqueue,
    ) -> Result<bool, AppError> {
        let written = queries::enqueue_job(
            &mut *conn,
            EnqueueJobParams {
                id: JobId::generate(),
                kind: kind.to_owned(),
                payload,
                delay_secs: options.delay.as_secs_f64(),
                unique_key: options.unique_key,
                traceparent: telemetry::current_traceparent(),
            },
        )
        .await?;
        // Workers in other processes wake on the commit.
        queries::notify_jobs(conn).await?;
        Ok(written > 0)
    }

    /// Commits the flow's transaction `tx`, then wakes this process's
    /// workers, so its jobs run now rather than at the next tick.
    pub async fn commit(&self, tx: Transaction<'_, Postgres>) -> Result<(), AppError> {
        tx.commit().await?;
        self.wake();
        Ok(())
    }

    /// Wakes this process's workers: call it after committing a
    /// transaction that enqueued jobs without [`commit`](Self::commit).
    pub fn wake(&self) {
        self.wake.notify_waiters();
    }

    /// What workers wait on.
    pub(super) fn waker(&self) -> Arc<Notify> {
        Arc::clone(&self.wake)
    }
}
