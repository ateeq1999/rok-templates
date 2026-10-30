// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The application's settings, read from the environment into one
//! [`Config`]. This is the only file that reads environment variables.
//! `.env.example` lists them all.
//!
//! | Variable | Default | Setting |
//! |---|---|---|
//! | `DATABASE_URL` | required | the database URL |
//! | `APP_ROLE` | `all` | what this process runs: `all` (the API and the job worker), `web` (the API) or `worker` (the job worker and the schedule; `/health` only on `BIND_ADDR`) |
//! | `BIND_ADDR` | `0.0.0.0:8080` | where the server listens |
//! | `METRICS_ADDR` | unset | where `/metrics` (Prometheus) is served, apart from the API: `0.0.0.0:9090` in a container; unset, no metrics |
//! | `JWT_SECRET` | required | the HS256 key tokens are signed with, at least 32 bytes |
//! | `JWT_TTL` | 3600 | seconds an issued token lasts |
//! | `JWT_SECRET_PREVIOUS` | unset | the secret being rotated out: tokens it signed still pass (at least 32 bytes) |
//! | `TOTP_ENCRYPTION_KEY` | required | the AES-256 key two-factor secrets are stored under: 32 bytes, base64 |
//! | `REQUIRE_VERIFIED_EMAIL` | `true` | sign-in needs a verified email |
//! | `APP_URL` | `http://localhost:8080` | the base of the links mails carry |
//! | `MAIL_TRANSPORT` | `log` | `log` (mails are logged) or `smtp` |
//! | `SMTP_URL` | required for `smtp` | such as `smtps://user:pass@host:465` |
//! | `MAIL_FROM` | `Actix API <no-reply@localhost>` | the sender of every mail |
//! | `BREACH_CHECK` | `false` | reject passwords Have I Been Pwned knows |
//! | `TRUST_PROXY` | `false` | take the client IP from `X-Forwarded-For` |
//! | `AUTH_RATE_LIMIT` | 20 | requests per minute per IP on `/auth` (`0`: off) |
//! | `GLOBAL_RATE_LIMIT` | 0 | requests per minute per IP on every route but health (`0`: off) |
//! | `JOBS_CONCURRENCY` | 4 | background jobs run at once; each holds a pool connection while it runs |
//! | `SHUTDOWN_DELAY_SECS` | 5 | on `SIGTERM`, seconds it keeps serving while `/health/ready` answers `503`, so the load balancer stops sending it requests |
//! | `SHUTDOWN_GRACE_SECS` | 25 | then, seconds requests in flight and running jobs have to finish |
//! | `CORS_ALLOWED_ORIGINS` | unset | the other origins a browser may call the API from: `*`, or a list such as `https://app.example.com,http://localhost:5173`; unset, none |
//! | `STORAGE_URL` | `file://./storage` | where uploads are stored: `file://<dir>`, `s3://<bucket>` (`AWS_*` settings) or `memory://` |
//! | `MEDIA_MAX_UPLOAD_BYTES` | 10485760 (10 MiB) | the largest `POST /media` body |
//! | `MEDIA_ALLOWED_TYPES` | `image/jpeg,image/png,image/webp,image/gif,application/pdf` | the detected types an upload may have |
//! | `MEDIA_UPLOAD_URL_TTL_SECS` | 900 | seconds a direct-upload URL (`POST /media/uploads`) works |
//! | `MEDIA_DOWNLOAD_URL_TTL_SECS` | 300 | seconds a download URL (`GET /media/{id}/url`) works |
//! | `STORAGE_PUBLIC_ENDPOINT` | unset | the endpoint S3 URLs are signed for, when clients reach the store elsewhere than the API (`http://localhost:9000` for `MinIO` in Docker) |
//! | `DB_MAX_CONNECTIONS` | 20 | pool size |
//! | `DB_MIN_CONNECTIONS` | 5 | idle connections kept open |
//! | `DB_ACQUIRE_TIMEOUT_SECS` | 3 | wait for a connection |
//! | `DB_IDLE_TIMEOUT_SECS` | 600 | close an idle connection after |
//! | `DB_MAX_LIFETIME_SECS` | 1800 | replace a connection after |
//! | `RUST_LOG` | `info` | the log filter ([`log_settings`]) |
//! | `LOG_FORMAT` | `pretty` | `pretty` (readable lines) or `json` (one object per line, for a log collector) |
//! | `OTEL_EXPORTER_OTLP_ENDPOINT` | unset | export every span over OTLP/HTTP to this collector, such as `http://localhost:4318`; unset, nothing is exported. The exporter reads the other standard `OTEL_*` variables itself |
//! | `OTEL_SERVICE_NAME` | the crate's name | the service the spans belong to |
//!
//! A boolean is `true` or `false` (also `1` or `0`). Every setting is
//! checked before the server starts, and every one that is wrong is
//! reported at once ([`ConfigErrors`]). [`Config::summary`] is what the
//! server logs it started with, secrets left out.

