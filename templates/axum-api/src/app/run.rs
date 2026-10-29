// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`] and [`serve`]: the server, the job worker and the upload sweep
//! beside it, and the shutdown that stops them.

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;

use super::{AppState, router};
use crate::config::Config;
use crate::db::pool::init_pool;
use crate::features::media::MediaService;
use crate::shared::jobs::Worker;
use crate::shared::mail::{Mailer, SendMail, transport};
use crate::shared::metrics;

/// Logs the settings (secrets left out), connects the pool, then serves the
/// application on `config.bind_addr` (see [`serve`]) until Ctrl+C or
/// `SIGTERM`.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config).map_err(std::io::Error::other)?;
    if let Some(addr) = config.metrics_addr {
        metrics::install(addr).map_err(std::io::Error::other)?;
        tokio::spawn(metrics::sample_pool(
            state.pool.clone(),
            metrics::POOL_SAMPLE_EVERY,
        ));
        tracing::info!(%addr, "serving /metrics");
    }
    if let Some(every) = config.media_upload_sweep {
        tokio::spawn(MediaService::clone(&state.media).sweep_every(every));
    }
    let listener = TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");
    serve(listener, state, mailer, shutdown_signal()).await
}

/// Serves [`router`] over `state` on `listener`, with the job [`worker`]
/// beside it (`tokio::spawn`), its mail sent through `mailer`, until
/// `shutdown` completes: then requests in flight finish, the server
/// returns, and the worker stops once its running jobs settle.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    mailer: Arc<dyn Mailer>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let worker = worker(&state, mailer);
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let working = tokio::spawn(worker.run(async {
        // A dropped `stop` stops the worker too.
        let _ = stopped.await;
    }));
    let app = router(state).into_make_service_with_connect_info::<SocketAddr>();
    let served = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await;
    let _ = stop.send(());
    if let Err(err) = working.await {
        tracing::error!(error = %err, "the job worker failed");
    }
    served
}

/// The job worker over `state`'s queue, `JOBS_CONCURRENCY` jobs at once,
/// every job kind registered with what it runs with. Register a kind of
/// your own here: `.register::<SendInvoice>(state.billing.clone())`.
#[must_use]
pub fn worker(state: &AppState, mailer: Arc<dyn Mailer>) -> Worker {
    Worker::new(state.pool.clone(), &state.queue)
        .with_concurrency(state.jobs_concurrency)
        .register::<SendMail>(mailer)
}

/// Resolves on Ctrl+C or, on Unix, `SIGTERM`.
async fn shutdown_signal() {
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
