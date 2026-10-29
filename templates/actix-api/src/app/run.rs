// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`]: the server and, beside it, the job [`worker`] and the
//! [`schedule`], as `APP_ROLE` says, and the shutdown that drains them.

use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};

use super::middleware::unready_while_draining;
use super::{AppState, app};
use crate::config::Config;
use crate::db::pool::init_pool;
use crate::features::media::SweepUploads;
use crate::shared::health;
use crate::shared::jobs::{Background, Schedule, ScheduleError, Worker};
use crate::shared::mail::{Mailer, SendMail, transport};
use crate::shared::metrics;
use crate::shared::shutdown;

/// Logs the settings (secrets left out), connects the pool, then serves
/// `APP_ROLE`'s part on `config.bind_addr` until Ctrl+C or `SIGTERM`: the
/// API (or, for a `worker`, its `/health` only), and, unless it is `web`,
/// the job [`worker`] and the [`schedule`] beside it. It then drains (see
/// `shared/shutdown.rs`): not ready but serving for `SHUTDOWN_DELAY_SECS`,
/// then the requests in flight and the running jobs finish within
/// `SHUTDOWN_GRACE_SECS`.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config).map_err(std::io::Error::other)?;
    let (role, settings, draining) = (config.app_role, state.shutdown, state.draining.clone());
    let background = if role.runs_jobs() {
        let schedule = schedule().map_err(std::io::Error::other)?;
        Some(Background::start(
            worker(&state, mailer),
            schedule,
            state.pool.clone(),
            state.queue.clone(),
        ))
    } else {
        None
    };
    if let Some(addr) = config.metrics_addr {
        metrics::install(addr).map_err(std::io::Error::other)?;
        actix_web::rt::spawn(metrics::sample_pool(
            state.pool.clone(),
            metrics::POOL_SAMPLE_EVERY,
        ));
        tracing::info!(%addr, "serving /metrics");
    }
    tracing::info!(addr = %config.bind_addr, role = role.name(), "listening");
    // actix-web's own signal handling would stop at once: the drain below
    // handles the signal, and `shutdown_timeout` is the grace.
    let grace = settings.grace.as_secs();
    let server: Server = if role.serves_api() {
        HttpServer::new(move || app(state.clone()))
            .bind(&config.bind_addr)?
            .disable_signals()
            .shutdown_timeout(grace)
            .run()
    } else {
        let pool = state.pool.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(state.draining.clone()))
                .configure(health::routes)
                .wrap(from_fn(unready_while_draining))
        })
        .bind(&config.bind_addr)?
        .disable_signals()
        .shutdown_timeout(grace)
        .run()
    };
    let handle = server.handle();
    let draining_task = actix_web::rt::spawn(async move {
        shutdown::signal().await;
        draining.start();
        tracing::info!(delay = ?settings.delay, "draining: /health/ready answers 503");
        tokio::time::sleep(settings.delay).await;
        // Stops accepting, and gives the requests in flight the grace.
        handle.stop(true).await;
    });
    let served = server.await;
    draining_task.abort();
    if let Some(background) = background {
        background.stop_within(settings.grace).await;
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
        .register::<SweepUploads>(state.media_service())
}

/// The jobs enqueued on a clock (UTC cron patterns), once per tick across
/// every process. Add one: `.cron::<NightlyReport>("0 3 * * *")?`; its kind
/// must be registered in [`worker`] too.
pub fn schedule() -> Result<Schedule, ScheduleError> {
    Schedule::new().cron::<SweepUploads>("*/5 * * * *")
}
