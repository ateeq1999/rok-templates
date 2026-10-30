// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`AppState`]: what the hand-written handlers reach through `State`,
//! built once from the [`Config`].

use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::PgPool;

use super::ports::AuditLog;
use crate::config::Config;
use crate::features::account::AccountService;
use crate::features::api_keys::ApiKeyService;
use crate::features::audit_log_entry::AuditLogEntryService;
use crate::features::auth::AuthService;
use crate::features::auth::service::AuditTrail;
use crate::features::jobs::JobService;
use crate::features::media::MediaService;
use crate::features::two_factor::TwoFactorService;
use crate::shared::jobs::Queue;
use crate::shared::mail::{Outbox, Templates};
use crate::shared::rate_limit::RateLimit;
use crate::shared::security::breach::{self, BreachCheck};
use crate::shared::security::headers::CorsOrigins;
use crate::shared::security::jwt::JwtKeys;
use crate::shared::security::totp::TotpCipher;
use crate::shared::shutdown::{Draining, ShutdownSettings};
use crate::shared::storage::{self, Storage};

/// What the hand-written handlers reach through `State`, built once: by
/// [`run`](super::run), or by a test. Each field is an extractable state of
/// its own (`FromRef`): the `auth` and `account` handlers take their
/// services from `auth` and `account`. The generated features build theirs
/// in `features::router` (`features/generated.rs`), once, not per request.
#[derive(Clone, FromRef)]
pub struct AppState {
    /// The database, which `/health/ready` checks and `AuthUser` reads the
    /// session from.
    pub pool: PgPool,
    /// Signs and verifies session tokens.
    pub keys: Arc<JwtKeys>,
    /// Where flows enqueue jobs; it wakes the job worker.
    pub queue: Queue,
    /// Where flows enqueue their mail: `mail.send` jobs on `queue`.
    pub outbox: Outbox,
    /// The messages flows enqueue, with links under `APP_URL`.
    pub templates: Arc<Templates>,
    /// The per-IP limit for `/auth` (`AUTH_RATE_LIMIT`, `TRUST_PROXY`),
    /// one for every route it wraps.
    pub rate_limit: RateLimit,
    /// `auth`'s service, over the parts above, the breach check
    /// (`BREACH_CHECK`), `REQUIRE_VERIFIED_EMAIL` and the audit trail (the
    /// `audit_log` feature). A test swaps the breach check or the audit
    /// trail with [`AppState::with_breach`] or [`AppState::with_audit`].
    pub auth: Arc<AuthService>,
    /// `two_factor`'s service, over the parts above and the cipher
    /// (`TOTP_ENCRYPTION_KEY`).
    pub two_factor: Arc<TwoFactorService>,
    /// `account`'s service, its profile photos in `storage`.
    pub account: Arc<AccountService>,
    /// `jobs`'s service: the queue, for admins.
    pub jobs: Arc<JobService>,
    /// `api_keys`'s service.
    pub api_keys: Arc<ApiKeyService>,
    /// Where uploaded bytes are stored: the store `STORAGE_URL` names,
    /// opened by [`AppState::new`]; a test may swap in its own with
    /// [`AppState::with_storage`].
    pub storage: Arc<dyn Storage>,
    /// `media`'s service, over `storage` and the `MEDIA_*` settings.
    pub media: Arc<MediaService>,
    /// The per-IP limit on every route but health (`GLOBAL_RATE_LIMIT`,
    /// off by default), counted apart from `rate_limit`.
    #[from_ref(skip)]
    pub global_rate_limit: RateLimit,
    /// The other origins a browser may call the API from
    /// (`CORS_ALLOWED_ORIGINS`).
    #[from_ref(skip)]
    pub cors_allowed_origins: CorsOrigins,
    /// Whether answers carry `Strict-Transport-Security` (`APP_URL` is
    /// `https://…`).
    #[from_ref(skip)]
    pub https: bool,
    /// How many jobs the worker runs at once (`JOBS_CONCURRENCY`).
    #[from_ref(skip)]
    pub jobs_concurrency: usize,
    /// Set on `SIGTERM`: `/health/ready` answers `503` from then on.
    #[from_ref(skip)]
    pub draining: Draining,
    /// How the process stops (`SHUTDOWN_DELAY_SECS`, `SHUTDOWN_GRACE_SECS`).
    #[from_ref(skip)]
    pub shutdown: ShutdownSettings,
}

