//! What the users list shows.

use crate::data::models::page::Page;
use crate::data::models::user::User;
use rok_ui::http::ApiError;

/// Where `UsersState` is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UsersStatus {
    /// Nothing asked for yet.
    #[default]
    Initial,
    /// A page is on its way; the last one's rows still show.
    Loading,
    /// `page` is the page the API answered.
    Success,
    /// The last page asked for failed: `error` says why.
    Failure,
}

/// What the list shows: an immutable value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UsersState {
    /// Where loading is.
    pub status: UsersStatus,
    /// The page shown, as the API paged it.
    pub page: Page<User>,
    /// Why the last page failed, while `status` says so.
    pub error: Option<ApiError>,
}

impl UsersState {
    /// How many pages the list has: the total over the page size, rounded up; 0 before a
    /// page has loaded or when there are no users.
    #[must_use]
    pub fn page_count(&self) -> u32 {
        if self.page.per_page == 0 {
            return 0;
        }
        let pages = self.page.total.div_ceil(u64::from(self.page.per_page));
        u32::try_from(pages).unwrap_or(u32::MAX)
    }
}
