// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Stopping without dropping work. On Ctrl+C or `SIGTERM` ([`signal`]),
//! `app/run.rs`:
//!
//! 1. marks the process [`Draining`]: `/health/ready` answers `503` from now
//!    on, so the load balancer takes it out of rotation, while it still
//!    serves every request it gets, for `SHUTDOWN_DELAY_SECS` (5);
//! 2. stops accepting connections and lets the requests in flight finish,
//!    until `SHUTDOWN_GRACE_SECS` (25) after the delay;
//! 3. stops the job worker and the schedule: a running job finishes within
//!    what is left of the grace, or is abandoned, its transaction rolled
//!    back, so its claim is released, no attempt counted, and it runs again.
//!
//! Keep the platform's own stop timeout (Kubernetes'
//! `terminationGracePeriodSeconds`, ECS's `stopTimeout`) above the delay
//! plus the grace. Framework-free, so both templates share it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The answer `/health/ready` gives while draining.
pub const DRAINING_MESSAGE: &str = "the server is shutting down";

/// Whether the process is shutting down. Cheap to clone: every clone sees
/// the same flag.
#[derive(Debug, Clone, Default)]
pub struct Draining(Arc<AtomicBool>);

impl Draining {
    /// Marks the process as shutting down, for good.
    pub fn start(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Whether [`start`](Self::start) was called.
    #[must_use]
    pub fn is_draining(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// How the process stops (`SHUTDOWN_DELAY_SECS`, `SHUTDOWN_GRACE_SECS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownSettings {
    /// How long it keeps serving, not ready, before it stops accepting.
    pub delay: Duration,
    /// How long, after the delay, requests in flight and running jobs have
    /// to finish.
    pub grace: Duration,
}

/// Resolves on Ctrl+C or, on Unix, `SIGTERM`.
pub async fn signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "could not listen for Ctrl+C");
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut terminate) => {
                terminate.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "could not listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
    tracing::info!("shutting down");
}
