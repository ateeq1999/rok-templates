// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Background`]: the worker and the scheduler, started together beside
//! the server and stopped together with it.

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
}
