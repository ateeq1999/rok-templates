// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` request and response bodies, and the callback's query.

use crate::db::user_identities::models::UserIdentityId;

/// One of the signed-in account's Linked identities.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct IdentityResponse {
    /// Unlink it with `DELETE /auth/oidc/identities/{id}`.
    pub id: UserIdentityId,
    /// The provider's name, such as `google`.
    pub provider: String,
    /// The email the provider gave when it was linked, if any.
    pub email: Option<String>,
    /// When it was linked.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `POST /auth/oidc/{provider}/link`'s answer: where to send the browser.
/// The provider sends it back to the callback, which links the identity
/// and goes on to `{APP_URL}/auth/oidc#linked=<provider>`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct LinkResponse {
    /// The provider's authorization URL.
    pub url: String,
}

/// What the provider's redirect brings to the callback: a `code` and the
/// `state` it was given, or an `error`.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CallbackQuery {
    /// The authorization code to redeem.
    pub code: Option<String>,
    /// The state the authorization carried.
    pub state: Option<String>,
    /// Why the provider didn't authorize, such as `access_denied`.
    pub error: Option<String>,
}

/// Which App a sign-in or a link comes back to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OidcClient {
    /// The web App: the callback sends the browser to `{APP_URL}/auth/oidc#…`.
    #[default]
    Web,
    /// The mobile App: the callback hands over at `<APP_SCHEME>://auth/oidc?…`,
    /// a session as a one-minute code for `POST /auth/oidc/handoff`.
    Mobile,
}

/// `GET /auth/oidc/{provider}/start`'s and `POST /auth/oidc/{provider}/link`'s
/// query string.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ClientQuery {
    /// The App to come back to: `web` (the default) or `mobile`.
    pub client: Option<OidcClient>,
}

/// One provider people may sign in with, for an App's "Continue with …"
/// button.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct SignInProviderResponse {
    /// Its name in the routes (`/auth/oidc/{name}/start`), such as `google`.
    pub name: String,
}

/// `POST /auth/oidc/handoff`'s body: the code the mobile App's deep link
/// brought.
#[derive(Debug, Clone, serde::Deserialize, rok_data::Input, utoipa::ToSchema)]
pub struct HandoffDto {
    /// The `code` of `<APP_SCHEME>://auth/oidc?code=…`.
    #[rok(validate(length(min = 1, message = "code must not be empty")))]
    pub code: String,
}

/// A session: a bearer token and how long it lasts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct TokenResponse {
    /// The JWT to send as `Authorization: Bearer <access_token>`.
    pub access_token: String,
    /// Seconds until it expires.
    pub expires_in: i64,
    /// Always `Bearer`.
    pub token_type: String,
}
