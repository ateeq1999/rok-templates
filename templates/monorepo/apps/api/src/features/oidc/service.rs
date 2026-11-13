// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! `OidcService`: sign-in with an `OpenID` provider's rules (workspace ADR
//! 0035). A Linked identity is matched by its provider and subject, never
//! by email. An unknown identity with a new email signs up; one whose
//! email an account has is refused, unless the provider is trusted with
//! emails (`OIDC_<NAME>_TRUST_EMAIL`) and verified it, which links it. An
//! account with two-factor on still gives its code. Unlinking never takes
//! an account's last way in.
//!
//! The provider is reached through `shared::oidc`'s `IdentityProvider`;
//! every write goes through this feature's query module,
//! `db::queries::oidc`.

use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use rok_data::Model;
use sqlx::PgPool;

use super::dto::{CallbackQuery, IdentityResponse, LinkResponse};
use crate::db::oidc_requests::models::OidcRequestId;
use crate::db::queries::oidc::{self as queries, InsertIdentityParams, InsertRequestParams};
use crate::db::types::AuthTokenPurpose;
use crate::db::user_identities::models::UserIdentityId;
use crate::db::users::models::{User, UserId};
use crate::error::AppError;
use crate::shared::oidc::{Authorization, Identity, IdentityProviders, Redemption, not_configured};
use crate::shared::security::auth_token::{issue_token, policy};
use crate::shared::security::jwt::JwtKeys;
use crate::shared::security::token::{self, SecretToken};

/// How long a sign-in waits at the provider for its callback, in minutes.
pub const REQUEST_TTL_MINUTES: i64 = 10;

/// The `oidc` service, built by `app/`.
#[derive(Clone)]
pub struct OidcService {
    /// The pool the flows read and write through.
    pub pool: PgPool,
    /// Signs the session a sign-in answers.
    pub keys: Arc<JwtKeys>,
    /// The configured providers (`OIDC_PROVIDERS`); none, and every route
    /// answers `404`.
    pub providers: Arc<IdentityProviders>,
    /// `APP_URL`: the callback is `{app_url}/api/v1/auth/oidc/{provider}/callback`,
    /// and it sends the browser on to `{app_url}/auth/oidc`.
    pub app_url: String,
    /// `REQUIRE_VERIFIED_EMAIL`: a sign-in needs a verified email.
    pub require_verified_email: bool,
}

/// How a callback ends, carried to the App in the redirect's fragment.
enum Outcome {
    /// Signed in: a session.
    Session {
        /// The bearer token.
        access_token: String,
        /// Seconds until it expires.
        expires_in: i64,
    },
    /// The account has two-factor on: the token `POST /auth/2fa/verify`
    /// takes with the code.
    TotpRequired {
        /// The pending sign-in's token.
        pending_token: String,
    },
    /// An identity was linked to the signed-in account.
    Linked {
        /// The provider's name.
        provider: String,
    },
    /// Not signed in, and why.
    Refused {
        /// A `snake_case` reason the App can act on.
        error: &'static str,
        /// The reason, for people.
        message: String,
    },
}

/// A refusal `error`, saying `message`.
fn refused(error: &'static str, message: impl Into<String>) -> Outcome {
    Outcome::Refused {
        error,
        message: message.into(),
    }
}

/// Starting a sign-in or a link, and the callback.
impl OidcService {
    /// Starts signing in at the provider `name`: its authorization URL, to
    /// send the browser to. No such provider, or none configured, is a
    /// `404`.
    pub async fn start(&self, name: &str) -> Result<String, AppError> {
        self.authorize(name, None).await
    }

    /// Starts linking an identity at the provider `name` to the signed-in
    /// account `user`: its authorization URL, to send the browser to.
    pub async fn link(&self, user: UserId, name: &str) -> Result<LinkResponse, AppError> {
        Ok(LinkResponse {
            url: self.authorize(name, Some(user)).await?,
        })
    }

