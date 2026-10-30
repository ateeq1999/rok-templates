//! What the `auth` views show.

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
    /// `user`.
    pub user: Option<User>,
    /// Why the last call failed, while `status` says so.
    pub error: Option<ApiError>,
}
