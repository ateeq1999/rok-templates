// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`]: the server, and the job worker and the upload sweep beside it.

use std::sync::Arc;

use actix_web::HttpServer;

use super::{AppState, app};
use crate::config::Config;
use crate::db::pool::init_pool;
use crate::shared::jobs::Worker;
use crate::shared::mail::{Mailer, SendMail, transport};
use crate::shared::metrics;

/// Logs the settings (secrets left out), connects the pool, starts the job
/// [`worker`], then serves the application on `config.bind_addr` until the
/// server stops, and stops the worker once its running jobs settle.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config).map_err(std::io::Error::other)?;
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let working = actix_web::rt::spawn(worker(&state, mailer).run(async {
        // A dropped `stop` stops the worker too.
        let _ = stopped.await;
    }));
    if let Some(addr) = config.metrics_addr {
        metrics::install(addr).map_err(std::io::Error::other)?;
        actix_web::rt::spawn(metrics::sample_pool(
            state.pool.clone(),
            metrics::POOL_SAMPLE_EVERY,
        ));
        tracing::info!(%addr, "serving /metrics");
    }
    if let Some(every) = state.media_upload_sweep {
        actix_web::rt::spawn(state.media_service().sweep_every(every));
    }
    tracing::info!(addr = %config.bind_addr, "listening");
    let server_state = state.clone();
    let served = HttpServer::new(move || app(server_state.clone()))
        .bind(&config.bind_addr)?
        .run()
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
