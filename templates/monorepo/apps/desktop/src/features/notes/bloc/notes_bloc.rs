//! `NotesBloc`: the business logic of `notes`.

use super::notes_event::NotesEvent;
use super::notes_state::NotesState;
use super::notes_state::NotesStatus;
use crate::data::repositories::notes_repository::NotesRepository;
use rok_ui::bloc::Bloc;
use rok_ui::bloc::Emitter;
use std::sync::Arc;

/// Turns [`NotesEvent`]s into [`NotesState`]s, reading and writing through a [`NotesRepository`].
pub struct NotesBloc {
    repository: Arc<dyn NotesRepository>,
}

impl NotesBloc {
    /// A bloc reading and writing through `repository`.
    #[must_use]
    pub fn new(repository: Arc<dyn NotesRepository>) -> Self {
        Self { repository }
    }
}

impl Bloc for NotesBloc {
    type Event = NotesEvent;
    type State = NotesState;

    fn initial_state(&self) -> NotesState {
        NotesState::default()
    }

    async fn on(&self, event: NotesEvent, emit: &Emitter<NotesState>) {
        match event {
            NotesEvent::NotesRequested => {
                emit.update(|state| state.status = NotesStatus::Loading);
                match self.repository.list().await {
                    Ok(value) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Success;
                            state.notes = value;
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
            NotesEvent::NoteAdded { title, body } => {
                emit.update(|state| state.status = NotesStatus::Loading);
                let result = match self.repository.add(title, body).await {
                    Ok(_) => self.repository.list().await,
                    Err(error) => Err(error),
                };
                match result {
                    Ok(value) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Success;
                            state.notes = value;
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
            NotesEvent::NoteDeleted { id } => {
                emit.update(|state| state.status = NotesStatus::Loading);
                let result = match self.repository.delete(id).await {
                    Ok(()) => self.repository.list().await,
                    Err(error) => Err(error),
                };
                match result {
                    Ok(value) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Success;
                            state.notes = value;
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = NotesStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
        }
    }
}
