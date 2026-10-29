// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`]: the server and, beside it, the job [`worker`] and the
//! [`schedule`], as `APP_ROLE` says.

use std::sync::Arc;

use actix_web::{App, HttpServer, web};

use super::{AppState, app};
use crate::config::Config;
use crate::db::pool::init_pool;
use crate::features::media::SweepUploads;
use crate::shared::health;
use crate::shared::jobs::{Background, Schedule, ScheduleError, Worker};
use crate::shared::mail::{Mailer, SendMail, transport};
use crate::shared::metrics;

/// Logs the settings (secrets left out), connects the pool, then serves
/// `APP_ROLE`'s part on `config.bind_addr` until the server stops: the API
/// (or, for a `worker`, its `/health` only), and, unless it is `web`, the
/// job [`worker`] and the [`schedule`] beside it, stopped once their running
/// jobs settle.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config).map_err(std::io::Error::other)?;
    let role = config.app_role;
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
    let served = if role.serves_api() {
        HttpServer::new(move || app(state.clone()))
            .bind(&config.bind_addr)?
            .run()
            .await
    } else {
        let pool = state.pool.clone();
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(health::routes)
        })
        .bind(&config.bind_addr)?
        .run()
        .await
    };
    if let Some(background) = background {
        background.stop().await;
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
