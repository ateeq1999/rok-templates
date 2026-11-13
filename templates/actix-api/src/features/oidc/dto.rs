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
