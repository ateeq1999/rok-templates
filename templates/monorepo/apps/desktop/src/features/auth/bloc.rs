//! Business logic: events, states and the bloc. No GPUI, no components.

// The business layer stays free of GPUI: see `clippy.toml`.
#![deny(clippy::disallowed_types)]

pub mod auth_bloc;
pub mod auth_event;
pub mod auth_state;
