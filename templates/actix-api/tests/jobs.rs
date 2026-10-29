//! The job queue: a job enqueued on a flow's transaction runs once it
//! commits, exactly one worker runs it, a failure retries with backoff
//! until the job is `dead`, and a running worker wakes on a commit and
//! stops on shutdown. Run with `run_due`, without sleeping, but for the
//! running worker's own test.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix_api::shared::jobs::{Enqueue, Job, JobError, Queue, Worker};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use support::{Database, TestResult};

/// What the test jobs did: the labels they ran with, in order.
type Log = Arc<Mutex<Vec<String>>>;

/// A job that records its label, or fails when told to.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    label: String,
    fail: bool,
}

#[async_trait]
impl Job for Record {
    const KIND: &'static str = "test.record";
    const MAX_ATTEMPTS: i32 = 3;
    type Context = Log;

    async fn run(self, log: &Log) -> Result<(), JobError> {
        if self.fail {
            return Err(JobError::new(format!("{} failed", self.label)));
        }
        tokio::time::sleep(Duration::from_millis(2)).await;
        log.lock()
            .map_err(|_| JobError::new("the log is poisoned"))?
            .push(self.label);
        Ok(())
    }
}

/// A job that runs past its timeout.
#[derive(Debug, Serialize, Deserialize)]
struct Hang;

#[async_trait]
impl Job for Hang {
    const KIND: &'static str = "test.hang";
    const MAX_ATTEMPTS: i32 = 1;
    const TIMEOUT: Duration = Duration::from_millis(50);
    type Context = ();

    async fn run(self, (): &()) -> Result<(), JobError> {
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(())
    }
}

/// A worker over `queue` running `Record` and `Hang`, retrying at once.
fn worker(pool: &sqlx::PgPool, queue: &Queue, log: &Log) -> Worker {
    Worker::new(pool.clone(), queue)
        .with_backoff(Duration::ZERO)
        .register::<Record>(Arc::clone(log))
        .register::<Hang>(())
}

/// A `Record` job labelled `label`.
fn record(label: &str) -> Record {
    Record {
        label: label.to_owned(),
        fail: false,
    }
}

/// Enqueues `job` on its own transaction, with `options`.
async fn enqueue<J: Job>(
    pool: &sqlx::PgPool,
    queue: &Queue,
    job: &J,
    options: Enqueue,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;
    let written = queue.enqueue_with(&mut tx, job, options).await?;
    queue.commit(tx).await?;
    Ok(written)
}

/// Every job's kind, status, attempts and last error, oldest first.
async fn jobs(
    pool: &sqlx::PgPool,
) -> Result<Vec<(String, String, i32, Option<String>)>, sqlx::Error> {
    sqlx::query_as("SELECT kind, status, attempts, last_error FROM jobs ORDER BY created_at, id")
        .fetch_all(pool)
        .await
}

fn logged(log: &Log) -> Vec<String> {
    log.lock().map(|log| log.clone()).unwrap_or_default()
}

#[actix_web::test]
async fn a_job_runs_once_then_is_deleted() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    enqueue(&db.pool, &queue, &record("a"), Enqueue::default()).await?;
    let ran = worker(&db.pool, &queue, &log).run_due().await?;
    assert_eq!((ran.done, ran.failed), (1, 0));
    assert_eq!(logged(&log), ["a"]);
    assert!(jobs(&db.pool).await?.is_empty());
    db.drop().await
}

#[actix_web::test]
async fn a_rolled_back_transaction_enqueues_nothing() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    let mut tx = db.pool.begin().await?;
    queue.enqueue(&mut tx, &record("never")).await?;
    tx.rollback().await?;
    assert_eq!(worker(&db.pool, &queue, &log).run_due().await?.done, 0);
    assert!(logged(&log).is_empty());
    db.drop().await
}

