//! Business logic: events, states and the bloc. No GPUI, no components.

// The business layer stays free of GPUI: see `clippy.toml`.
#![deny(clippy::disallowed_types)]

pub mod notes_bloc;
pub mod notes_event;
pub mod notes_state;
