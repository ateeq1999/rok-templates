//! `NotesRepository`: what the `notes` business logic reads and writes.

use crate::data::models::attachment::Attachment;
use crate::data::models::note::Note;
use crate::data::models::note::NoteId;
use crate::data::providers::notes_api::AttachmentDto;
use crate::data::providers::notes_api::NewNoteDto;
use crate::data::providers::notes_api::NoteDto;
use crate::data::providers::notes_api::NotesApi;
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;

/// What the `notes` business logic reads and writes. A trait, so tests and other data sources can stand in.
pub trait NotesRepository: Send + Sync {
    /// List.
    fn list(&self) -> BoxFuture<'_, Result<Vec<Note>, ApiError>>;
    /// Add.
    fn add(&self, title: String, body: String) -> BoxFuture<'_, Result<Note, ApiError>>;
    /// Delete.
    fn delete(&self, id: NoteId) -> BoxFuture<'_, Result<(), ApiError>>;
}

/// [`NotesRepository`] over a [`NotesApi`].
#[derive(Debug)]
pub struct NotesRepositoryImpl {
    provider: NotesApi,
}

impl NotesRepositoryImpl {
    /// A repository reading and writing through `provider`.
    #[must_use]
    pub fn new(provider: NotesApi) -> Self {
        Self { provider }
    }

    /// The data source, for the methods you add.
    #[must_use]
    pub fn provider(&self) -> &NotesApi {
        &self.provider
    }
}

impl NotesRepository for NotesRepositoryImpl {
    fn list(&self) -> BoxFuture<'_, Result<Vec<Note>, ApiError>> {
        Box::pin(async move {
            self.provider
                .list_notes(None)
                .await
                .map(|value| value.into_iter().map(Into::into).collect())
        })
    }

    fn add(&self, title: String, body: String) -> BoxFuture<'_, Result<Note, ApiError>> {
        Box::pin(async move {
            self.provider
                .create_note(NewNoteDto { title, body })
                .await
                .map(Into::into)
        })
    }

    fn delete(&self, id: NoteId) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async move { self.provider.delete_note(id).await })
    }
}

impl From<NoteDto> for Note {
    fn from(dto: NoteDto) -> Self {
        Self {
            id: dto.id,
            title: dto.title,
            body: dto.body,
            created_at: dto.created_at,
            attachments: dto.attachments.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<AttachmentDto> for Attachment {
    fn from(dto: AttachmentDto) -> Self {
        match dto {
            AttachmentDto::Image { url, width } => Self::Image { url, width },
            AttachmentDto::File { url, file_name } => Self::File { url, file_name },
        }
    }
}
