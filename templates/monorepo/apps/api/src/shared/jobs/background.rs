// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Background`]: the worker and the scheduler, started together beside
//! the server and stopped together with it.

use std::time::Duration;

use sqlx::PgPool;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use super::{Queue, Schedule, Worker};

/// The worker and the scheduler, running (`tokio::spawn`).
pub struct Background {
    stop: watch::Sender<bool>,
    tasks: Vec<JoinHandle<()>>,
}

impl Background {
    /// Starts `worker`, and `schedule` enqueuing on `queue` in `pool`.
    #[must_use]
    pub fn start(worker: Worker, schedule: Schedule, pool: PgPool, queue: Queue) -> Self {
        let (stop, stopped) = watch::channel(false);
        let until_stopped = |mut stopped: watch::Receiver<bool>| async move {
            // A dropped sender stops them too.
            let _ = stopped.changed().await;
        };
        let tasks = vec![
            tokio::spawn(worker.run(until_stopped(stopped.clone()))),
            tokio::spawn(schedule.run(pool, queue, until_stopped(stopped))),
        ];
        Background { stop, tasks }
    }

    /// Stops the scheduler, and the worker once its running jobs settle.
    pub async fn stop(self) {
        let _ = self.stop.send(true);
        for task in self.tasks {
            if let Err(err) = task.await {
                tracing::error!(error = %err, "a background task failed");
            }
        }
    }

    /// [`stop`](Self::stop), but a job still running `grace` from now is
    /// abandoned: its transaction rolls back, so its claim is released and
    /// no attempt is counted, and it runs again on the next worker.
    pub async fn stop_within(self, grace: Duration) {
        let _ = self.stop.send(true);
        let deadline = tokio::time::Instant::now() + grace;
        for mut task in self.tasks {
            match tokio::time::timeout_at(deadline, &mut task).await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => tracing::error!(error = %err, "a background task failed"),
                Err(_) => {
                    tracing::warn!(
                        "jobs still running after the shutdown grace: abandoned, they run again"
                    );
                    task.abort();
                    let _ = task.await;
                }
            }
        }
    }
}
