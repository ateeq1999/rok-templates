//! `UsersRepository`: what the `users` business logic reads.

use crate::data::models::page::Page;
use crate::data::models::user::{User, UserId};
use crate::data::providers::api::{app_error, Api};
use api_client::{PaginatedResponseUserResponse, UsersListQuery};
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;

/// What the `users` business logic reads. A trait, so tests and other data sources can stand in.
pub trait UsersRepository: Send + Sync {
    /// The page `page` (from 1) of the users, as the API pages them.
    fn list(&self, page: u32) -> BoxFuture<'_, Result<Page<User>, ApiError>>;
    /// The user `id`.
    fn get(&self, id: UserId) -> BoxFuture<'_, Result<User, ApiError>>;
}

/// [`UsersRepository`] over the [`Api`].
#[derive(Debug)]
pub struct UsersRepositoryImpl {
    provider: Api,
}

impl UsersRepositoryImpl {
    /// A repository reading through `provider`.
    #[must_use]
    pub fn new(provider: Api) -> Self {
        Self { provider }
    }

    /// The data source, for the methods you add.
    #[must_use]
    pub fn provider(&self) -> &Api {
        &self.provider
    }
}

impl UsersRepository for UsersRepositoryImpl {
    fn list(&self, page: u32) -> BoxFuture<'_, Result<Page<User>, ApiError>> {
        Box::pin(async move {
            let query = UsersListQuery {
                page: Some(i64::from(page)),
                per_page: None,
                q: None,
                sort: None,
                order: None,
            };
            self.provider
                .client()
                .users()
                .list(&query)
                .await
                .map(Into::into)
                .map_err(app_error)
        })
    }

    fn get(&self, id: UserId) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async move {
            self.provider
                .client()
                .users()
                .get_one(&id)
                .await
                .map(Into::into)
                .map_err(app_error)
        })
    }
}

impl From<PaginatedResponseUserResponse> for Page<User> {
    fn from(page: PaginatedResponseUserResponse) -> Self {
        Self {
            items: page.items.into_iter().map(Into::into).collect(),
            // The API answers a page from 1, at most 100 rows, never a negative total.
            page: u32::try_from(page.page).unwrap_or(1),
            per_page: u32::try_from(page.per_page).unwrap_or(0),
            total: u64::try_from(page.total).unwrap_or(0),
        }
    }
}
