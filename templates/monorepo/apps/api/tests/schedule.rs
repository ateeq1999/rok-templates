//! The schedule: a kind enqueued at every tick of its cron pattern, once
//! however many processes tick, its first tick waited for, a missed one
//! run once. Driven with `tick_at` and the clock each test wants.

mod support;

use std::time::Duration;

use async_trait::async_trait;
use axum_api::shared::jobs::{Job, JobError, Queue, Schedule};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use support::{Database, TestResult};

/// A job to schedule.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Report;

#[async_trait]
impl Job for Report {
    const KIND: &'static str = "test.report";
    type Context = ();

    async fn run(self, (): &()) -> Result<(), JobError> {
        Ok(())
    }
}

/// Every five minutes.
fn schedule() -> Result<Schedule, Box<dyn std::error::Error>> {
    Ok(Schedule::new().cron::<Report>("*/5 * * * *")?)
}

/// 2026-01-01 at `hour`:`minute`:`second` UTC.
fn at(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, hour, minute, second)
        .single()
        .unwrap_or_default()
}

async fn queued(pool: &sqlx::PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM jobs WHERE kind = 'test.report'")
        .fetch_one(pool)
        .await
}

/// A schedule seen for the first time waits for its next tick; that tick
/// enqueues once, and looking again within the same tick does nothing.
#[tokio::test]
async fn the_first_tick_is_waited_for_then_each_runs_once() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, schedule) = (Queue::new(), schedule()?);
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 2, 0)).await?, 0);
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 4, 59)).await?, 0);
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 5, 0)).await?, 1);
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 5, 30)).await?, 0);
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 10, 1)).await?, 1);
    assert_eq!(queued(&db.pool).await?, 2);
    db.drop().await
}

/// Ticks missed while nothing ran (an hour: twelve of them) run once.
#[tokio::test]
async fn missed_ticks_run_once() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let (queue, schedule) = (Queue::new(), schedule()?);
    schedule.tick_at(&db.pool, &queue, at(9, 0, 0)).await?;
    assert_eq!(schedule.tick_at(&db.pool, &queue, at(10, 1, 0)).await?, 1);
    assert_eq!(queued(&db.pool).await?, 1);
    db.drop().await
}

/// Eight replicas ticking at the same moment enqueue the tick once, even
/// once it ran and its job was deleted before the slowest one looked.
#[tokio::test]
async fn a_tick_runs_once_across_replicas() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let queue = Queue::new();
    schedule()?.tick_at(&db.pool, &queue, at(10, 0, 0)).await?;
    let replicas: Vec<_> = (0..8)
        .map(|_| {
            let (pool, queue) = (db.pool.clone(), queue.clone());
            tokio::spawn(async move {
                let schedule = Schedule::new().cron::<Report>("*/5 * * * *")?;
                schedule
                    .tick_at(&pool, &queue, at(10, 5, 0))
                    .await
                    .map_err(|err| err.to_string().into())
                    as Result<usize, Box<dyn std::error::Error + Send + Sync>>
            })
        })
        .collect();
    let mut enqueued = 0;
    for replica in replicas {
        enqueued += replica.await?.map_err(|err| err.to_string())?;
    }
    assert_eq!(enqueued, 1);
    // The job ran and is gone; a late replica still doesn't enqueue it.
    sqlx::query("DELETE FROM jobs").execute(&db.pool).await?;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(
        schedule()?.tick_at(&db.pool, &queue, at(10, 5, 2)).await?,
        0
    );
    db.drop().await
}

#[test]
fn a_pattern_that_does_not_parse_is_an_error() {
    let err = Schedule::new().cron::<Report>("every five minutes").err();
    assert!(
        err.is_some_and(|err| err.to_string().contains("`test.report`")),
        "a bad pattern was accepted"
    );
}
