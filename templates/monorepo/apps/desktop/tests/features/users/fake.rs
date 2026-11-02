//! A `UsersRepository` over a list in memory, paged as the API pages it.

use desktop::data::models::page::Page;
use desktop::data::models::user::{Role, User, UserId};
use desktop::data::repositories::users_repository::UsersRepository;
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;

/// Users in memory, `per_page` to a page; or every call failing with `failure`.
pub struct FakeUsersRepository {
    users: Vec<User>,
    per_page: u32,
    failure: Option<ApiError>,
}

impl FakeUsersRepository {
    /// `count` users, `per_page` to a page.
    pub fn with_users(count: u32, per_page: u32) -> Self {
        Self {
            users: (1..=count).map(user).collect(),
            per_page,
            failure: None,
        }
    }

    /// Every call answers `failure`.
    pub fn failing(failure: ApiError) -> Self {
        Self {
            users: Vec::new(),
            per_page: 20,
            failure: Some(failure),
        }
    }
}

/// The user numbered `n`: id `user-<n>`, `user<n>@example.com`.
pub fn user(n: u32) -> User {
    User {
        id: format!("user-{n}"),
        name: format!("User {n}"),
        email: format!("user{n}@example.com"),
        email_verified: true,
        role: Role::User,
    }
}

impl UsersRepository for FakeUsersRepository {
    fn list(&self, page: u32) -> BoxFuture<'_, Result<Page<User>, ApiError>> {
        Box::pin(async move {
            if let Some(failure) = &self.failure {
                return Err(failure.clone());
            }
            let per_page = self.per_page as usize;
            let start = (page.saturating_sub(1) as usize) * per_page;
            Ok(Page {
                items: self
                    .users
                    .iter()
                    .skip(start)
                    .take(per_page)
                    .cloned()
                    .collect(),
                page,
                per_page: self.per_page,
                total: self.users.len() as u64,
            })
        })
    }

    fn get(&self, id: UserId) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            if let Some(failure) = &self.failure {
                return Err(failure.clone());
            }
            self.users
                .iter()
                .find(|user| user.id == id)
                .cloned()
                .ok_or_else(|| ApiError::new(404, "not_found", "User not found"))
        })
    }
}
