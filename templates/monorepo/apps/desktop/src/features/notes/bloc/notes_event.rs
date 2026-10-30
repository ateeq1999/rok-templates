//! What happens in `notes`, in the past tense.

use crate::data::models::note::NoteId;

/// Events for `NotesBloc`.
#[derive(Clone, Debug)]
pub enum NotesEvent {
    /// Notes requested.
    NotesRequested,
    /// Note added.
    NoteAdded {
        /// `title`.
        title: String,
        /// `body`.
        body: String,
    },
    /// Note deleted.
    NoteDeleted {
        /// `id`.
        id: NoteId,
    },
}
