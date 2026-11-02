//! `UserBloc`: one user's detail, with a fake repository.

use super::fake::{user, FakeUsersRepository};
use desktop::features::users::bloc::user_bloc::UserBloc;
use desktop::features::users::bloc::user_event::UserEvent;
use desktop::features::users::bloc::user_state::UserStatus;
use std::sync::Arc;

use rok_ui::bloc::test;

fn bloc(repository: FakeUsersRepository) -> UserBloc {
    UserBloc::new(Arc::new(repository))
}

#[test]
fn a_requested_user_is_the_user_the_repository_answers() {
    let states = test::run(
        bloc(FakeUsersRepository::with_users(3, 20)),
        [UserEvent::Opened {
            id: "user-2".into(),
        }],
    );

    let last = states.last().expect("a state");
    assert_eq!(last.status, UserStatus::Success);
    assert_eq!(last.id.as_deref(), Some("user-2"));
    assert_eq!(last.user, Some(user(2)));
}

#[test]
fn an_unknown_user_is_not_found() {
    let states = test::run(
        bloc(FakeUsersRepository::with_users(3, 20)),
        [UserEvent::Opened {
            id: "nobody".into(),
        }],
    );

    let last = states.last().expect("a state");
    assert_eq!(last.status, UserStatus::Failure);
    assert_eq!(last.user, None);
    assert_eq!(last.error.as_ref().map(|error| error.status), Some(404));
}

#[test]
fn the_previous_user_goes_while_the_next_loads() {
    let states = test::run_settled(
        bloc(FakeUsersRepository::with_users(3, 20)),
        [
            UserEvent::Opened {
                id: "user-1".into(),
            },
            UserEvent::Opened {
                id: "user-3".into(),
            },
        ],
    );

    let loading_the_second = &states[2];
    assert_eq!(loading_the_second.status, UserStatus::Loading);
    assert_eq!(loading_the_second.id.as_deref(), Some("user-3"));
    assert_eq!(
        loading_the_second.user, None,
        "never user 1 under user 3's id"
    );
    assert_eq!(
        states.last().and_then(|state| state.user.clone()),
        Some(user(3))
    );
}
