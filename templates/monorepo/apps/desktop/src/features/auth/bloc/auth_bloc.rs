//! `AuthBloc`: the business logic of `auth`.

use super::auth_event::AuthEvent;
use super::auth_state::AuthState;
use super::auth_state::AuthStatus;
use crate::data::repositories::auth_repository::AuthRepository;
use rok_ui::bloc::Bloc;
use rok_ui::bloc::Concurrency;
use rok_ui::bloc::Emitter;
use std::sync::Arc;

/// Turns [`AuthEvent`]s into [`AuthState`]s, reading and writing through a [`AuthRepository`].
pub struct AuthBloc {
    repository: Arc<dyn AuthRepository>,
}

impl AuthBloc {
    /// A bloc reading and writing through `repository`.
    #[must_use]
    pub fn new(repository: Arc<dyn AuthRepository>) -> Self {
        Self { repository }
    }
}

impl Bloc for AuthBloc {
    type Event = AuthEvent;
    type State = AuthState;

    fn initial_state(&self) -> AuthState {
        AuthState::default()
    }

    fn concurrency(&self, event: &AuthEvent) -> Concurrency {
        match event {
            AuthEvent::SignInSubmitted { .. } => Concurrency::Droppable,
            AuthEvent::SignOutRequested => Concurrency::Sequential,
            AuthEvent::ProfileRequested => Concurrency::Sequential,
        }
    }

    async fn on(&self, event: AuthEvent, emit: &Emitter<AuthState>) {
        match event {
            AuthEvent::SignInSubmitted { email, password } => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.sign_in(email, password).await {
                    Ok(value) => {
                        emit.update(|state| {
                            state.status = AuthStatus::Success;
                            state.user = Some(value);
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = AuthStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
            AuthEvent::SignOutRequested => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.sign_out().await {
                    Ok(()) => {
                        emit.emit(AuthState {
                            status: AuthStatus::Success,
                            ..AuthState::default()
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = AuthStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
            AuthEvent::ProfileRequested => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.me().await {
                    Ok(value) => {
                        emit.update(|state| {
                            state.status = AuthStatus::Success;
                            state.user = Some(value);
                            state.error = None;
                        });
                    }
                    Err(error) if error.is_cancelled() => {}
                    Err(error) => {
                        emit.update(|state| {
                            state.status = AuthStatus::Failure;
                            state.error = Some(error);
                        });
                    }
                }
            }
        }
    }
}
