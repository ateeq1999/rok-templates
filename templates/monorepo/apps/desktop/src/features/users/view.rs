//! Presentation: the pages that read the blocs and add their events, and the gate around them.

pub mod admin_only;
pub mod user_page;
pub mod users_page;

/// Where a user's detail screen is: `/users/<id>` (the route `routes/_app/users/$id.rs`).
#[must_use]
pub fn user_path(id: &str) -> String {
    format!("/users/{id}")
}
