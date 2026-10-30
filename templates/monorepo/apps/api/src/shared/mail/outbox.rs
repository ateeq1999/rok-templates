// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The mail outbox: a flow writes its mail as a `mail.send` job
//! ([`SendMail`]) on its own transaction ([`Outbox::enqueue`]), so a
//! rolled-back flow sends nothing, and commits through
//! [`Outbox::commit`], which wakes the workers. A worker sends it through
//! the [`Mailer`] it was registered with, and deletes it; a failure is
//! retried with backoff, and after [`MAX_ATTEMPTS`] the job is `dead`,
//! keeping the address, the subject and the error but not the bodies.
//!
//! Delivery is at least once: a crash after a send but before the claim
//! commits sends that mail again.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Postgres, Transaction};

use super::{Mail, Mailer};
use crate::error::AppError;
use crate::shared::jobs::{Job, JobError, Queue};

/// Failed attempts after which a mail is `dead`: kept with its address,
/// subject and `last_error` (its bodies are dropped), never tried again.
pub const MAX_ATTEMPTS: i32 = 5;

/// What a dead mail's bodies are replaced with.
const REMOVED: &str = "(removed: never sent)";

/// The job sending one [`Mail`]: `mail.send`, run with the [`Mailer`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SendMail(pub Mail);

#[async_trait]
impl Job for SendMail {
    const KIND: &'static str = "mail.send";
    const MAX_ATTEMPTS: i32 = MAX_ATTEMPTS;
    type Context = Arc<dyn Mailer>;

    async fn run(self, mailer: &Arc<dyn Mailer>) -> Result<(), JobError> {
        mailer.send(&self.0).await?;
        Ok(())
    }

    /// A dead mail's bodies are dropped, since a link they carry must not
    /// outlive the attempts to send it.
    fn on_dead(mut payload: serde_json::Value) -> serde_json::Value {
        for body in ["text", "html"] {
            if let Some(value) = payload.get_mut(body) {
                *value = serde_json::Value::from(REMOVED);
            }
        }
        payload
    }
}

/// Where flows put their mail: the job [`Queue`]. Cheap to clone: every
/// clone wakes the same workers.
#[derive(Clone, Default)]
pub struct Outbox {
    queue: Queue,
}

impl Outbox {
    /// An outbox over a queue of its own.
    #[must_use]
    pub fn new() -> Self {
        Outbox::default()
    }

    /// An outbox enqueuing on `queue`.
    #[must_use]
    pub fn on(queue: Queue) -> Self {
        Outbox { queue }
    }

    /// The queue mail goes to: what the worker sending it wakes on.
    #[must_use]
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Writes `mail` as a `mail.send` job on `conn`, the flow's transaction
    /// (`&mut tx`). It is sent only once that transaction commits.
    pub async fn enqueue(&self, conn: &mut PgConnection, mail: &Mail) -> Result<(), AppError> {
        self.queue.enqueue(conn, &SendMail(mail.clone())).await
    }

    /// Commits the flow's transaction `tx`, then wakes the workers so its
    /// mail leaves now rather than at the next tick.
    pub async fn commit(&self, tx: Transaction<'_, Postgres>) -> Result<(), AppError> {
        self.queue.commit(tx).await
    }

    /// Wakes the workers: call it after committing a transaction that
    /// enqueued mail without [`commit`](Self::commit).
    pub fn wake(&self) {
        self.queue.wake();
    }
}
