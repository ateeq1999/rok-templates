//! `UsersBloc`: the users list, a page at a time, with a fake repository.

use super::fake::{user, FakeUsersRepository};
use desktop::features::users::bloc::users_bloc::UsersBloc;
use desktop::features::users::bloc::users_event::UsersEvent;
use desktop::features::users::bloc::users_state::{UsersState, UsersStatus};
use rok_ui::bloc::test;
use rok_ui::http::ApiError;
use std::sync::Arc;

fn bloc(repository: FakeUsersRepository) -> UsersBloc {
    UsersBloc::new(Arc::new(repository))
}

#[test]
fn a_requested_page_is_the_page_the_repository_answers() {
    let states = test::run(
        bloc(FakeUsersRepository::with_users(45, 20)),
        [UsersEvent::PageRequested { page: 3 }],
    );

    let last = states.last().expect("a state");
    assert_eq!(last.status, UsersStatus::Success);
    assert_eq!(last.page.page, 3);
    assert_eq!(last.page.total, 45);
    assert_eq!(
        last.page.items,
        (41..=45).map(user).collect::<Vec<_>>(),
        "the third page of 20 holds users 41 to 45"
    );
}

#[test]
fn the_page_count_is_the_total_over_the_page_size() {
    let states = test::run(
        bloc(FakeUsersRepository::with_users(41, 20)),
        [UsersEvent::PageRequested { page: 1 }],
    );

    assert_eq!(states.last().map(UsersState::page_count), Some(3));
    assert_eq!(UsersState::default().page_count(), 0, "nothing loaded yet");
}

#[test]
fn the_rows_stay_while_the_next_page_loads() {
    let states = test::run_settled(
        bloc(FakeUsersRepository::with_users(30, 20)),
        [
            UsersEvent::PageRequested { page: 1 },
            UsersEvent::PageRequested { page: 2 },
        ],
    );

    let statuses: Vec<UsersStatus> = states.iter().map(|state| state.status).collect();
    assert_eq!(
        statuses,
        [
            UsersStatus::Loading,
            UsersStatus::Success,
            UsersStatus::Loading,
            UsersStatus::Success,
        ]
    );
    assert_eq!(
        states[2].page.items.len(),
        20,
        "page 1 shows while page 2 loads"
    );
    assert_eq!(
        states[3].page.items,
        (21..=30).map(user).collect::<Vec<_>>()
    );
}

#[test]
fn a_failed_page_says_why() {
    let states = test::run(
        bloc(FakeUsersRepository::failing(ApiError::new(
            403,
            "forbidden",
            "Admins only",
        ))),
        [UsersEvent::PageRequested { page: 1 }],
    );

    let last = states.last().expect("a state");
    assert_eq!(last.status, UsersStatus::Failure);
    assert_eq!(
        last.error.as_ref().map(ToString::to_string).as_deref(),
        Some("Admins only")
    );
}
