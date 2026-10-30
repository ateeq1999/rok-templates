//! `AuthBloc`: events in, states out, with a fake repository.

use desktop::data::models::user::User;
use desktop::data::repositories::auth_repository::AuthRepository;
use desktop::features::auth::bloc::auth_bloc::AuthBloc;
use desktop::features::auth::bloc::auth_event::AuthEvent;
use desktop::features::auth::bloc::auth_state::AuthStatus;
use rok_ui::bloc::test;
use rok_ui::bloc::BoxFuture;
use rok_ui::http::ApiError;
use std::sync::Arc;

/// Answers every call with an empty success.
struct FakeAuthRepository;

impl AuthRepository for FakeAuthRepository {
    fn sign_in(&self, _email: String, _password: String) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async { Ok(User::default()) })
    }

    fn sign_out(&self) -> BoxFuture<'_, Result<(), ApiError>> {
        Box::pin(async { Ok(()) })
    }

    fn me(&self) -> BoxFuture<'_, Result<User, ApiError>> {
        Box::pin(async { Ok(User::default()) })
    }
}

#[test]
fn sign_in_submitted() {
    let states = test::run(
        AuthBloc::new(Arc::new(FakeAuthRepository)),
        [AuthEvent::SignInSubmitted {
            email: String::new(),
            password: String::new(),
        }],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(AuthStatus::Success)
    );
}

#[test]
fn sign_out_requested() {
    let states = test::run(
        AuthBloc::new(Arc::new(FakeAuthRepository)),
        [AuthEvent::SignOutRequested],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(AuthStatus::Success)
    );
}

#[test]
fn profile_requested() {
    let states = test::run(
        AuthBloc::new(Arc::new(FakeAuthRepository)),
        [AuthEvent::ProfileRequested],
    );
    assert_eq!(
        states.last().map(|state| state.status),
        Some(AuthStatus::Success)
    );
}
