//! `AuthRepository`: what the `auth` business logic reads and writes.

use crate::data::models::user::User;
use crate::data::providers::auth_api::AuthApi;
use crate::data::providers::auth_api::SignInDto;
use crate::data::providers::auth_api::UserDto;
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

/// [`AuthRepository`] over a [`AuthApi`].
#[derive(Debug)]
pub struct AuthRepositoryImpl {
    provider: AuthApi,
}

impl AuthRepositoryImpl {
    /// A repository reading and writing through `provider`.
    #[must_use]
    pub fn new(provider: AuthApi) -> Self {
        Self { provider }
    }

    /// The data source, for the methods you add.
    #[must_use]
    pub fn provider(&self) -> &AuthApi {
        &self.provider
    }
}

impl AuthRepository for AuthRepositoryImpl {
    fn sign_in(&self, email: String, password: String) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            let value = self.provider.sign_in(SignInDto { email, password }).await?;
            if let Some(session) = self.provider.client().session() {
                session.set_token(value.token.clone());
            }
            Ok(value.user.into())
        })
    }

    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async move {
            let result = self.provider.sign_out().await;
            // Signed out locally even when the server call fails.
            if let Some(session) = self.provider.client().session() {
                session.clear();
            }
            result
        })
    }

    fn me(&self) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move { self.provider.me().await.map(Into::into) })
    }
}

impl From<UserDto> for User {
    fn from(dto: UserDto) -> Self {
        Self {
            id: dto.id,
            name: dto.name,
            email: dto.email,
        }
    }
}