use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use crate::db::pool::PoolSettings;
use crate::shared::security::headers::CorsOrigins;
use crate::shared::security::totp::KEY_BYTES as TOTP_KEY_BYTES;
use crate::shared::settings::Vars;
pub use crate::shared::settings::{ConfigError, ConfigErrors};
use crate::shared::shutdown::ShutdownSettings;
use crate::shared::storage::StorageUrl;
use crate::shared::telemetry::{LogFormat, LogSettings};

/// Where the server listens when `BIND_ADDR` is not set.
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// The log filter when `RUST_LOG` is not set: everything at `info` and
/// above.
const DEFAULT_LOG_FILTER: &str = "info";

/// How long a token lasts when `JWT_TTL` is not set: an hour.
const DEFAULT_JWT_TTL_SECS: u64 = 3600;

/// The shortest `JWT_SECRET` accepted, in bytes: an HS256 key shorter than
/// the hash (32 bytes) is easier to guess than the signature is to forge.
pub const MIN_JWT_SECRET_BYTES: usize = 32;

/// The link base when `APP_URL` is not set.
const DEFAULT_APP_URL: &str = "http://localhost:8080";

/// The sender when `MAIL_FROM` is not set.
const DEFAULT_MAIL_FROM: &str = "Actix API <no-reply@localhost>";

/// Requests per minute per IP on `/auth` when `AUTH_RATE_LIMIT` is not set.
const DEFAULT_AUTH_RATE_LIMIT: u32 = 20;

/// Jobs one worker runs at once when `JOBS_CONCURRENCY` is not set.
const DEFAULT_JOBS_CONCURRENCY: usize = 4;

/// Seconds of serving, not ready, after `SIGTERM` when `SHUTDOWN_DELAY_SECS`
/// is not set.
const DEFAULT_SHUTDOWN_DELAY_SECS: u64 = 5;

/// Seconds for requests and jobs to finish when `SHUTDOWN_GRACE_SECS` is not
/// set.
const DEFAULT_SHUTDOWN_GRACE_SECS: u64 = 25;

/// The largest upload when `MEDIA_MAX_UPLOAD_BYTES` is not set: 10 MiB.
const DEFAULT_MEDIA_MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

/// The detected types an upload may have when `MEDIA_ALLOWED_TYPES` is not
/// set.
const DEFAULT_MEDIA_ALLOWED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/webp",
    "image/gif",
    "application/pdf",
];

/// Seconds a direct-upload URL works when `MEDIA_UPLOAD_URL_TTL_SECS` is not
/// set: 15 minutes.
const DEFAULT_MEDIA_UPLOAD_URL_TTL_SECS: u64 = 900;

/// Seconds a download URL works when `MEDIA_DOWNLOAD_URL_TTL_SECS` is not set.
const DEFAULT_MEDIA_DOWNLOAD_URL_TTL_SECS: u64 = 300;

/// How the process logs and traces, read from the environment: `main.rs`
/// starts tracing with it before [`Config::from_env`].
pub fn log_settings() -> Result<LogSettings, ConfigErrors> {
    log_settings_from(|name| env::var(name).ok())
}

