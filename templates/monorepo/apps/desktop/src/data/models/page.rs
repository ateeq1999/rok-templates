//! The `Page` model.

/// One page of a list, as the API pages it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page<T> {
    /// The page's rows.
    pub items: Vec<T>,
    /// The page, from 1.
    pub page: u32,
    /// The most rows a page holds.
    pub per_page: u32,
    /// The rows on every page together.
    pub total: u64,
}
