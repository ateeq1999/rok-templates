// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! `UserEventsService`: a subscription to the `user_events` channel. Each
//! subscriber has a connection of its own listening on it, and a task that
//! reads each notification's user and hands it over through a bounded
//! queue: [`listen`](UserEventsService::listen). The task stops, and the
//! connection goes back to the pool, once the subscriber hangs up, falls
//! [`BUFFER`] deliveries behind, or loses the connection.

use sqlx::PgPool;
use tokio::sync::mpsc;

use super::model::{User, UserId};
use super::repo::{self, Listener};
use crate::error::AppError;

/// How many deliveries wait for a subscriber that reads slowly. One that
/// falls this far behind is ended with [`Delivery::FellBehind`]: waiting for
/// it would leave the notifications to pile up in PostgreSQL's queue.
pub const BUFFER: usize = 16;

/// What a subscriber receives, in order. Only [`Changed`](Self::Changed)
/// is followed by more: the others are the last.
#[derive(Debug)]
pub enum Delivery {
    /// The user `id` changed: `user` is the row as it is now, or `None`
    /// once it is gone (deleted).
    Changed {
        /// The id the notification named.
        id: UserId,
        /// The row, or `None` once it is gone.
        user: Option<Box<User>>,
    },
    /// The subscriber fell [`BUFFER`] deliveries behind: the ones after
    /// were dropped.
    FellBehind,
    /// The connection was lost: the notifications sent until it was made
    /// again are gone.
    Interrupted,
}

/// The `user_events` service, added in `app/routes.rs`.
pub struct UserEventsService {
    /// The pool the listeners' connections and the reads come from.
    pub pool: PgPool,
}

impl UserEventsService {
    /// The service over `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        UserEventsService { pool }
    }

    /// A subscription: listening once this returns, then a [`Delivery`]
    /// per notification until the receiver is dropped or a last one is
    /// sent. A failure while delivering is the last item.
    ///
    /// # Errors
    ///
    /// When no connection is had or `LISTEN` fails.
    pub async fn listen(&self) -> Result<mpsc::Receiver<Result<Delivery, AppError>>, AppError> {
        let listener = repo::listen(&self.pool).await?;
        let (sender, receiver) = mpsc::channel(BUFFER);
        tokio::spawn(deliver(self.pool.clone(), listener, sender));
        Ok(receiver)
    }

    /// Tells every subscriber that the user `id` changed. To send it with a
    /// transaction's commit, call `repo::publish` on the transaction.
    ///
    /// # Errors
    ///
    /// When the database can't be reached.
    pub async fn publish(&self, id: UserId) -> Result<(), AppError> {
        repo::publish(&self.pool, id).await
    }
}

/// Reads `listener`'s notifications into `sender` until the subscriber
/// hangs up (the receiver is dropped) or a last delivery is sent; then the
/// listener is dropped, which stops listening.
async fn deliver(
    pool: PgPool,
    mut listener: Listener,
    sender: mpsc::Sender<Result<Delivery, AppError>>,
) {
    loop {
        let next = tokio::select! {
            () = sender.closed() => return,
            next = listener.next() => next,
        };
        let delivery = match next {
            Ok(Some(payload)) => match changed(&pool, &payload).await.transpose() {
                Some(delivery) => delivery,
                None => continue,
            },
            Ok(None) => Ok(Delivery::Interrupted),
            Err(error) => Err(error),
        };
        let more = matches!(delivery, Ok(Delivery::Changed { .. }));
        // The last free slot is kept for a last delivery: a change takes
        // it only when one more is free, so a last one always fits.
        let (delivery, more) = if more && sender.capacity() <= 1 {
            (Ok(Delivery::FellBehind), false)
        } else {
            (delivery, more)
        };
        if sender.try_send(delivery).is_err() || !more {
            return;
        }
    }
}

/// The delivery of a notification whose payload is `payload`, or `None`
/// (logged) when it names no user id: another publisher's mistake, which
/// the subscribers don't hear of.
async fn changed(pool: &PgPool, payload: &str) -> Result<Option<Delivery>, AppError> {
    let Ok(id) = payload.parse::<uuid::Uuid>() else {
        tracing::warn!(
            payload,
            "a `user_events` notification names no user id: skipped"
        );
        return Ok(None);
    };
    let id = UserId(id);
    let user = User::by_primary_key(id).first(pool).await?;
    Ok(Some(Delivery::Changed {
        id,
        user: user.map(Box::new),
    }))
}
