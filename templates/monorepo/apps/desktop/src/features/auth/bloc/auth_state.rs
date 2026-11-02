//! What the `auth` views show.

use crate::data::models::sign_in::PendingSignIn;
use crate::data::models::user::User;
use rok_ui::http::ApiError;

/// Where `AuthState` is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuthStatus {
    /// Initial.
    #[default]
    Initial,
    /// Loading.
    Loading,
    /// Success.
    Success,
    /// Failure.
    Failure,
}

/// What the views show: an immutable value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthState {
    /// Where loading is.
    pub status: AuthStatus,
    /// The signed-in user.
    pub user: Option<User>,
    /// A sign-in waiting for its two-factor code: the views ask for it.
    pub pending: Option<PendingSignIn>,
    /// The email an account was just created with, which `apps/api` mailed a link to verify.
    pub signed_up: Option<String>,
    /// Why the last call failed, while `status` says so. Its message is `apps/api`'s words.
    pub error: Option<ApiError>,
}

impl AuthState {
    /// Whether a sign-in waits for its two-factor code.
    #[must_use]
    pub fn code_required(&self) -> bool {
        self.pending.is_some()
    }
}