/// A failing job retries until its kind's last attempt, then is `dead`
/// with its error; with the default backoff it waits before its second.
#[actix_web::test]
async fn a_failing_job_backs_off_then_is_dead() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    let failing = Record {
        label: "b".to_owned(),
        fail: true,
    };
    enqueue(&db.pool, &queue, &failing, Enqueue::default()).await?;
    let patient = Worker::new(db.pool.clone(), &queue).register::<Record>(Arc::clone(&log));
    assert_eq!(patient.run_due().await?.failed, 1);
    let waits: bool = sqlx::query_scalar("SELECT run_at > now() FROM jobs")
        .fetch_one(&db.pool)
        .await?;
    assert!(waits, "a failed job waits for its retry");
    sqlx::query("UPDATE jobs SET run_at = now()")
        .execute(&db.pool)
        .await?;
    assert_eq!(worker(&db.pool, &queue, &log).run_due().await?.failed, 2);
    let rows = jobs(&db.pool).await?;
    assert_eq!(rows.len(), 1);
    let (kind, status, attempts, last_error) = &rows[0];
    assert_eq!(
        (kind.as_str(), status.as_str(), *attempts),
        ("test.record", "dead", 3)
    );
    assert_eq!(last_error.as_deref(), Some("b failed"));
    db.drop().await
}

/// A job of a kind no worker runs, and one past its timeout, fail too.
#[actix_web::test]
async fn an_unknown_kind_and_a_timed_out_attempt_fail() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    enqueue(&db.pool, &queue, &Hang, Enqueue::default()).await?;
    let nobody = Worker::new(db.pool.clone(), &queue);
    assert_eq!(nobody.run_due().await?.failed, 1);
    let rows = jobs(&db.pool).await?;
    assert_eq!(rows[0].1, "dead");
    assert!(
        rows[0]
            .3
            .as_deref()
            .is_some_and(|err| err.contains("no worker runs")),
        "{rows:?}"
    );
    sqlx::query("DELETE FROM jobs").execute(&db.pool).await?;
    enqueue(&db.pool, &queue, &Hang, Enqueue::default()).await?;
    assert_eq!(worker(&db.pool, &queue, &log).run_due().await?.failed, 1);
    let rows = jobs(&db.pool).await?;
    assert!(
        rows[0]
            .3
            .as_deref()
            .is_some_and(|err| err.contains("took longer")),
        "{rows:?}"
    );
    db.drop().await
}

/// A unique key holds one job at a time; a delayed job is not due yet.
#[actix_web::test]
async fn a_unique_key_holds_one_job_and_a_delay_postpones_it() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    let once = || Enqueue {
        unique_key: Some("nightly-report".to_owned()),
        delay: Duration::from_secs(3600),
    };
    assert!(enqueue(&db.pool, &queue, &record("first"), once()).await?);
    assert!(!enqueue(&db.pool, &queue, &record("second"), once()).await?);
    assert_eq!(jobs(&db.pool).await?.len(), 1);
    assert_eq!(worker(&db.pool, &queue, &log).run_due().await?.done, 0);
    db.drop().await
}

/// Two workers draining one queue at once never run a job twice.
#[actix_web::test]
async fn two_workers_never_run_a_job_twice() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    for n in 0..40 {
        enqueue(
            &db.pool,
            &queue,
            &record(&n.to_string()),
            Enqueue::default(),
        )
        .await?;
    }
    let (first, second) = (
        worker(&db.pool, &queue, &log),
        worker(&db.pool, &queue, &log),
    );
    let (a, b) = tokio::join!(first.run_due(), second.run_due());
    assert_eq!(a?.done + b?.done, 40);
    let mut labels = logged(&log);
    labels.sort();
    labels.dedup();
    assert_eq!(labels.len(), 40, "a job ran twice");
    db.drop().await
}

/// The running worker wakes on a commit, well before its 5-second tick,
/// and its slots stop on shutdown.
#[actix_web::test]
async fn the_running_worker_wakes_on_commit_and_stops_on_shutdown() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, log) = (Queue::new(), Log::default());
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(
        worker(&db.pool, &queue, &log)
            .with_concurrency(2)
            .run(async {
                let _ = stopped.await;
            }),
    );
    // Let the worker reach its wait first: it must be woken, not find the
    // job on its first pass.
    tokio::time::sleep(Duration::from_millis(100)).await;
    enqueue(&db.pool, &queue, &record("woken"), Enqueue::default()).await?;
    let mut waited = Duration::ZERO;
    while logged(&log).is_empty() && waited < Duration::from_secs(2) {
        tokio::time::sleep(Duration::from_millis(20)).await;
        waited += Duration::from_millis(20);
    }
    assert_eq!(logged(&log), ["woken"]);
    let _ = stop.send(());
    tokio::time::timeout(Duration::from_secs(2), task).await??;
    db.drop().await
}