/// How the process logs and traces, read through `var`: `RUST_LOG` (such as
/// `debug` or `info,sqlx=warn`; else [`DEFAULT_LOG_FILTER`]), `LOG_FORMAT`,
/// whether `OTEL_EXPORTER_OTLP_ENDPOINT` is set, and `OTEL_SERVICE_NAME`.
pub fn log_settings_from(
    var: impl Fn(&str) -> Option<String>,
) -> Result<LogSettings, ConfigErrors> {
    let set = |name| var(name).filter(|value: &String| !value.trim().is_empty());
    let format = match set("LOG_FORMAT").as_deref() {
        None | Some("pretty") => LogFormat::Pretty,
        Some("json") => LogFormat::Json,
        Some(other) => {
            return Err(ConfigErrors(vec![ConfigError::NotOneOf {
                name: "LOG_FORMAT",
                value: other.to_owned(),
                expected: "pretty, json",
            }]));
        }
    };
    Ok(LogSettings {
        filter: set("RUST_LOG").unwrap_or_else(|| DEFAULT_LOG_FILTER.to_owned()),
        format,
        export_traces: set("OTEL_EXPORTER_OTLP_ENDPOINT").is_some(),
        service_name: set("OTEL_SERVICE_NAME"),
    })
}

/// What this process runs (`APP_ROLE`). One image runs any of them: scale
/// `web` and `worker` processes apart, or run `all` in one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRole {
    /// `all`: the API, the job worker and the scheduler. The default.
    All,
    /// `web`: the API only; jobs it enqueues run in a `worker`.
    Web,
    /// `worker`: the job worker and the scheduler, serving only `/health`
    /// on `BIND_ADDR` (for the platform's health check).
    Worker,
}

impl AppRole {
    /// Whether this process serves the API.
    #[must_use]
    pub fn serves_api(self) -> bool {
        matches!(self, AppRole::All | AppRole::Web)
    }

    /// Whether this process runs the job worker and the scheduler.
    #[must_use]
    pub fn runs_jobs(self) -> bool {
        matches!(self, AppRole::All | AppRole::Worker)
    }

    /// Its `APP_ROLE` value.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            AppRole::All => "all",
            AppRole::Web => "web",
            AppRole::Worker => "worker",
        }
    }
}

/// How mail leaves the application (`MAIL_TRANSPORT`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailTransport {
    /// `log`: each mail is logged, not sent. The default, for development.
    Log,
    /// `smtp`: sent through `SMTP_URL`.
    Smtp,
}

/// The application's settings. It has no `Debug`, so the secrets can't end
/// up in a log; [`Config::summary`] is the loggable view.
pub struct Config {
    /// `DATABASE_URL`.
    pub database_url: String,
    /// `APP_ROLE`: what this process runs.
    pub app_role: AppRole,
    /// `METRICS_ADDR`: where `/metrics` is served; `None`, nowhere.
    pub metrics_addr: Option<SocketAddr>,
    /// `BIND_ADDR`.
    pub bind_addr: String,
    /// `JWT_SECRET`.
    pub jwt_secret: String,
    /// `JWT_TTL`: how long an issued token lasts.
    pub jwt_ttl: Duration,
    /// `JWT_SECRET_PREVIOUS`: the secret being rotated out, whose tokens
    /// still pass until they expire.
    pub jwt_secret_previous: Option<String>,
    /// `TOTP_ENCRYPTION_KEY`: the key two-factor secrets are encrypted
    /// with at rest (AES-256-GCM).
    pub totp_encryption_key: [u8; TOTP_KEY_BYTES],
    /// `REQUIRE_VERIFIED_EMAIL`: sign-in answers `403` until the email is
    /// verified.
    pub require_verified_email: bool,
    /// `APP_URL`: the base of mailed links, such as
    /// `{app_url}/verify-email?token=…`.
    pub app_url: String,
    /// `MAIL_TRANSPORT`.
    pub mail_transport: MailTransport,
    /// `SMTP_URL`, set whenever `mail_transport` is `Smtp`.
    pub smtp_url: Option<String>,
    /// `MAIL_FROM`.
    pub mail_from: String,
    /// `BREACH_CHECK`: check new passwords against Have I Been Pwned.
    pub breach_check: bool,
    /// `TRUST_PROXY`: take the client IP from `X-Forwarded-For`.
    pub trust_proxy: bool,
    /// `AUTH_RATE_LIMIT`: requests per minute per IP on `/auth`; `0` turns
    /// the limit off.
    pub auth_rate_limit: u32,
    /// `GLOBAL_RATE_LIMIT`: requests per minute per IP on every route but
    /// health; `0` (the default) turns the limit off.
    pub global_rate_limit: u32,
    /// `JOBS_CONCURRENCY`: jobs the worker runs at once, each holding a
    /// pool connection while it runs.
    pub jobs_concurrency: usize,
    /// `SHUTDOWN_DELAY_SECS` and `SHUTDOWN_GRACE_SECS`: how the process
    /// stops.
    pub shutdown: ShutdownSettings,
    /// `CORS_ALLOWED_ORIGINS`: the other origins a browser may call the API
    /// from.
    pub cors_allowed_origins: CorsOrigins,
    /// `STORAGE_URL`: where uploaded files are stored.
    pub storage_url: StorageUrl,
    /// `MEDIA_MAX_UPLOAD_BYTES`: the largest upload, in bytes.
    pub media_max_upload_bytes: usize,
    /// `MEDIA_ALLOWED_TYPES`: the types, detected from the bytes, an upload
    /// may have (comma-separated).
    pub media_allowed_types: Vec<String>,
    /// `MEDIA_UPLOAD_URL_TTL_SECS`: how long a direct-upload URL works, and
    /// so how long its upload intent waits for completion.
    pub media_upload_url_ttl: Duration,
    /// `MEDIA_DOWNLOAD_URL_TTL_SECS`: how long a download URL works.
    pub media_download_url_ttl: Duration,
    /// `STORAGE_PUBLIC_ENDPOINT`: the endpoint an S3-compatible store's URLs
    /// are signed for, when clients reach it elsewhere than the API does.
    pub storage_public_endpoint: Option<String>,
    /// The pool's size and timeouts, `DB_*`.
    pub pool: PoolSettings,
}

