//! `AuthBloc`: the business logic of `auth`.

use super::auth_event::AuthEvent;
use super::auth_state::AuthState;
use super::auth_state::AuthStatus;
use crate::data::models::sign_in::SignIn;
use crate::data::models::user::User;
use crate::data::repositories::auth_repository::AuthRepository;
use rok_ui::bloc::Bloc;
use rok_ui::bloc::Concurrency;
use rok_ui::bloc::Emitter;
use rok_ui::http::ApiError;
use std::sync::Arc;

/// The code `apps/api` answers a pending sign-in with once it expired or was spent.
const SPENT_SIGN_IN: &str = "invalid_token";

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

/// Shows `user` as signed in: no pending sign-in, no error.
fn signed_in(emit: &Emitter<AuthState>, user: User) {
    emit.emit(AuthState {
        status: AuthStatus::Success,
        user: Some(user),
        ..AuthState::default()
    });
}

/// Shows `error`, unless the call was cancelled.
fn failed(emit: &Emitter<AuthState>, error: ApiError) {
    if error.is_cancelled() {
        return;
    }
    emit.update(|state| {
        state.status = AuthStatus::Failure;
        state.error = Some(error);
    });
}

impl Bloc for AuthBloc {
    type Event = AuthEvent;
    type State = AuthState;

    fn initial_state(&self) -> AuthState {
        AuthState::default()
    }

    fn concurrency(&self, event: &AuthEvent) -> Concurrency {
        match event {
            AuthEvent::SignUpSubmitted { .. }
            | AuthEvent::SignInSubmitted { .. }
            | AuthEvent::CodeSubmitted { .. } => Concurrency::Droppable,
            AuthEvent::CodeCancelled
            | AuthEvent::SignOutRequested
            | AuthEvent::ProfileRequested => Concurrency::Sequential,
        }
    }

    async fn on(&self, event: AuthEvent, emit: &Emitter<AuthState>) {
        match event {
            AuthEvent::SignUpSubmitted { email, password } => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.sign_up(email, password).await {
                    Ok(email) => {
                        emit.emit(AuthState {
                            status: AuthStatus::Success,
                            signed_up: Some(email),
                            ..AuthState::default()
                        });
                    }
                    Err(error) => failed(emit, error),
                }
            }
            AuthEvent::SignInSubmitted { email, password } => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.sign_in(email, password).await {
                    Ok(SignIn::SignedIn(user)) => signed_in(emit, user),
                    Ok(SignIn::CodeRequired(pending)) => {
                        emit.emit(AuthState {
                            pending: Some(pending),
                            ..AuthState::default()
                        });
                    }
                    Err(error) => failed(emit, error),
                }
            }
            AuthEvent::CodeSubmitted { code } => {
                let Some(pending) = emit.state().pending else {
                    return;
                };
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.verify_code(pending, code).await {
                    Ok(user) => signed_in(emit, user),
                    Err(error) => {
                        // A wrong code may be tried again; an expired or spent sign-in starts over.
                        if error.code == SPENT_SIGN_IN {
                            emit.update(|state| state.pending = None);
                        }
                        failed(emit, error);
                    }
                }
            }
            AuthEvent::CodeCancelled => {
                emit.emit(AuthState::default());
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
                    Err(error) => failed(emit, error),
                }
            }
            AuthEvent::ProfileRequested => {
                emit.update(|state| state.status = AuthStatus::Loading);
                match self.repository.me().await {
                    Ok(user) => signed_in(emit, user),
                    Err(error) => failed(emit, error),
                }
            }
        }
    }
}
