//! The `Note` model.

use crate::data::models::attachment::Attachment;

/// Note.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Note {
    /// `id`.
    pub id: NoteId,
    /// `title`.
    pub title: String,
    /// `body`.
    pub body: String,
    /// `created_at`.
    pub created_at: String,
    /// `attachments`.
    pub attachments: Vec<Attachment>,
}

/// A `Note`'s id.
pub type NoteId = u64;
