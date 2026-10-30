//! The `User` model.

/// User.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct User {
    /// `id`.
    pub id: UserId,
    /// `name`.
    pub name: String,
    /// `email`.
    pub email: String,
}

/// A `User`'s id.
pub type UserId = u64;
