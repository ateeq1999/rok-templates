// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Metrics, in Prometheus's text format. When `METRICS_ADDR` is set,
//! [`install`] serves them at `http://<METRICS_ADDR>/metrics`, a listener
//! of its own apart from the API's port, so they are never public; unset,
//! nothing is recorded and nothing listens.
//!
//! What is measured, with its labels:
//!
//! | Metric | Kind | Labels |
//! |---|---|---|
//! | `http_requests_total` | counter | `method`, `route` (the matched pattern, or `unmatched`), `status` |
//! | `http_request_duration_seconds` | histogram | `method`, `route` |
//! | `db_pool_connections` | gauge | `state`: `idle` or `in_use` |
//! | `db_pool_max_connections` | gauge | — |
//! | `auth_logins_total` | counter | `outcome`: `success`, `two_factor`, `bad_credentials`, `locked`, `forbidden` |
//! | `auth_lockouts_total` | counter | — |
//! | `jobs_total` | counter | `kind`, `outcome`: `done` or `failed` (an attempt) |
//! | `job_duration_seconds` | histogram | `kind` |
//! | `jobs_queued` | gauge | `state`: `due`, `scheduled` (not due yet) or `dead` |
//!
//! Add your own with the `metrics` macros anywhere
//! (`metrics::counter!("orders_placed_total").increment(1)`); they are
//! exported with these. Keep labels to a few known values: a label per
//! user or per id makes a series per user. Framework-free, so both
//! templates share it; each template's `app/middleware.rs` times the
//! requests.

use std::net::SocketAddr;
use std::time::Duration;

use metrics_exporter_prometheus::{BuildError, Matcher, PrometheusBuilder};
use sqlx::PgPool;

use crate::db::queries::queue;

/// Requests answered.
pub const HTTP_REQUESTS: &str = "http_requests_total";
/// How long requests took.
pub const HTTP_REQUEST_DURATION: &str = "http_request_duration_seconds";
/// The pool's open connections, by state.
pub const DB_POOL_CONNECTIONS: &str = "db_pool_connections";
/// The most connections the pool opens.
pub const DB_POOL_MAX_CONNECTIONS: &str = "db_pool_max_connections";
/// Sign-in attempts, by outcome.
pub const AUTH_LOGINS: &str = "auth_logins_total";
/// Accounts locked for too many wrong passwords.
pub const AUTH_LOCKOUTS: &str = "auth_lockouts_total";
/// Job attempts, by kind and outcome.
pub const JOBS: &str = "jobs_total";
/// How long job attempts took.
pub const JOB_DURATION: &str = "job_duration_seconds";
/// The jobs in the queue, by state.
pub const JOBS_QUEUED: &str = "jobs_queued";

/// The route label of a request no route matched: one series for them
/// all, whatever paths are tried.
pub const UNMATCHED_ROUTE: &str = "unmatched";

/// The request duration's histogram buckets, in seconds.
const DURATION_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0,
];

/// How often [`sample_pool`] reads the pool.
pub const POOL_SAMPLE_EVERY: Duration = Duration::from_secs(15);

/// Records the metrics from now on and serves them at
/// `http://{addr}/metrics`. Call it once, from inside the runtime.
pub fn install(addr: SocketAddr) -> Result<(), BuildError> {
    PrometheusBuilder::new()
        .with_http_listener(addr)
        .set_buckets_for_metric(
            Matcher::Full(HTTP_REQUEST_DURATION.to_owned()),
            DURATION_BUCKETS,
        )?
        .install()
}

/// Counts and times one answered request: its `method`, its `route` (the
/// matched pattern, or [`UNMATCHED_ROUTE`]) and its `status`.
pub fn record_request(method: &str, route: &str, status: u16, elapsed: Duration) {
    let method = method.to_owned();
    let route = route.to_owned();
    metrics::counter!(
        HTTP_REQUESTS,
        "method" => method.clone(),
        "route" => route.clone(),
        "status" => status.to_string(),
    )
    .increment(1);
    metrics::histogram!(HTTP_REQUEST_DURATION, "method" => method, "route" => route)
        .record(elapsed.as_secs_f64());
}

/// How a sign-in attempt ended, for [`record_login`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginOutcome {
    /// Signed in.
    Success,
    /// The password was right; the second factor is asked for next.
    TwoFactor,
    /// A wrong password or an unknown email.
    BadCredentials,
    /// The account is locked for too many wrong passwords.
    Locked,
    /// The password was right, but the account is disabled or unverified.
    Forbidden,
}

impl LoginOutcome {
    /// The `outcome` label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            LoginOutcome::Success => "success",
            LoginOutcome::TwoFactor => "two_factor",
            LoginOutcome::BadCredentials => "bad_credentials",
            LoginOutcome::Locked => "locked",
            LoginOutcome::Forbidden => "forbidden",
        }
    }
}

/// Counts a sign-in attempt that ended in `outcome`.
pub fn record_login(outcome: LoginOutcome) {
    metrics::counter!(AUTH_LOGINS, "outcome" => outcome.label()).increment(1);
}

/// Counts an account locked for too many wrong passwords.
pub fn record_lockout() {
    metrics::counter!(AUTH_LOCKOUTS).increment(1);
}

/// Counts and times one attempt at a job of `kind`.
pub fn record_job(kind: &str, succeeded: bool, elapsed: Duration) {
    let outcome = if succeeded { "done" } else { "failed" };
    metrics::counter!(JOBS, "kind" => kind.to_owned(), "outcome" => outcome).increment(1);
    metrics::histogram!(JOB_DURATION, "kind" => kind.to_owned()).record(elapsed.as_secs_f64());
}

/// Reads `pool`'s connections and the job queue's depth into their gauges
/// every `every`, forever: spawn it beside the server when metrics are on.
pub async fn sample_pool(pool: PgPool, every: Duration) {
    let mut ticks = tokio::time::interval(every);
    loop {
        ticks.tick().await;
        let size = pool.size();
        let idle = u32::try_from(pool.num_idle()).unwrap_or(u32::MAX);
        metrics::gauge!(DB_POOL_CONNECTIONS, "state" => "idle").set(f64::from(idle));
        metrics::gauge!(DB_POOL_CONNECTIONS, "state" => "in_use")
            .set(f64::from(size.saturating_sub(idle)));
        metrics::gauge!(DB_POOL_MAX_CONNECTIONS)
            .set(f64::from(pool.options().get_max_connections()));
        match queue::count_jobs(&pool).await {
            Ok(counts) => {
                // A state with no job left is no row: zero it.
                for state in ["due", "scheduled", "dead"] {
                    let jobs = counts
                        .iter()
                        .find(|row| row.state == state)
                        .map_or(0, |row| row.jobs);
                    #[allow(clippy::cast_precision_loss)]
                    metrics::gauge!(JOBS_QUEUED, "state" => state).set(jobs as f64);
                }
            }
            Err(err) => tracing::warn!(error = %err, "could not count the queued jobs"),
        }
    }
}
