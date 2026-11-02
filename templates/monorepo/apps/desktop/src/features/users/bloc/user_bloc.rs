//! `UserBloc`: one user, for the detail screen.

use super::user_event::UserEvent;
use super::user_state::{UserState, UserStatus};
use crate::data::repositories::users_repository::UsersRepository;
use rok_ui::bloc::{Bloc, Concurrency, Emitter};
use std::sync::Arc;

/// Turns [`UserEvent`]s into [`UserState`]s, reading through a [`UsersRepository`].
pub struct UserBloc {
    repository: Arc<dyn UsersRepository>,
}

impl UserBloc {
    /// A bloc reading through `repository`.
    #[must_use]
    pub fn new(repository: Arc<dyn UsersRepository>) -> Self {
        Self { repository }
    }
}

impl Bloc for UserBloc {
    type Event = UserEvent;
    type State = UserState;

    fn initial_state(&self) -> UserState {
        UserState::default()
    }

    fn concurrency(&self, event: &UserEvent) -> Concurrency {
        match event {
            // The user opened last wins.
            UserEvent::Opened { .. } => Concurrency::Restartable,
        }
    }

    async fn on(&self, event: UserEvent, emit: &Emitter<UserState>) {
        match event {
            UserEvent::Opened { id } => {
                // Another user's detail never shows under this id while it loads.
                emit.emit(UserState {
                    status: UserStatus::Loading,
                    id: Some(id.clone()),
                    ..UserState::default()
                });
                match self.repository.get(id).await {
                    Ok(user) => {
                        emit.update(|state| {
                            state.status = UserStatus::Success;
                            state.user = Some(user);
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = UserStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
        }
    }
}
