//! `AuthApi`: the `auth` API. Every path, verb and wire name lives in this file, so a backend change is a one-place fix.

use crate::data::models::user::UserId;
use rok_ui::http::ApiError;
use rok_ui::http::HttpClient;
use rok_ui::http::Options;

/// The `auth` endpoints, over the app's [`HttpClient`].
#[derive(Clone, Debug)]
pub struct AuthApi {
    client: HttpClient,
}

impl AuthApi {
    /// The endpoints, called through `client`.
    #[must_use]
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }

    /// The client (and through it, the session).
    #[must_use]
    pub fn client(&self) -> &HttpClient {
        &self.client
    }

    /// `POST /sessions`.
    ///
    /// A 401 here is an answer (wrong credentials), not an expired session.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn sign_in(&self, body: SignInDto) -> Result<SignedInDto, ApiError> {
        self.client
            .request("/sessions", Options::post().json(&body).skip_expire(true))
            .await
    }

    /// `DELETE /sessions/current`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn sign_out(&self) -> Result<(), ApiError> {
        self.client
            .request_empty("/sessions/current", Options::delete())
            .await
    }

    /// `GET /me`.
    ///
    /// # Errors
    ///
    /// The server's error, or why the server could not be reached.
    pub async fn me(&self) -> Result<UserDto, ApiError> {
        self.client.request("/me", Options::get()).await
    }
}

/// Sign in, as the API sends and receives it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignInDto {
    /// `email`.
    pub email: String,
    /// `password`.
    pub password: String,
}

/// Signed in, as the API sends and receives it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignedInDto {
    /// `token`.
    pub token: String,
    /// `user`.
    pub user: UserDto,
}

/// User, as the API sends and receives it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UserDto {
    /// `id`.
    pub id: UserId,
    /// `name`.
    pub name: String,
    /// `email`.
    pub email: String,
}
