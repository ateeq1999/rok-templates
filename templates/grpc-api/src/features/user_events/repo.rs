// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `user_events` channel: a [`Listener`] on it, and [`publish`], which
//! notifies it. A notification's payload is the id of the user that
//! changed, as text.

use sqlx::PgPool;
use sqlx::postgres::PgListener;

use super::model::UserId;
use crate::error::AppError;

/// The PostgreSQL channel the events travel on: `NOTIFY user_events,
/// '<user id>'` (or `SELECT pg_notify('user_events', '<user id>')`).
pub const CHANNEL: &str = "user_events";

/// A connection listening on [`CHANNEL`]. It holds one of the pool's
/// connections until it is dropped, which stops listening.
pub struct Listener(PgListener);

/// A connection from `pool`, listening on [`CHANNEL`] once this returns:
/// a notification sent after it is received.
///
/// # Errors
///
/// When no connection is had or `LISTEN` fails.
pub async fn listen(pool: &PgPool) -> Result<Listener, AppError> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(CHANNEL).await?;
    Ok(Listener(listener))
}

impl Listener {
    /// The next notification's payload, or `None` when the connection was
    /// lost (and made again): the notifications sent meanwhile are gone.
    ///
    /// # Errors
    ///
    /// When the connection can't be made again, or the pool is closed.
    pub async fn next(&mut self) -> Result<Option<String>, AppError> {
        let notification = self.0.try_recv().await?;
        Ok(notification.map(|notification| notification.payload().to_owned()))
    }
}

/// Notifies [`CHANNEL`] that the user `id` changed. Inside a transaction,
/// the notification is sent on commit, and not at all on rollback.
///
/// # Errors
///
/// When the database can't be reached.
pub async fn publish<'e, E>(executor: E, id: UserId) -> Result<(), AppError>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query("SELECT pg_notify($1, $2)")
        .bind(CHANNEL)
        .bind(id.0.to_string())
        .execute(executor)
        .await?;
    Ok(())
}
