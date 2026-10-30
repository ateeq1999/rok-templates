// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Schedule`]: jobs enqueued on a cron pattern, once per tick however
//! many processes run the scheduler.
//!
//! Every process running jobs runs it. At a tick, each one tries to move
//! its schedule's `last_tick` (`job_schedules`) forward to that tick; the
//! row lock lets exactly one transaction succeed, and that one enqueues
//! the tick's job, so a tick never runs twice. A tick missed while nothing
//! ran (a deploy, an outage) runs once when a process comes back, not once
//! per missed tick; a schedule seen for the first time waits for its next
//! tick rather than running at once. Times are UTC.

use std::fmt;
use std::future::Future;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use croner::Cron;
use sqlx::PgPool;

use super::{Enqueue, Job, Queue};
use crate::db::queries::queue as queries;
use crate::error::AppError;

/// The longest the scheduler sleeps between two looks at the clock.
const MAX_SLEEP: Duration = Duration::from_secs(60);

/// A scheduled kind: its pattern, and the payload each tick enqueues.
struct Entry {
    kind: &'static str,
    pattern: String,
    cron: Cron,
    payload: serde_json::Value,
}

/// A cron pattern that doesn't parse, named by the kind it schedules.
#[derive(Debug)]
pub struct ScheduleError {
    kind: &'static str,
    pattern: String,
    reason: String,
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the schedule of `{}`, `{}`, is not a cron pattern: {}",
            self.kind, self.pattern, self.reason
        )
    }
}

impl std::error::Error for ScheduleError {}

/// The jobs enqueued on a clock, by cron pattern (`minute hour day month
/// weekday`, UTC): `Schedule::new().cron::<SweepUploads>("*/5 * * * *")?`.
#[derive(Default)]
pub struct Schedule {
    entries: Vec<Entry>,
}

impl Schedule {
    /// A schedule of nothing yet.
    #[must_use]
    pub fn new() -> Self {
        Schedule::default()
    }

    /// This schedule, enqueueing `J::default()` at every tick of `pattern`,
    /// such as `"*/5 * * * *"` (every 5 minutes) or `"0 3 * * *"` (03:00
    /// UTC daily). A pattern that doesn't parse is an error, so a typo
    /// stops the server at start-up rather than never running.
    pub fn cron<J: Job + Default>(mut self, pattern: &str) -> Result<Self, ScheduleError> {
        let wrong = |reason: String| ScheduleError {
            kind: J::KIND,
            pattern: pattern.to_owned(),
            reason,
        };
        let cron = Cron::from_str(pattern).map_err(|err| wrong(err.to_string()))?;
        let payload = serde_json::to_value(J::default()).map_err(|err| wrong(err.to_string()))?;
        self.entries.push(Entry {
            kind: J::KIND,
            pattern: pattern.to_owned(),
            cron,
            payload,
        });
        Ok(self)
    }

    /// Enqueues, on `queue` in `pool`, the job of every schedule whose
    /// latest tick at `now` no process claimed yet: how many it enqueued.
    /// What the running scheduler calls each time it wakes; a test calls it
    /// with the clock it wants.
    pub async fn tick_at(
        &self,
        pool: &PgPool,
        queue: &Queue,
        now: DateTime<Utc>,
    ) -> Result<usize, AppError> {
        let mut enqueued = 0;
        for entry in &self.entries {
            let Ok(tick) = entry.cron.find_previous_occurrence(&now, true) else {
                continue;
            };
            let kind = entry.kind.to_owned().into();
            let mut tx = pool.begin().await?;
            queries::start_schedule(&mut *tx, &kind, tick).await?;
            if queries::claim_tick(&mut *tx, &kind, tick).await? > 0 {
                queue
                    .enqueue_value(
                        &mut tx,
                        entry.kind,
                        entry.payload.clone(),
                        Enqueue::default(),
                    )
                    .await?;
                tracing::info!(kind = entry.kind, %tick, "scheduled job enqueued");
                enqueued += 1;
            }
            queue.commit(tx).await?;
        }
        Ok(enqueued)
    }

    /// Enqueues each tick's jobs until `shutdown` completes, sleeping until
    /// the next tick (a minute at most). A database error is logged; the
    /// next wake tries again.
    pub async fn run(self, pool: PgPool, queue: Queue, shutdown: impl Future<Output = ()>) {
        if self.entries.is_empty() {
            return;
        }
        for entry in &self.entries {
            tracing::info!(kind = entry.kind, pattern = %entry.pattern, "scheduled");
        }
        let mut shutdown = std::pin::pin!(shutdown);
        loop {
            let now = Utc::now();
            if let Err(err) = self.tick_at(&pool, &queue, now).await {
                tracing::error!(error = %err, "the scheduler could not enqueue its jobs");
            }
            tokio::select! {
                biased;
                () = &mut shutdown => return,
                () = tokio::time::sleep(self.until_next_tick(now)) => {}
            }
        }
    }

    /// How long from `now` until the earliest next tick, a minute at most.
    fn until_next_tick(&self, now: DateTime<Utc>) -> Duration {
        self.entries
            .iter()
            .filter_map(|entry| entry.cron.find_next_occurrence(&now, false).ok())
            .min()
            .and_then(|next| (next - now).to_std().ok())
            .map_or(MAX_SLEEP, |wait| wait.min(MAX_SLEEP))
    }
}