    /// The provider `name`'s redirect back: where to send the browser
    /// next, `{app_url}/auth/oidc#…` with a session (`access_token`,
    /// `expires_in`, `token_type`), a two-factor step (`requires_totp`,
    /// `pending_token`), a link (`linked`), or a refusal (`error`,
    /// `message`). Only an unknown provider (a `404`) or a failure of the
    /// API itself is an error.
    pub async fn callback(&self, name: &str, query: CallbackQuery) -> Result<String, AppError> {
        let provider = self.providers.get(name)?;
        let outcome = if let Some(error) = query.error {
            refused(
                "provider_refused",
                format!("{name} didn't sign you in ({error})"),
            )
        } else if let (Some(code), Some(state)) = (query.code, query.state) {
            match queries::take_request(&self.pool, name, &token::hash(&state)).await? {
                None => refused(
                    "invalid_request",
                    "the sign-in expired or was already used: start it again",
                ),
                Some(request) => {
                    let redemption = Redemption {
                        code: &code,
                        redirect_uri: &self.redirect_uri(name),
                        pkce_verifier: &request.pkce_verifier,
                        nonce: &request.nonce,
                    };
                    match provider.connection.redeem(redemption).await {
                        Err(AppError::BadRequest(message)) => refused("provider_refused", message),
                        Err(error) => return Err(error),
                        Ok(identity) => match request.user_id {
                            Some(user) => self.link_identity(user, name, identity).await?,
                            None => self.sign_in(name, provider.trust_email, identity).await?,
                        },
                    }
                }
            }
        } else {
            refused("invalid_request", "the provider's redirect has no code")
        };
        Ok(self.app_redirect(outcome))
    }

    /// The authorization URL of a sign-in (`user` none) or a link at the
    /// provider `name`, its single-use state, PKCE verifier and nonce
    /// stored for the callback.
    async fn authorize(&self, name: &str, user: Option<UserId>) -> Result<String, AppError> {
        let provider = self.providers.get(name)?;
        let state = SecretToken::generate();
        let pkce_verifier = SecretToken::generate().raw;
        let nonce = SecretToken::generate().raw;
        let url = provider
            .connection
            .authorize_url(Authorization {
                redirect_uri: &self.redirect_uri(name),
                state: &state.raw,
                nonce: &nonce,
                pkce_verifier: &pkce_verifier,
            })
            .await?;
        queries::forget_expired_requests(&self.pool).await?;
        queries::insert_request(
            &self.pool,
            InsertRequestParams {
                id: OidcRequestId::generate(),
                provider: name.to_owned(),
                state_hash: state.hash,
                pkce_verifier,
                nonce,
                user_id: user,
                expires_at: Utc::now() + TimeDelta::minutes(REQUEST_TTL_MINUTES),
            },
        )
        .await?;
        Ok(url)
    }