impl Config {
    /// Reads every setting from the environment.
    pub fn from_env() -> Result<Self, ConfigErrors> {
        Config::from_vars(|name| env::var(name).ok())
    }

    /// Reads every setting through `var`, which gives a variable's value
    /// when it is set. A wrong setting doesn't stop the reading: each is
    /// recorded, and the result is every one of them.
    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigErrors> {
        let vars = Vars::new(&var);
        let mut pool = PoolSettings::default();
        if let Some(max) = vars.number("DB_MAX_CONNECTIONS") {
            pool.max_connections = max;
        }
        if let Some(min) = vars.number("DB_MIN_CONNECTIONS") {
            pool.min_connections = min;
        }
        if let Some(secs) = vars.number("DB_ACQUIRE_TIMEOUT_SECS") {
            pool.acquire_timeout = Duration::from_secs(secs);
        }
        if let Some(secs) = vars.number("DB_IDLE_TIMEOUT_SECS") {
            pool.idle_timeout = Duration::from_secs(secs);
        }
        if let Some(secs) = vars.number("DB_MAX_LIFETIME_SECS") {
            pool.max_lifetime = Duration::from_secs(secs);
        }
        let database_url = vars.required("DATABASE_URL");
        let jwt_secret = vars.secret("JWT_SECRET", MIN_JWT_SECRET_BYTES);
        let jwt_ttl = vars.number("JWT_TTL").unwrap_or(DEFAULT_JWT_TTL_SECS);
        let jwt_secret_previous = vars.optional_secret("JWT_SECRET_PREVIOUS", MIN_JWT_SECRET_BYTES);
        let totp_encryption_key = vars.key("TOTP_ENCRYPTION_KEY");
        let mail_transport = match var("MAIL_TRANSPORT").as_deref() {
            None | Some("log") => MailTransport::Log,
            Some("smtp") => MailTransport::Smtp,
            Some(other) => {
                vars.fail(ConfigError::NotOneOf {
                    name: "MAIL_TRANSPORT",
                    value: other.to_owned(),
                    expected: "log, smtp",
                });
                MailTransport::Log
            }
        };
        let smtp_url = match mail_transport {
            MailTransport::Smtp => Some(vars.required("SMTP_URL")),
            MailTransport::Log => var("SMTP_URL"),
        };
        let config = Config {
            database_url,
            bind_addr: var("BIND_ADDR").unwrap_or_else(|| DEFAULT_BIND_ADDR.to_owned()),
            app_role: match var("APP_ROLE").as_deref().map(str::trim) {
                None | Some("" | "all") => AppRole::All,
                Some("web") => AppRole::Web,
                Some("worker") => AppRole::Worker,
                Some(other) => {
                    vars.fail(ConfigError::NotOneOf {
                        name: "APP_ROLE",
                        value: other.to_owned(),
                        expected: "all, web, worker",
                    });
                    AppRole::All
                }
            },
            metrics_addr: var("METRICS_ADDR")
                .filter(|addr| !addr.trim().is_empty())
                .and_then(|addr| {
                    let parsed = addr.trim().parse().ok();
                    if parsed.is_none() {
                        vars.fail(ConfigError::NotOneOf {
                            name: "METRICS_ADDR",
                            value: addr,
                            expected: "an address and port, such as 0.0.0.0:9090",
                        });
                    }
                    parsed
                }),
            jwt_secret,
            jwt_ttl: Duration::from_secs(jwt_ttl),
            jwt_secret_previous,
            totp_encryption_key,
            require_verified_email: vars.flag("REQUIRE_VERIFIED_EMAIL").unwrap_or(true),
            app_url: var("APP_URL").unwrap_or_else(|| DEFAULT_APP_URL.to_owned()),
            mail_transport,
            smtp_url,
            mail_from: var("MAIL_FROM").unwrap_or_else(|| DEFAULT_MAIL_FROM.to_owned()),
            breach_check: vars.flag("BREACH_CHECK").unwrap_or(false),
            trust_proxy: vars.flag("TRUST_PROXY").unwrap_or(false),
            auth_rate_limit: vars
                .number("AUTH_RATE_LIMIT")
                .unwrap_or(DEFAULT_AUTH_RATE_LIMIT),
            global_rate_limit: vars.number("GLOBAL_RATE_LIMIT").unwrap_or(0),
            jobs_concurrency: vars
                .number("JOBS_CONCURRENCY")
                .unwrap_or(DEFAULT_JOBS_CONCURRENCY),
            shutdown: ShutdownSettings {
                delay: Duration::from_secs(
                    vars.number("SHUTDOWN_DELAY_SECS")
                        .unwrap_or(DEFAULT_SHUTDOWN_DELAY_SECS),
                ),
                grace: Duration::from_secs(
                    vars.number("SHUTDOWN_GRACE_SECS")
                        .unwrap_or(DEFAULT_SHUTDOWN_GRACE_SECS),
                ),
            },
            cors_allowed_origins: match var("CORS_ALLOWED_ORIGINS") {
                Some(value) => value.parse().unwrap_or_else(|_| {
                    vars.fail(ConfigError::NotOneOf {
                        name: "CORS_ALLOWED_ORIGINS",
                        value,
                        expected: "*, or origins such as https://app.example.com",
                    });
                    CorsOrigins::SameOrigin
                }),
                None => CorsOrigins::SameOrigin,
            },
            storage_url: match var("STORAGE_URL") {
                Some(url) => url.parse().unwrap_or_else(|_| {
                    vars.fail(ConfigError::NotOneOf {
                        name: "STORAGE_URL",
                        value: url,
                        expected: "file://<dir>, s3://<bucket>, memory://",
                    });
                    StorageUrl::default_for_development()
                }),
                None => StorageUrl::default_for_development(),
            },
            media_max_upload_bytes: vars
                .number("MEDIA_MAX_UPLOAD_BYTES")
                .unwrap_or(DEFAULT_MEDIA_MAX_UPLOAD_BYTES),
            media_allowed_types: match var("MEDIA_ALLOWED_TYPES") {
                Some(list) => list
                    .split(',')
                    .map(str::trim)
                    .filter(|kind| !kind.is_empty())
                    .map(str::to_owned)
                    .collect(),
                None => DEFAULT_MEDIA_ALLOWED_TYPES
                    .iter()
                    .map(|kind| (*kind).to_owned())
                    .collect(),
            },
            media_upload_url_ttl: Duration::from_secs(
                vars.number("MEDIA_UPLOAD_URL_TTL_SECS")
                    .unwrap_or(DEFAULT_MEDIA_UPLOAD_URL_TTL_SECS),
            ),
            media_download_url_ttl: Duration::from_secs(
                vars.number("MEDIA_DOWNLOAD_URL_TTL_SECS")
                    .unwrap_or(DEFAULT_MEDIA_DOWNLOAD_URL_TTL_SECS),
            ),
            storage_public_endpoint: var("STORAGE_PUBLIC_ENDPOINT").filter(|url| !url.is_empty()),
            pool,
        };
        vars.finish(config)
    }

