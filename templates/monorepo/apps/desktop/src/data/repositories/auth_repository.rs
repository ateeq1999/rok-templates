//! `AuthRepository`: what the `auth` business logic reads and writes.

use crate::data::models::sign_in::{PendingSignIn, SignIn};
use crate::data::models::user::User;
use crate::data::providers::api::{app_error, Api};
use api_client::{CredentialsDto, LoginResponse, VerifyTotpDto};
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;

/// What the `auth` business logic reads and writes. A trait, so tests and other data sources can stand in.
pub trait AuthRepository: Send + Sync {
    /// Creates an account (`POST /auth/register`) and answers its email. It does not sign in:
    /// `apps/api` mails a link to verify the email first.
    fn sign_up(&self, email: String, password: String) -> BoxFuture<'_, Result<String, ApiError>>;
    /// Signs in (`POST /auth/login`), or answers that the account's two-factor code is needed.
    fn sign_in(&self, email: String, password: String) -> BoxFuture<'_, Result<SignIn, ApiError>>;
    /// Finishes a two-factor sign-in with an authenticator or recovery `code`
    /// (`POST /auth/2fa/verify`).
    fn verify_code(
        &self,
        pending: PendingSignIn,
        code: String,
    ) -> BoxFuture<'_, Result<User, ApiError>>;
    /// Signs out.
    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>>;
    /// The signed-in user (`GET /auth/me`).
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

    /// Signs in with `token`, then reads who that is.
    async fn signed_in(&self, token: String) -> Result<User, ApiError> {
        self.provider.sign_in(token);
        self.me().await
    }
}

impl AuthRepository for AuthRepositoryImpl {
    fn sign_up(&self, email: String, password: String) -> BoxFuture<'_, Result<String, ApiError>> {
        Box::pin(async move {
            self.provider
                .client()
                .auth()
                .register(&CredentialsDto { email, password })
                .await
                .map(|account| account.email)
                .map_err(app_error)
        })
    }

    fn sign_in(&self, email: String, password: String) -> BoxFuture<'_, Result<SignIn, ApiError>> {
        Box::pin(async move {
            // Signing in sends no token, so its 401 (wrong credentials) never expires the session.
            let answer = self
                .provider
                .client()
                .auth()
                .login(&CredentialsDto { email, password })
                .await
                .map_err(app_error)?;
            match answer {
                LoginResponse::TokenResponse(token) => self
                    .signed_in(token.access_token)
                    .await
                    .map(SignIn::SignedIn),
                LoginResponse::PendingToken(pending) => Ok(SignIn::CodeRequired(
                    PendingSignIn::new(pending.pending_token),
                )),
            }
        })
    }

    fn verify_code(
        &self,
        pending: PendingSignIn,
        code: String,
    ) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            let body = VerifyTotpDto {
                pending_token: pending.token().to_owned(),
                code,
            };
            let token = self
                .provider
                .client()
                .auth()
                .verify2fa(&body)
                .await
                .map_err(app_error)?;
            self.signed_in(token.access_token).await
        })
    }

    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async move {
            // The API's sessions are bearer tokens it does not keep: signing out forgets ours
            // (and the store's, see `Api::kept`).
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
