//! `UsersBloc`: the users list, a page at a time.

use super::users_event::UsersEvent;
use super::users_state::{UsersState, UsersStatus};
use crate::data::repositories::users_repository::UsersRepository;
use rok_ui::bloc::{Bloc, Concurrency, Emitter};
use std::sync::Arc;

/// Turns [`UsersEvent`]s into [`UsersState`]s, reading through a [`UsersRepository`].
pub struct UsersBloc {
    repository: Arc<dyn UsersRepository>,
}

impl UsersBloc {
    /// A bloc reading through `repository`.
    #[must_use]
    pub fn new(repository: Arc<dyn UsersRepository>) -> Self {
        Self { repository }
    }
}

impl Bloc for UsersBloc {
    type Event = UsersEvent;
    type State = UsersState;

    fn initial_state(&self) -> UsersState {
        UsersState::default()
    }

    fn concurrency(&self, event: &UsersEvent) -> Concurrency {
        match event {
            // The page asked for last wins: an older page never lands over it.
            UsersEvent::PageRequested { .. } => Concurrency::Restartable,
        }
    }

    async fn on(&self, event: UsersEvent, emit: &Emitter<UsersState>) {
        match event {
            UsersEvent::PageRequested { page } => {
                emit.update(|state| state.status = UsersStatus::Loading);
                match self.repository.list(page).await {
                    Ok(page) => {
                        emit.update(|state| {
                            state.status = UsersStatus::Success;
                            state.page = page;
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = UsersStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
        }
    }
}
