// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Tools with no business meaning, shared by the features. `shared/` never
//! imports `features/`.
//!
//! `rok sqlc g` regenerates `grpc`, `pagination`, `response` and
//! `validation`; don't edit them. `auth`, `jobs`, `security`, `mail`,
//! `metrics`, `settings`, `shutdown`, `storage` and `telemetry` are yours,
//! like any tool you add here (a queue); the ones with more than one
//! implementation sit behind a trait. All but `auth` are the axum
//! template's, file for file.

pub mod auth;
pub mod grpc;
pub mod jobs;
pub mod mail;
pub mod metrics;
pub mod pagination;
pub mod response;
pub mod security;
pub mod settings;
pub mod shutdown;
pub mod storage;
pub mod telemetry;
pub mod validation;
