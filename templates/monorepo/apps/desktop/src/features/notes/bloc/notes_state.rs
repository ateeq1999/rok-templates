//! What the `notes` views show.

use crate::data::models::note::Note;
use rok_ui::http::ApiError;

/// Where `NotesState` is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotesStatus {
    /// Initial.
    #[default]
    Initial,
    /// Loading.
    Loading,
    /// Success.
    Success,
    /// Failure.
    Failure,
}

/// What the views show: an immutable value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NotesState {
    /// Where loading is.
    pub status: NotesStatus,
    /// `notes`.
    pub notes: Vec<Note>,
    /// Why the last call failed, while `status` says so.
    pub error: Option<ApiError>,
}
