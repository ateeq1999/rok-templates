// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Topics: live updates the API pushes to its signed-in clients, so a
//! screen refreshes on news instead of polling.
//!
//! A feature publishes a notice on its own transaction:
//!
//! ```text
//! let mut tx = pool.begin().await?;
//! // ... the change ...
//! live::publish(&mut tx, tenant, "orders.new", json!({"id": order.id})).await?;
//! tx.commit().await?;
//! ```
//!
//! [`publish`] is a `PostgreSQL` `NOTIFY` on the [`CHANNEL`], which the
//! database delivers only when the transaction commits: a rolled-back
//! change sends nothing. Every API instance listens with one connection
//! ([`Topics::start`]) and fans each notice out to its subscribers of the
//! notice's tenant and Topic ([`Topics::subscribe`]), through a buffer of
//! [`BUFFER`] notices: a subscriber that falls further behind is dropped,
//! and its client reconnects and refetches.
//!
//! A notice says what changed, not the new rows: its payload is small
//! (`NOTIFY` carries at most 8000 bytes), and a client refetches what it
//! shows. The `live` feature serves the subscriptions as server-sent
//! events, behind a guard and a short-lived [`Tickets`] ticket.

mod ticket;

use std::collections::HashSet;
use std::fmt::Display;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgListener;
use sqlx::{PgConnection, PgPool};
use tokio::sync::{broadcast, watch};

pub use ticket::{Ticket, Tickets};

use crate::error::AppError;

/// The `PostgreSQL` channel notices travel on.
pub const CHANNEL: &str = "live";

/// How many notices an instance holds for a subscriber that hasn't taken
/// them yet; one that falls further behind is dropped.
pub const BUFFER: usize = 256;

/// One piece of news on a Topic: what a subscriber receives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notice {
    /// The Topic, such as `orders.new`.
    pub topic: String,
    /// What changed, such as `{"id": "…"}`; `null` for none.
    #[serde(default)]
    pub payload: Value,
}

/// A notice and the tenant it is for, as it travels over `NOTIFY`.
#[derive(Debug, Serialize, Deserialize)]
struct Envelope {
    tenant: String,
    #[serde(flatten)]
    notice: Notice,
}

/// Publishes `payload` on `topic` to the subscribers of `tenant`, once the
/// transaction `conn` belongs to (`&mut tx`) commits; outside a
/// transaction, at once.
///
/// # Errors
///
/// When `payload` can't be serialized, or the database refuses the notice
/// (a payload over 8000 bytes).
pub async fn publish(
    conn: &mut PgConnection,
    tenant: impl Display,
    topic: &str,
    payload: impl Serialize,
) -> Result<(), AppError> {
    let envelope = Envelope {
        tenant: tenant.to_string(),
        notice: Notice {
            topic: topic.to_owned(),
            payload: serde_json::to_value(payload).map_err(|err| AppError::Internal(err.into()))?,
        },
    };
    let text = serde_json::to_string(&envelope).map_err(|err| AppError::Internal(err.into()))?;
    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(CHANNEL)
        .bind(text)
        .execute(conn)
        .await?;
    Ok(())
}

/// An instance's subscriptions: one `LISTEN` connection, fanned out to
/// every subscriber. Cheap to clone: every clone is the same hub.
#[derive(Clone)]
pub struct Topics {
    notices: broadcast::Sender<Arc<Envelope>>,
    closed: watch::Sender<bool>,
    /// Signs and checks the tickets a subscription is opened with.
    pub tickets: Arc<Tickets>,
}

impl Topics {
    /// A hub that hears nothing until [`Topics::start`], its tickets signed
    /// under a key derived from `secret` (`JWT_SECRET`).
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Topics {
            notices: broadcast::channel(BUFFER).0,
            closed: watch::channel(false).0,
            tickets: Arc::new(Tickets::from_secret(secret.as_bytes())),
        }
    }

    /// Listens on [`CHANNEL`] over a connection of its own from `pool`, and
    /// fans what it hears out in the background until [`Topics::close`]. A
    /// lost connection is reconnected; what was published meanwhile is
    /// missed.
    ///
    /// # Errors
    ///
    /// When no connection is had or `LISTEN` fails.
    pub async fn start(&self, pool: &PgPool) -> Result<(), AppError> {
        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen(CHANNEL).await?;
        let (notices, mut closed) = (self.notices.clone(), self.closed.subscribe());
        tokio::spawn(async move {
            loop {
                let received = tokio::select! {
                    received = listener.recv() => received,
                    // A hub dropped without closing keeps listening.
                    Ok(_) = closed.wait_for(|closed| *closed) => return,
                };
                match received {
                    Ok(notification) => {
                        match serde_json::from_str::<Envelope>(notification.payload()) {
                            // No subscriber is no error.
                            Ok(envelope) => drop(notices.send(Arc::new(envelope))),
                            Err(err) => tracing::warn!(error = %err, "live: an unreadable notice"),
                        }
                    }
                    Err(err) => {
                        tracing::warn!(error = %err, "live: the listener failed; reconnecting");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });
        Ok(())
    }

    /// The notices on `topics` for `tenant`, as they come. The stream ends
    /// when the subscriber falls more than [`BUFFER`] notices behind, or
    /// when the hub is closed.
    pub fn subscribe(&self, tenant: &str, topics: impl IntoIterator<Item = String>) -> Notices {
        let state = Subscription {
            tenant: tenant.to_owned(),
            topics: topics.into_iter().collect(),
            notices: self.notices.subscribe(),
            closed: self.closed.subscribe(),
        };
        Notices(Box::pin(futures_util::stream::unfold(
            state,
            |mut state| async move {
                let notice = state.next().await?;
                Some((notice, state))
            },
        )))
    }

    /// Stops listening and ends every subscription: the drain on
    /// `SIGTERM`, so open streams don't hold the shutdown up.
    pub fn close(&self) {
        self.closed.send_replace(true);
    }
}

/// One subscriber's side of the hub.
struct Subscription {
    tenant: String,
    topics: HashSet<String>,
    notices: broadcast::Receiver<Arc<Envelope>>,
    closed: watch::Receiver<bool>,
}

impl Subscription {
    /// The next notice for this subscriber, or `None` once it fell behind
    /// or the hub closed.
    async fn next(&mut self) -> Option<Notice> {
        loop {
            let envelope = tokio::select! {
                received = self.notices.recv() => received.ok()?,
                Ok(_) = self.closed.wait_for(|closed| *closed) => return None,
            };
            if envelope.tenant == self.tenant && self.topics.contains(&envelope.notice.topic) {
                return Some(envelope.notice.clone());
            }
        }
    }
}

/// The notices of one subscription ([`Topics::subscribe`]), as a stream.
pub struct Notices(Pin<Box<dyn Stream<Item = Notice> + Send>>);

impl Stream for Notices {
    type Item = Notice;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Notice>> {
        self.0.as_mut().poll_next(cx)
    }
}