    /// Signs `identity` at the provider `name` in: as the account it is
    /// linked to; as the account with its email when the provider is
    /// `trust_email` and verified it (linking it); or as a new account when
    /// no account has its email.
    async fn sign_in(
        &self,
        name: &str,
        trust_email: bool,
        identity: Identity,
    ) -> Result<Outcome, AppError> {
        if let Some(user) = queries::find_identity(&self.pool, name, &identity.subject).await? {
            return self.session_for(user).await;
        }
        let Some(email) = identity.email.as_deref().map(normalized) else {
            return Ok(refused(
                "no_email",
                format!("{name} shared no email address"),
            ));
        };
        if let Some(user) = queries::find_user_by_email(&self.pool, &email).await? {
            if !(trust_email && identity.email_verified) {
                return Ok(refused(
                    "email_in_use",
                    format!(
                        "an account already uses {email}: sign in to it, then link {name} from its settings"
                    ),
                ));
            }
            queries::insert_identity(&self.pool, linking(user, name, &identity)).await?;
            return self.session_for(user).await;
        }
        if self.require_verified_email && !identity.email_verified {
            return Ok(refused(
                "email_not_verified",
                format!("{name} hasn't verified {email}"),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let user = queries::create_user(
            &mut *tx,
            UserId::generate(),
            &email,
            identity.email_verified,
        )
        .await?;
        queries::insert_identity(&mut *tx, linking(user, name, &identity)).await?;
        tx.commit().await?;
        self.session_for(user).await
    }

    /// Links `identity` at the provider `name` to the account `user`,
    /// unless another account has it.
    async fn link_identity(
        &self,
        user: UserId,
        name: &str,
        identity: Identity,
    ) -> Result<Outcome, AppError> {
        match queries::find_identity(&self.pool, name, &identity.subject).await? {
            Some(owner) if owner != user => Ok(refused(
                "identity_in_use",
                format!("this {name} account is linked to another account"),
            )),
            Some(_) => Ok(Outcome::Linked {
                provider: name.to_owned(),
            }),
            None => {
                queries::insert_identity(&self.pool, linking(user, name, &identity)).await?;
                Ok(Outcome::Linked {
                    provider: name.to_owned(),
                })
            }
        }
    }

    /// The sign-in of the account `id`, as a password's would end: refused
    /// when it is gone, disabled, locked or (with `REQUIRE_VERIFIED_EMAIL`)
    /// unverified, and a two-factor step when it has two-factor on.
    async fn session_for(&self, id: UserId) -> Result<Outcome, AppError> {
        let Some(user) = User::find(&self.pool, id).await? else {
            return Ok(refused("account_gone", "the account no longer exists"));
        };
        if !user.is_active {
            return Ok(refused("forbidden", "the account is disabled"));
        }
        if user.locked_until.is_some_and(|until| until > Utc::now()) {
            return Ok(refused("locked", "the account is temporarily locked"));
        }
        if self.require_verified_email && !user.email_verified {
            return Ok(refused(
                "email_not_verified",
                "the email address is not verified yet",
            ));
        }
        if user.totp_enabled {
            let purpose = AuthTokenPurpose::TwoFactorPending;
            let mut tx = self.pool.begin().await?;
            let pending = issue_token(&mut tx, user.id, purpose, policy(purpose).0, None).await?;
            tx.commit().await?;
            return Ok(Outcome::TotpRequired {
                pending_token: pending.raw,
            });
        }
        Ok(Outcome::Session {
            access_token: self
                .keys
                .issue(user.id.into(), user.role, user.token_version)?,
            expires_in: i64::try_from(self.keys.ttl().as_secs()).unwrap_or(i64::MAX),
        })
    }

    /// The callback URL registered with the provider `name`.
    fn redirect_uri(&self, name: &str) -> String {
        format!(
            "{}/api/v1/auth/oidc/{name}/callback",
            self.app_url.trim_end_matches('/')
        )
    }

    /// `{app_url}/auth/oidc#…`, the fragment carrying `outcome`: a fragment
    /// never reaches a server, so the token stays out of every log.
    fn app_redirect(&self, outcome: Outcome) -> String {
        let mut fragment = url::form_urlencoded::Serializer::new(String::new());
        match outcome {
            Outcome::Session {
                access_token,
                expires_in,
            } => fragment
                .append_pair("access_token", &access_token)
                .append_pair("expires_in", &expires_in.to_string())
                .append_pair("token_type", "Bearer"),
            Outcome::TotpRequired { pending_token } => fragment
                .append_pair("requires_totp", "true")
                .append_pair("pending_token", &pending_token),
            Outcome::Linked { provider } => fragment.append_pair("linked", &provider),
            Outcome::Refused { error, message } => fragment
                .append_pair("error", error)
                .append_pair("message", &message),
        };
        format!(
            "{}/auth/oidc#{}",
            self.app_url.trim_end_matches('/'),
            fragment.finish()
        )
    }
}

/// The signed-in account's Linked identities.
impl OidcService {
    /// The account `user`'s Linked identities, oldest first.
    pub async fn identities(&self, user: UserId) -> Result<Vec<IdentityResponse>, AppError> {
        if self.providers.is_empty() {
            return Err(not_configured());
        }
        Ok(queries::list_identities(&self.pool, user)
            .await?
            .into_iter()
            .map(|row| IdentityResponse {
                id: row.id,
                provider: row.provider,
                email: row.email,
                created_at: row.created_at,
            })
            .collect())
    }

    /// Unlinks the account `user`'s identity `id`. Not the account's (or no
    /// such identity) is a `404`; its last way in, with no password and no
    /// other identity, is a `422` saying why.
    pub async fn unlink(&self, user: UserId, id: UserIdentityId) -> Result<(), AppError> {
        if self.providers.is_empty() {
            return Err(not_configured());
        }
        if queries::find_linked(&self.pool, id, user).await?.is_none() {
            return Err(AppError::NotFound("no such linked identity".to_owned()));
        }
        if queries::unlink(&self.pool, id, user).await? == 0 {
            return Err(AppError::Validation {
                message: "this is the account's only way to sign in: set a password or link \
                          another identity first"
                    .to_owned(),
                details: None,
            });
        }
        Ok(())
    }
}

/// What linking `identity` at the provider `name` to `user` inserts.
fn linking(user: UserId, name: &str, identity: &Identity) -> InsertIdentityParams {
    InsertIdentityParams {
        id: UserIdentityId::generate(),
        user_id: user,
        provider: name.to_owned(),
        subject: identity.subject.clone(),
        email: identity.email.as_deref().map(normalized),
    }
}

/// An email as accounts keep one: trimmed and lowercase.
fn normalized(email: &str) -> String {
    email.trim().to_lowercase()
}
