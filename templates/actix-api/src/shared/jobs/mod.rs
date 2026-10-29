// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Background work on PostgreSQL, with no business meaning: a [`Job`] is a
//! typed payload and what to do with it; the [`Queue`] writes one on the
//! flow's own transaction, so a rolled-back flow enqueues nothing; a
//! [`Worker`] claims due jobs (`FOR UPDATE SKIP LOCKED`, so two workers
//! never run the same one), runs each, and deletes it, or retries it with
//! backoff until it is `dead` after [`Job::MAX_ATTEMPTS`].
//!
//! Delivery is at least once: a crash after a job ran but before its claim
//! committed runs it again, so make a job safe to repeat. A running job
//! holds its claim (one pool connection) until it settles, so keep
//! `JOBS_CONCURRENCY` well under the pool's size.
//!
//! Adding a job is a payload struct, its `impl Job`, one
//! [`Worker::register`] in `app/run.rs`, and a [`Queue::enqueue`] on the
//! flow's transaction:
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! pub struct SendInvoice { pub order: OrderId }
//!
//! #[async_trait]
//! impl Job for SendInvoice {
//!     const KIND: &'static str = "billing.send_invoice";
//!     type Context = BillingService;
//!     async fn run(self, billing: &BillingService) -> Result<(), JobError> {
//!         billing.send_invoice(self.order).await?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Framework-free, so both templates share it.

pub mod queue;
pub mod worker;

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub use queue::{Enqueue, Queue};
pub use worker::{Ran, Worker};

/// A kind of background work: its payload is `Self`, stored as JSON until
/// a worker runs it with its [`Context`](Job::Context).
#[async_trait]
pub trait Job: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Names the kind in the `jobs` table and the metrics, such as
    /// `mail.send`. Never rename one with jobs still queued.
    const KIND: &'static str;

    /// Attempts before the job is `dead`.
    const MAX_ATTEMPTS: i32 = 5;

    /// How long one attempt may run before it counts as failed.
    const TIMEOUT: Duration = Duration::from_secs(5 * 60);

    /// What a run needs (a service, a port), given once to
    /// [`Worker::register`].
    type Context: Send + Sync + 'static;

    /// Does the work. An error is logged and retried, with backoff.
    async fn run(self, context: &Self::Context) -> Result<(), JobError>;

    /// The payload a `dead` job keeps, from the one it had: strip what must
    /// not outlive the attempts (mail drops its bodies and their links).
    /// It keeps everything by default.
    #[must_use]
    fn on_dead(payload: serde_json::Value) -> serde_json::Value {
        payload
    }
}

/// Why an attempt failed: it goes to the log and the job's `last_error`,
/// never to an API client. Any error converts with `?`.
#[derive(Debug)]
pub struct JobError(String);

impl JobError {
    /// An error saying `message`.
    #[must_use]
    pub fn new(message: impl fmt::Display) -> Self {
        JobError(message.to_string())
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<E: std::error::Error> From<E> for JobError {
    fn from(error: E) -> Self {
        JobError(error.to_string())
    }
}
