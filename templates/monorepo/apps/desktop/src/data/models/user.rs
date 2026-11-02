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
    /// `email_verified`: whether the user opened the link `apps/api` mailed them.
    pub email_verified: bool,
    /// `role`: what the user may see and do.
    pub role: Role,
}

impl User {
    /// Whether the user holds the admin role, which `apps/api`'s users routes need.
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }
}

/// A `User`'s id: the API's, a UUID.
pub type UserId = String;

/// A `User`'s role, as `apps/api` grants it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    /// A signed-up user: their own account.
    #[default]
    User,
    /// An admin: every user's account too.
    Admin,
}

impl Role {
    /// The role's name, as `apps/api` spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
        }
    }
}
