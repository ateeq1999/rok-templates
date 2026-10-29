// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The application, as a library: `main.rs` starts it, and tests build it
//! with `app::app`. Every module is public, so generated code the
//! application does not use yet is not dead code.
//!
//! The lints below are the bar the generated code meets (engine ADR 0013): every
//! public item is documented, and no `unwrap`, `expect` or `panic` outside
//! the tests.

#![deny(missing_docs, warnings)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// The selected pedantic lints. They warn, so `deny(warnings)` and
// `cargo clippy -- -D warnings` turn any hit into an error.
#![warn(
    clippy::default_trait_access,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::items_after_statements,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::ref_option,
    clippy::return_self_not_must_use,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
    clippy::trivially_copy_pass_by_ref,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps
)]

pub mod app;
pub mod config;
pub mod db;
pub mod error;
pub mod features;
pub mod openapi;
pub mod shared;
