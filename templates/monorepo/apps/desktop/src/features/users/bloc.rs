//! Business logic: events, states and the blocs. No GPUI, no components.
//!
//! `UsersBloc` pages the list; `UserBloc` reads one user for the detail screen.

// The business layer stays free of GPUI: see `clippy.toml`.
#![deny(clippy::disallowed_types)]

pub mod user_bloc;
pub mod user_event;
pub mod user_state;
pub mod users_bloc;
pub mod users_event;
pub mod users_state;
