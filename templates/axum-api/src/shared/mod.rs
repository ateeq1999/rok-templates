// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Tools with no business meaning, shared by the features. `shared/` never
//! imports `features/`.
//!
//! `rok sqlc g` regenerates `health`, `pagination`, `response`, `tenant`
//! and `validation`; don't edit them. `auth`, `body`, `security`, `mail`,
//! `metrics`, `rate_limit`, `settings`, `storage`, `telemetry` and
//! `thumbnail` are yours, like any tool you add here (a queue); the ones
//! with more than one implementation sit behind a trait.

pub mod auth;
pub mod body;
pub mod health;
pub mod mail;
pub mod metrics;
pub mod pagination;
pub mod rate_limit;
pub mod response;
pub mod security;
pub mod settings;
pub mod storage;
pub mod telemetry;
pub mod tenant;
pub mod thumbnail;
pub mod validation;
