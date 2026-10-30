//! `NotesBloc`: events in, states out, with a fake repository.

use desktop::data::models::note::Note;
use desktop::data::models::note::NoteId;
use desktop::data::repositories::notes_repository::NotesRepository;
use desktop::features::notes::bloc::notes_bloc::NotesBloc;
use desktop::features::notes::bloc::notes_event::NotesEvent;
use desktop::features::notes::bloc::notes_state::NotesStatus;
use rok_ui::bloc::test;
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;
use std::sync::Arc;

/// Answers every call with an empty success.
struct FakeNotesRepository;

impl NotesRepository for FakeNotesRepository {
    fn list(&self) -> BoxFuture<'_, Result<Vec<Note>, ApiError>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn add(&self, _title: String, _body: String) -> BoxFuture<'_, Result<Note, ApiError>> {
        Box::pin(async { Ok(Note::default()) })
    }

    fn delete(&self, _id: NoteId) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async { Ok(()) })
    }
}

#[test]
fn notes_requested() {
    let states = test::run(
        NotesBloc::new(Arc::new(FakeNotesRepository)),
        [NotesEvent::NotesRequested],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(NotesStatus::Success)
    );
}

#[test]
fn note_added() {
    let states = test::run(
        NotesBloc::new(Arc::new(FakeNotesRepository)),
        [NotesEvent::NoteAdded {
            title: String::new(),
            body: String::new(),
        }],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(NotesStatus::Success)
    );
}

#[test]
fn note_deleted() {
    let states = test::run(
        NotesBloc::new(Arc::new(FakeNotesRepository)),
        [NotesEvent::NoteDeleted {
            id: NoteId::default(),
        }],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(NotesStatus::Success)
    );
}
