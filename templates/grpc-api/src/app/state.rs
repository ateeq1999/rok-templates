// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`AppState`]: what the server and the services are built from, once,
//! from the [`Config`].

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;
use crate::shared::jobs::Queue;
use crate::shared::security::jwt::JwtKeys;
use crate::shared::shutdown::{Draining, ShutdownSettings};

/// What the server and the services are built from, once: by
/// [`run`](super::run), or by a test. The generated features build their
/// services over `pool` in `features::routes` (`features/generated.rs`).
#[derive(Clone)]
pub struct AppState {
    /// The database the services read and write.
    pub pool: PgPool,
    /// Verifies the bearer tokens the axum template signs: the same
    /// `JWT_SECRET` (and `JWT_SECRET_PREVIOUS`), the same claims.
    pub keys: Arc<JwtKeys>,
    /// Where jobs are enqueued; it wakes the job worker.
    pub queue: Queue,
    /// How many jobs the worker runs at once (`JOBS_CONCURRENCY`).
    pub jobs_concurrency: usize,
    /// Set on `SIGTERM`: the health service answers `NOT_SERVING` from then
    /// on.
    pub draining: Draining,
    /// How the process stops (`SHUTDOWN_DELAY_SECS`, `SHUTDOWN_GRACE_SECS`).
    pub shutdown: ShutdownSettings,
}

impl AppState {
    /// The state `config` describes, over `pool`.
    #[must_use]
    pub fn new(pool: PgPool, config: &Config) -> Self {
        let keys = Arc::new(
            JwtKeys::new(&config.jwt_secret, config.jwt_ttl)
                .with_previous(config.jwt_secret_previous.as_deref()),
        );
        AppState {
            pool,
            keys,
            queue: Queue::new(),
            jobs_concurrency: config.jobs_concurrency,
            draining: Draining::default(),
            shutdown: config.shutdown,
        }
    }
}