    /// Whether the API is served over HTTPS (`APP_URL` is `https://…`), so
    /// answers carry `Strict-Transport-Security`.
    #[must_use]
    pub fn https(&self) -> bool {
        self.app_url.starts_with("https://")
    }

    /// The settings as `NAME=value` pairs, one per line, for the start-up
    /// log: every secret is its length only (`JWT_SECRET=<34 bytes>`), and
    /// a URL's password is `<redacted>`.
    #[must_use]
    pub fn summary(&self) -> String {
        let secs = |duration: Duration| duration.as_secs().to_string();
        let flag = |on: bool| on.to_string();
        let unset = || "<unset>".to_owned();
        let settings = [
            ("DATABASE_URL", redact_url(&self.database_url)),
            ("APP_ROLE", self.app_role.name().to_owned()),
            ("BIND_ADDR", self.bind_addr.clone()),
            (
                "METRICS_ADDR",
                self.metrics_addr
                    .map_or_else(unset, |addr| addr.to_string()),
            ),
            ("JWT_SECRET", format!("<{} bytes>", self.jwt_secret.len())),
            ("JWT_TTL", secs(self.jwt_ttl)),
            (
                "JWT_SECRET_PREVIOUS",
                self.jwt_secret_previous
                    .as_ref()
                    .map_or_else(unset, |secret| format!("<{} bytes>", secret.len())),
            ),
            (
                "TOTP_ENCRYPTION_KEY",
                format!("<{} bytes>", self.totp_encryption_key.len()),
            ),
            ("REQUIRE_VERIFIED_EMAIL", flag(self.require_verified_email)),
            ("APP_URL", self.app_url.clone()),
            (
                "MAIL_TRANSPORT",
                match self.mail_transport {
                    MailTransport::Log => "log".to_owned(),
                    MailTransport::Smtp => "smtp".to_owned(),
                },
            ),
            (
                "SMTP_URL",
                self.smtp_url.as_deref().map_or_else(unset, redact_url),
            ),
            ("MAIL_FROM", self.mail_from.clone()),
            ("BREACH_CHECK", flag(self.breach_check)),
            ("TRUST_PROXY", flag(self.trust_proxy)),
            ("AUTH_RATE_LIMIT", self.auth_rate_limit.to_string()),
            ("GLOBAL_RATE_LIMIT", self.global_rate_limit.to_string()),
            ("JOBS_CONCURRENCY", self.jobs_concurrency.to_string()),
            ("SHUTDOWN_DELAY_SECS", secs(self.shutdown.delay)),
            ("SHUTDOWN_GRACE_SECS", secs(self.shutdown.grace)),
            (
                "CORS_ALLOWED_ORIGINS",
                self.cors_allowed_origins.to_string(),
            ),
            ("STORAGE_URL", self.storage_url.to_string()),
            (
                "MEDIA_MAX_UPLOAD_BYTES",
                self.media_max_upload_bytes.to_string(),
            ),
            ("MEDIA_ALLOWED_TYPES", self.media_allowed_types.join(",")),
            ("MEDIA_UPLOAD_URL_TTL_SECS", secs(self.media_upload_url_ttl)),
            (
                "MEDIA_DOWNLOAD_URL_TTL_SECS",
                secs(self.media_download_url_ttl),
            ),
            (
                "STORAGE_PUBLIC_ENDPOINT",
                self.storage_public_endpoint.clone().unwrap_or_else(unset),
            ),
            ("DB_MAX_CONNECTIONS", self.pool.max_connections.to_string()),
            ("DB_MIN_CONNECTIONS", self.pool.min_connections.to_string()),
            ("DB_ACQUIRE_TIMEOUT_SECS", secs(self.pool.acquire_timeout)),
            ("DB_IDLE_TIMEOUT_SECS", secs(self.pool.idle_timeout)),
            ("DB_MAX_LIFETIME_SECS", secs(self.pool.max_lifetime)),
        ];
        settings
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// `url` with the password of its user info, if it has one, as
/// `<redacted>`: `postgres://app:<redacted>@db/app`.
fn redact_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_owned();
    };
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let Some((user_info, _)) = rest[..authority_end].rsplit_once('@') else {
        return url.to_owned();
    };
    match user_info.split_once(':') {
        Some((user, _)) => format!("{scheme}://{user}:<redacted>{}", &rest[user_info.len()..]),
        None => url.to_owned(),
    }
}
