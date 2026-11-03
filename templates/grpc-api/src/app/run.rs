// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`run`] and [`serve`]: the gRPC server and, beside it, the job
//! [`worker`] and the [`schedule`], as `APP_ROLE` says, and the shutdown
//! that stops them.

use std::future::Future;
use std::sync::Arc;

use tokio::net::TcpListener;
use tonic::service::Routes;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tonic_health::ServingStatus;
use tonic_health::server::HealthReporter;

use super::{AppState, routes, services};
use crate::config::{AppRole, Config};
use crate::db::pool::init_pool;
use crate::shared::jobs::{Background, Schedule, ScheduleError, Worker};
use crate::shared::mail::{Mailer, SendMail, transport};
use crate::shared::metrics;
use crate::shared::shutdown::{self, Draining, ShutdownSettings};

/// The service name the health service reports the whole server under, as
/// the gRPC health protocol names it.
const SERVER: &str = "";

/// Logs the settings (secrets left out), connects the pool, then serves
/// what `APP_ROLE` says on `config.bind_addr` (see [`serve_as`]) until
/// Ctrl+C or `SIGTERM`.
pub async fn run(config: Config) -> std::io::Result<()> {
    tracing::info!("settings:\n{}", config.summary());
    let pool = init_pool(&config.database_url, &config.pool)
        .await
        .map_err(std::io::Error::other)?;
    let mailer = transport::from_config(&config).map_err(std::io::Error::other)?;
    let state = AppState::new(pool, &config);
    if let Some(addr) = config.metrics_addr {
        metrics::install(addr).map_err(std::io::Error::other)?;
        tokio::spawn(metrics::sample_pool(
            state.pool.clone(),
            metrics::POOL_SAMPLE_EVERY,
        ));
        tracing::info!(%addr, "serving /metrics");
    }
    let listener = TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, role = config.app_role.name(), "listening");
    serve_as(config.app_role, listener, state, mailer, shutdown::signal()).await
}

/// [`serve_as`] for `APP_ROLE=all`: the API, the job worker and the
/// schedule in one process.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    mailer: Arc<dyn Mailer>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    serve_as(AppRole::All, listener, state, mailer, shutdown).await
}

/// Serves `role`'s part on `listener` until `shutdown` completes: the
/// [`routes`] over `state` (or, for a `worker`, the health and reflection
/// services only), and, unless it is `web`, the job [`worker`] and the
/// [`schedule`] beside it, its mail sent through `mailer`. On shutdown the
/// calls in flight finish, the server returns, and the worker stops once
/// its running jobs settle.
pub async fn serve_as(
    role: AppRole,
    listener: TcpListener,
    state: AppState,
    mailer: Arc<dyn Mailer>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let (draining, settings) = (state.draining.clone(), state.shutdown);
    let background = if role.runs_jobs() {
        let schedule = schedule().map_err(std::io::Error::other)?;
        let worker = worker(&state, mailer);
        Some(Background::start(
            worker,
            schedule,
            state.pool.clone(),
            state.queue.clone(),
        ))
    } else {
        None
    };
    let health = HealthReporter::new();
    let served = if role.serves_api() {
        routes(&state, health.clone())
    } else {
        services(Routes::default(), health.clone())
    }
    .map_err(std::io::Error::other)?;
    let (stop_accepting, accepting_stopped) = tokio::sync::oneshot::channel::<()>();
    let mut server = tokio::spawn(
        Server::builder()
            .add_routes(served)
            .serve_with_incoming_shutdown(TcpIncoming::from(listener), async {
                let _ = accepting_stopped.await;
            }),
    );
    let served = tokio::select! {
        // The server failed on its own: nothing to drain.
        served = &mut server => Some(
            served
                .map_err(std::io::Error::other)
                .and_then(|served| served.map_err(std::io::Error::other)),
        ),
        () = shutdown => None,
    };
    let served = match served {
        Some(served) => served,
        None => drain(&draining, &health, settings, stop_accepting, server).await,
    };
    if let Some(background) = background {
        background.stop_within(settings.grace).await;
    }
    served
}

/// Drains the `server` (see `shared/shutdown.rs`): `NOT_SERVING` but
/// serving for `settings.delay`, then accepting no more and finishing the
/// calls in flight within `settings.grace`; the ones still running then
/// are dropped.
async fn drain(
    draining: &Draining,
    health: &HealthReporter,
    settings: ShutdownSettings,
    stop_accepting: tokio::sync::oneshot::Sender<()>,
    mut server: tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
) -> std::io::Result<()> {
    draining.start();
    health
        .set_service_status(SERVER, ServingStatus::NotServing)
        .await;
    tracing::info!(delay = ?settings.delay, "draining: the health service answers NOT_SERVING");
    tokio::time::sleep(settings.delay).await;
    let _ = stop_accepting.send(());
    if let Ok(served) = tokio::time::timeout(settings.grace, &mut server).await {
        served
            .map_err(std::io::Error::other)?
            .map_err(std::io::Error::other)
    } else {
        tracing::warn!(grace = ?settings.grace, "calls still running after the grace: dropped");
        server.abort();
        Ok(())
    }
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

/// The jobs enqueued on a clock (UTC cron patterns), once per tick across
/// every process. Add one: `.cron::<NightlyReport>("0 3 * * *")?`; its kind
/// must be registered in [`worker`] too.
///
/// # Errors
///
/// A pattern that doesn't parse.
pub fn schedule() -> Result<Schedule, ScheduleError> {
    Ok(Schedule::new())
}
