//! The `User` model.

/// User.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct User {
    /// `id`.
    pub id: UserId,
    /// `name`: the display name, empty when the user has none.
    pub name: String,
    /// `email`.
    pub email: String,
}

/// A `User`'s id: the API's, a UUID.
pub type UserId = String;
