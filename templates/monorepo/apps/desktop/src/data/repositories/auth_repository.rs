//! `AuthRepository`: what the `auth` business logic reads and writes.

use crate::data::models::user::User;
use crate::data::providers::api::{app_error, Api};
use api_client::{AccountResponse, CredentialsDto, LoginResponse};
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;

/// What the `auth` business logic reads and writes. A trait, so tests and other data sources can stand in.
pub trait AuthRepository: Send + Sync {
    /// Sign in.
    fn sign_in(&self, email: String, password: String) -> BoxFuture<'_, Result<User, ApiError>>;
    /// Sign out.
    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>>;
    /// Me.
    fn me(&self) -> BoxFuture<'_, Result<User, ApiError>>;
}

/// [`AuthRepository`] over the [`Api`].
#[derive(Debug)]
pub struct AuthRepositoryImpl {
    provider: Api,
}

impl AuthRepositoryImpl {
    /// A repository reading and writing through `provider`.
    #[must_use]
    pub fn new(provider: Api) -> Self {
        Self { provider }
    }

    /// The data source, for the methods you add.
    #[must_use]
    pub fn provider(&self) -> &Api {
        &self.provider
    }
}

impl AuthRepository for AuthRepositoryImpl {
    fn sign_in(&self, email: String, password: String) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            // Signing in sends no token, so its 401 (wrong credentials) never expires the session.
            let answer = self
                .provider
                .client()
                .auth()
                .login(&CredentialsDto { email, password })
                .await
                .map_err(app_error)?;
            let token = match answer {
                LoginResponse::TokenResponse(token) => token.access_token,
                LoginResponse::PendingToken(_) => {
                    return Err(ApiError::new(
                        0,
                        "totp_required",
                        "This account signs in with a two-factor code, which this app does not ask for.",
                    ))
                }
            };
            self.provider.sign_in(token);
            self.me().await
        })
    }

    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async move {
            // The API's sessions are bearer tokens it does not keep: signing out forgets ours.
            self.provider.sign_out();
            Ok(())
        })
    }

    fn me(&self) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            self.provider
                .client()
                .auth()
                .me()
                .await
                .map(Into::into)
                .map_err(app_error)
        })
    }
}

impl From<AccountResponse> for User {
    fn from(account: AccountResponse) -> Self {
        Self {
            id: account.id,
            name: account.display_name.unwrap_or_default(),
            email: account.email,
        }
    }
}
