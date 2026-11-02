//! Repositories: traits the business layer depends on, with their implementations.

use crate::data::models::user::{Role, User};

pub mod auth_repository;
pub mod users_repository;

/// `From<$answer> for User`, for each of `apps/api`'s answers about an account: they carry
/// the same fields, so they read the same way.
macro_rules! user_from {
    ($($answer:ty),+ $(,)?) => {$(
        impl From<$answer> for User {
            fn from(answer: $answer) -> Self {
                Self {
                    id: answer.id,
                    name: answer.display_name.unwrap_or_default(),
                    email: answer.email,
                    email_verified: answer.email_verified,
                    role: answer.role.into(),
                }
            }
        }
    )+};
}

// The signed-in account (`GET /auth/me`) and a row of the users list (`GET /users`).
user_from!(api_client::AccountResponse, api_client::UserResponse);

impl From<api_client::Role> for Role {
    fn from(role: api_client::Role) -> Self {
        match role {
            api_client::Role::User => Self::User,
            api_client::Role::Admin => Self::Admin,
        }
    }
}
