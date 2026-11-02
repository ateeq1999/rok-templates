//! What happens on a user's detail screen, in the past tense.

use crate::data::models::user::UserId;

/// Events for `UserBloc`.
#[derive(Clone, Debug)]
pub enum UserEvent {
    /// The detail screen opened on the user `id`.
    Requested {
        /// The user's id.
        id: UserId,
    },
}
