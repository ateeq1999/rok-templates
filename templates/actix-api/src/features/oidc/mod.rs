// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `oidc` feature, under `/api/v1/auth/oidc`: signing in with an
//! `OpenID` provider, and the signed-in account's Linked identities.
//! Inert until a provider is configured (`OIDC_PROVIDERS`): until then
//! every route answers `404`. Hand-written in the generated features'
//! layout; `check-arch` applies to it in full. Its SQL is
//! `queries/oidc.sql` (`db::queries::oidc`); the providers are reached
//! through `shared::oidc`.

/// Request and response bodies.
pub mod dto;
/// The handlers: parse, call one service method, answer.
pub mod handlers;
/// The URL to handler mapping.
pub mod routes;
/// `OidcService`: the flows' rules.
pub mod service;

pub use routes::routes;
pub use service::OidcService;
