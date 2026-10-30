//! The app, in three layers (see the architecture guide):
//!
//! - `data`: models, providers (where data comes from) and repositories.
//! - `features`: one module per feature, with its business logic (`bloc`) and views (`view`).
//! - `routes`: thin route files that render feature pages.
//!
//! `cargo rok-ui g feature <name>` adds a feature and wires it into `app.rs`.

pub mod app;
pub mod data;
pub mod features;
pub mod shared;

rok_ui::routes!();
