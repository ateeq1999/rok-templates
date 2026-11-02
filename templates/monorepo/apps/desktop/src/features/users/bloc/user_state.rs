//! What a user's detail screen shows.

use crate::data::models::user::{User, UserId};
use rok_ui::http::ApiError;

/// Where `UserState` is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UserStatus {
    /// Nothing asked for yet.
    #[default]
    Initial,
    /// The user `id` is on its way.
    Loading,
    /// `user` is the user `id`.
    Success,
    /// Reading the user `id` failed: `error` says why.
    Failure,
}

/// What the detail screen shows: an immutable value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UserState {
    /// Where loading is.
    pub status: UserStatus,
    /// The user asked for last.
    pub id: Option<UserId>,
    /// The user `id`, once it has loaded.
    pub user: Option<User>,
    /// Why reading the user failed, while `status` says so.
    pub error: Option<ApiError>,
}
