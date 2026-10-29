// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Worker`]: claims due jobs and runs them, several at once.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::postgres::PgListener;
use sqlx::{PgConnection, PgPool};
use tokio::sync::{Notify, watch};
use tokio::task::JoinSet;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::{Job, JobError, Queue};
use crate::db::queries::queue::{self as queries, BuryJobParams, ClaimDueJobRow, RetryJobParams};
use crate::error::AppError;
use crate::shared::{metrics, telemetry};

/// How often an idle worker looks for due jobs when nothing wakes it.
const TICK: Duration = Duration::from_secs(5);

/// The wait after a first failure; it doubles with each further one (30s,
/// 1m, 2m, 4m).
const DEFAULT_BACKOFF: Duration = Duration::from_secs(30);

/// The channel enqueuing notifies (`queries/queue.sql`'s `NotifyJobs`).
const CHANNEL: &str = "jobs";

/// One attempt's future.
type Attempt = Pin<Box<dyn Future<Output = Result<(), JobError>> + Send>>;

/// A registered kind, its type erased.
struct Handler {
    run: Box<dyn Fn(serde_json::Value) -> Attempt + Send + Sync>,
    max_attempts: i32,
    timeout: Duration,
    on_dead: fn(serde_json::Value) -> serde_json::Value,
}

/// What [`Worker::run_due`] did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ran {
    /// Jobs that ran and were deleted.
    pub done: usize,
    /// Attempts that failed: the job retries later, or is `dead`.
    pub failed: usize,
}

/// Runs the jobs of the kinds [`register`](Self::register)ed.
pub struct Worker {
    pool: PgPool,
    handlers: HashMap<&'static str, Handler>,
    wake: Arc<Notify>,
    concurrency: usize,
    backoff: Duration,
}

impl Worker {
    /// A worker over `pool`, woken by `queue`, running one job at a time
    /// until [`with_concurrency`](Self::with_concurrency) says more.
    #[must_use]
    pub fn new(pool: PgPool, queue: &Queue) -> Self {
        Worker {
            pool,
            handlers: HashMap::new(),
            wake: queue.waker(),
            concurrency: 1,
            backoff: DEFAULT_BACKOFF,
        }
    }

    /// This worker, running jobs of kind `J` with `context`.
    #[must_use]
    pub fn register<J: Job>(mut self, context: J::Context) -> Self {
        let context = Arc::new(context);
        let run = move |payload: serde_json::Value| -> Attempt {
            let context = Arc::clone(&context);
            Box::pin(async move {
                let job: J = serde_json::from_value(payload)
                    .map_err(|err| JobError::new(format!("unreadable payload: {err}")))?;
                job.run(&context).await
            })
        };
        self.handlers.insert(
            J::KIND,
            Handler {
                run: Box::new(run),
                max_attempts: J::MAX_ATTEMPTS,
                timeout: J::TIMEOUT,
                on_dead: J::on_dead,
            },
        );
        self
    }

    /// This worker, running up to `concurrency` jobs at once
    /// (`JOBS_CONCURRENCY`); at least one.
    #[must_use]
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    /// The wait after a first failure instead of 30 seconds; it doubles
    /// with each further failure.
    #[must_use]
    pub fn with_backoff(mut self, backoff: Duration) -> Self {
        self.backoff = backoff;
        self
    }

    /// Runs jobs until `shutdown` completes: whenever a job is enqueued
    /// (in this process, or `NOTIFY`ed by another), and every 5 seconds.
    /// The jobs running then finish first.
    pub async fn run(self, shutdown: impl Future<Output = ()>) {
        let worker = Arc::new(self);
        let (stop, stopped) = watch::channel(false);
        // Both sets abort their tasks when dropped: abandoning this future
        // (`Background::stop_within` past its grace) abandons the running
        // jobs, whose transactions roll back and release their claims.
        let mut listening = JoinSet::new();
        listening.spawn(listen(worker.pool.clone(), Arc::clone(&worker.wake)));
        let mut slots = JoinSet::new();
        for _ in 0..worker.concurrency {
            slots.spawn(Arc::clone(&worker).slot(stopped.clone()));
        }
        shutdown.await;
        let _ = stop.send(true);
        while let Some(slot) = slots.join_next().await {
            if let Err(err) = slot {
                tracing::error!(error = %err, "a job worker failed");
            }
        }
    }

    /// One of [`run`](Self::run)'s slots: runs due jobs one after another,
    /// then waits for a wake, a tick or the stop.
    async fn slot(self: Arc<Self>, mut stopped: watch::Receiver<bool>) {
        loop {
            let woken = self.wake.notified();
            tokio::pin!(woken);
            woken.as_mut().enable();
            loop {
                if *stopped.borrow() {
                    return;
                }
                match self.run_one().await {
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(err) => {
                        tracing::error!(error = %err, "the job worker could not read the queue");
                        break;
                    }
                }
            }
            tokio::select! {
                biased;
                _ = stopped.changed() => return,
                () = &mut woken => {}
                () = tokio::time::sleep(TICK) => {}
            }
        }
    }