impl AppState {
    /// The state `config` describes, over `pool`, storing uploads where
    /// `STORAGE_URL` says (a test configures `memory://`). A store that
    /// can't be opened, such as a `file://` directory that can't be
    /// created, is an error.
    pub fn new(pool: PgPool, config: &Config) -> Result<Self, object_store::Error> {
        let keys = Arc::new(
            JwtKeys::new(&config.jwt_secret, config.jwt_ttl)
                .with_previous(config.jwt_secret_previous.as_deref()),
        );
        let queue = Queue::new();
        let outbox = Outbox::on(queue.clone());
        let templates = Arc::new(Templates::new(&config.app_url));
        let auth = Arc::new(AuthService {
            pool: pool.clone(),
            keys: keys.clone(),
            outbox: Arc::new(outbox.clone()),
            templates: templates.clone(),
            breach: breach::from_config(config),
            require_verified_email: config.require_verified_email,
            audit: Arc::new(AuditLog(AuditLogEntryService::new(pool.clone()))),
        });
        let two_factor = Arc::new(TwoFactorService {
            pool: pool.clone(),
            keys: keys.clone(),
            outbox: Arc::new(outbox.clone()),
            templates: templates.clone(),
            totp: Arc::new(TotpCipher::new(&config.totp_encryption_key)),
        });
        // The API signs the URLs of a store that can't sign its own, under a
        // key derived from `JWT_SECRET` (engine ADR 0024).
        let signing = storage::Signing {
            api: storage::ApiSigner::from_secret(config.jwt_secret.as_bytes()),
            public_endpoint: config.storage_public_endpoint.clone(),
        };
        let storage = storage::open(&config.storage_url, signing)?;
        let media = Arc::new(MediaService {
            pool: pool.clone(),
            storage: storage.clone(),
            max_upload_bytes: config.media_max_upload_bytes,
            allowed_types: config.media_allowed_types.clone(),
            upload_url_ttl: config.media_upload_url_ttl,
            download_url_ttl: config.media_download_url_ttl,
        });
        let account = Arc::new(AccountService::new(pool.clone(), storage.clone()));
        Ok(AppState {
            storage,
            media,
            account,
            api_keys: Arc::new(ApiKeyService::new(pool.clone())),
            jobs: Arc::new(JobService {
                pool: pool.clone(),
                queue: queue.clone(),
                audit: Arc::new(AuditLog(AuditLogEntryService::new(pool.clone()))),
            }),
            auth,
            two_factor,
            pool,
            keys,
            outbox,
            queue,
            jobs_concurrency: config.jobs_concurrency,
            draining: Draining::default(),
            shutdown: config.shutdown,
            templates,
            rate_limit: RateLimit::per_minute(config.auth_rate_limit, config.trust_proxy),
            global_rate_limit: RateLimit::per_minute(config.global_rate_limit, config.trust_proxy),
            cors_allowed_origins: config.cors_allowed_origins.clone(),
            https: config.https(),
        })
    }

    /// This state, its `auth` checking new passwords with `breach`.
    #[must_use]
    pub fn with_breach(mut self, breach: Arc<dyn BreachCheck>) -> Self {
        Arc::make_mut(&mut self.auth).breach = breach;
        self
    }

    /// This state, storing uploads in `storage`.
    #[must_use]
    pub fn with_storage(mut self, storage: Arc<dyn Storage>) -> Self {
        Arc::make_mut(&mut self.media).storage = storage.clone();
        Arc::make_mut(&mut self.account).storage = storage.clone();
        self.storage = storage;
        self
    }

    /// This state, its `auth` recording what admins do with `audit`.
    #[must_use]
    pub fn with_audit(mut self, audit: Arc<dyn AuditTrail>) -> Self {
        Arc::make_mut(&mut self.auth).audit = audit;
        self
    }
}
