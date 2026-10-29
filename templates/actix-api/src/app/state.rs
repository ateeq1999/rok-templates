// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`AppState`]: what every worker shares, built once from the [`Config`].

use std::sync::Arc;

use actix_web::web;
use sqlx::PgPool;

use super::ports::AuditLog;
use crate::config::Config;
use crate::features::audit_log_entry::AuditLogEntryService;
use crate::features::auth::service::AuditTrail;
use crate::features::media::MediaService;
use crate::shared::mail::{Outbox, Templates};
use crate::shared::rate_limit::RateLimit;
use crate::shared::security::breach::{self, BreachCheck};
use crate::shared::security::headers::CorsOrigins;
use crate::shared::security::jwt::JwtKeys;
use crate::shared::security::totp::TotpCipher;
use crate::shared::storage::{self, Storage};

/// What every worker shares, built once: by [`run`](super::run), or by a
/// test.
#[derive(Clone)]
pub struct AppState {
    /// The database.
    pub pool: PgPool,
    /// Signs and verifies session tokens.
    pub keys: web::Data<JwtKeys>,
    /// Where flows enqueue their mail; it wakes the sender.
    pub outbox: web::Data<Outbox>,
    /// The messages flows enqueue, with links under `APP_URL`.
    pub templates: web::Data<Templates>,
    /// Checks new passwords against known breaches (`BREACH_CHECK`); a
    /// test swaps in its own.
    pub breach: Arc<dyn BreachCheck>,
    /// The per-IP limit on `/auth` (`AUTH_RATE_LIMIT`, `TRUST_PROXY`),
    /// one for every worker.
    pub rate_limit: RateLimit,
    /// `REQUIRE_VERIFIED_EMAIL`.
    pub require_verified_email: bool,
    /// Where `auth` records what admins do: the `audit_log` feature, or a
    /// test's own.
    pub audit: Arc<dyn AuditTrail>,
    /// Where uploaded bytes are stored: the store `STORAGE_URL` names,
    /// opened by [`AppState::new`]; a test may swap in its own with
    /// [`AppState::with_storage`].
    pub storage: Arc<dyn Storage>,
    /// `MEDIA_MAX_UPLOAD_BYTES`.
    pub media_max_upload_bytes: usize,
    /// `MEDIA_ALLOWED_TYPES`.
    pub media_allowed_types: Vec<String>,
    /// `MEDIA_UPLOAD_URL_TTL_SECS`.
    pub media_upload_url_ttl: std::time::Duration,
    /// `MEDIA_DOWNLOAD_URL_TTL_SECS`.
    pub media_download_url_ttl: std::time::Duration,
    /// `MEDIA_UPLOAD_SWEEP_SECS`: how often [`run`](super::run) sweeps
    /// expired uploads.
    pub media_upload_sweep: Option<std::time::Duration>,
    /// Encrypts two-factor secrets at rest (`TOTP_ENCRYPTION_KEY`).
    pub totp: Arc<TotpCipher>,
    /// The per-IP limit on every route but health (`GLOBAL_RATE_LIMIT`,
    /// off by default), counted apart from `rate_limit`.
    pub global_rate_limit: RateLimit,
    /// The other origins a browser may call the API from
    /// (`CORS_ALLOWED_ORIGINS`).
    pub cors_allowed_origins: CorsOrigins,
    /// Whether answers carry `Strict-Transport-Security` (`APP_URL` is
    /// `https://…`).
    pub https: bool,
}

impl AppState {
    /// The state `config` describes, over `pool`, storing uploads where
    /// `STORAGE_URL` says (a test configures `memory://`). A store that
    /// can't be opened, such as a `file://` directory that can't be
    /// created, is an error.
    pub fn new(pool: PgPool, config: &Config) -> Result<Self, object_store::Error> {
        Ok(AppState {
            pool: pool.clone(),
            keys: web::Data::new(
                JwtKeys::new(&config.jwt_secret, config.jwt_ttl)
                    .with_previous(config.jwt_secret_previous.as_deref()),
            ),
            outbox: web::Data::new(Outbox::new()),
            templates: web::Data::new(Templates::new(&config.app_url)),
            breach: breach::from_config(config),
            rate_limit: RateLimit::per_minute(config.auth_rate_limit, config.trust_proxy),
            require_verified_email: config.require_verified_email,
            audit: Arc::new(AuditLog(AuditLogEntryService::new(pool))),
            storage: storage::open(&config.storage_url, signing(config))?,
            media_max_upload_bytes: config.media_max_upload_bytes,
            media_allowed_types: config.media_allowed_types.clone(),
            media_upload_url_ttl: config.media_upload_url_ttl,
            media_download_url_ttl: config.media_download_url_ttl,
            media_upload_sweep: config.media_upload_sweep,
            totp: Arc::new(TotpCipher::new(&config.totp_encryption_key)),
            global_rate_limit: RateLimit::per_minute(config.global_rate_limit, config.trust_proxy),
            cors_allowed_origins: config.cors_allowed_origins.clone(),
            https: config.https(),
        })
    }

    /// This state, storing uploads in `storage`.
    #[must_use]
    pub fn with_storage(mut self, storage: Arc<dyn Storage>) -> Self {
        self.storage = storage;
        self
    }

    /// `media`'s service, over the state's store and the `MEDIA_*` settings.
    pub(super) fn media_service(&self) -> MediaService {
        MediaService {
            pool: self.pool.clone(),
            storage: self.storage.clone(),
            max_upload_bytes: self.media_max_upload_bytes,
            allowed_types: self.media_allowed_types.clone(),
            upload_url_ttl: self.media_upload_url_ttl,
            download_url_ttl: self.media_download_url_ttl,
        }
    }
}

/// How the store's presigned URLs are signed: by the API under a key
/// derived from `JWT_SECRET`, unless the store signs its own (engine ADR 0024).
fn signing(config: &Config) -> storage::Signing {
    storage::Signing {
        api: storage::ApiSigner::from_secret(config.jwt_secret.as_bytes()),
        public_endpoint: config.storage_public_endpoint.clone(),
    }
}