    /// Runs due jobs, one after another, until none is due: what a test
    /// calls instead of [`run`](Self::run). A job that fails and is due
    /// again at once (no backoff) runs again, until it is `dead`.
    pub async fn run_due(&self) -> Result<Ran, AppError> {
        let mut ran = Ran::default();
        while let Some(succeeded) = self.run_one().await? {
            if succeeded {
                ran.done += 1;
            } else {
                ran.failed += 1;
            }
        }
        Ok(ran)
    }

    /// Claims the due job waiting longest, runs it and settles it, in one
    /// transaction: whether it succeeded, or `None` when none is due.
    pub async fn run_one(&self) -> Result<Option<bool>, AppError> {
        let mut tx = self.pool.begin().await?;
        let Some(job) = queries::claim_due_job(&mut *tx).await? else {
            tx.commit().await?;
            return Ok(None);
        };
        let span = tracing::info_span!(
            "job",
            kind = %job.kind,
            id = %job.id,
            attempt = job.attempts + 1,
        );
        // Without an exporter there is no trace to join: nothing to set.
        let _ = span.set_parent(telemetry::remote_parent(job.traceparent.as_deref(), None));
        let started = Instant::now();
        let outcome = self.attempt(&job).instrument(span.clone()).await;
        let succeeded = outcome.is_ok();
        self.settle(&mut tx, &job, outcome).instrument(span).await?;
        tx.commit().await?;
        metrics::record_job(&job.kind, succeeded, started.elapsed());
        Ok(Some(succeeded))
    }

    /// One attempt at `job`, within its kind's timeout.
    async fn attempt(&self, job: &ClaimDueJobRow) -> Result<(), JobError> {
        let Some(handler) = self.handlers.get(job.kind.as_str()) else {
            return Err(JobError::new(format!(
                "no worker runs jobs of kind `{}`",
                job.kind
            )));
        };
        match tokio::time::timeout(handler.timeout, (handler.run)(job.payload.clone())).await {
            Ok(outcome) => outcome,
            Err(_) => Err(JobError::new(format!(
                "the attempt took longer than {:?}",
                handler.timeout
            ))),
        }
    }

    /// Deletes a job that ran; counts a failed attempt: a retry after the
    /// backoff, or `dead` after its kind's last attempt (at once for a kind
    /// no worker runs), its payload as its `on_dead` leaves it.
    async fn settle(
        &self,
        conn: &mut PgConnection,
        job: &ClaimDueJobRow,
        outcome: Result<(), JobError>,
    ) -> Result<(), AppError> {
        let Err(err) = outcome else {
            queries::delete_job(conn, job.id).await?;
            return Ok(());
        };
        let attempts = job.attempts + 1;
        let handler = self.handlers.get(job.kind.as_str());
        if handler.is_none_or(|handler| attempts >= handler.max_attempts) {
            tracing::error!(attempts, error = %err, "job dead after its last attempt");
            let payload = handler.map_or_else(
                || job.payload.clone(),
                |handler| (handler.on_dead)(job.payload.clone()),
            );
            queries::bury_job(
                conn,
                BuryJobParams {
                    id: job.id,
                    attempts,
                    last_error: Some(err.to_string()),
                    payload,
                },
            )
            .await?;
        } else {
            tracing::warn!(attempts, error = %err, "job failed; will retry");
            let retry_in = self.backoff * 2u32.pow(u32::try_from(attempts - 1).unwrap_or(0));
            queries::retry_job(
                conn,
                RetryJobParams {
                    id: job.id,
                    attempts,
                    last_error: Some(err.to_string()),
                    retry_in_secs: retry_in.as_secs_f64(),
                },
            )
            .await?;
        }
        Ok(())
    }
}

/// Wakes the workers on every `NOTIFY jobs`, so a job another process
/// enqueued runs now. Without the listener they still find it at the next
/// tick; a lost connection is reconnected by `PgListener`.
async fn listen(pool: PgPool, wake: Arc<Notify>) {
    let mut listener = match PgListener::connect_with(&pool).await {
        Ok(listener) => listener,
        Err(err) => {
            tracing::warn!(error = %err, "jobs: no NOTIFY listener; polling only");
            return;
        }
    };
    if let Err(err) = listener.listen(CHANNEL).await {
        tracing::warn!(error = %err, "jobs: could not LISTEN; polling only");
        return;
    }
    loop {
        match listener.recv().await {
            Ok(_) => wake.notify_waiters(),
            Err(err) => {
                tracing::warn!(error = %err, "jobs: the NOTIFY listener failed; polling only");
                return;
            }
        }
    }
}
