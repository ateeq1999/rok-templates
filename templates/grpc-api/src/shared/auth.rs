// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Who is calling: the [`AuthUser`] the interceptor (`app/auth.rs`) puts in
//! a request's extensions once it checked the credentials, and the
//! [`AdminUser`] the admin-only services take. The generated services read
//! them through `shared/grpc.rs` (`caller`, `admin`): no caller is
//! `UNAUTHENTICATED`, a caller who is not an admin on an admin-only service
//! `PERMISSION_DENIED`.
//!
//! They are the axum template's types, so the bearer tokens and API keys
//! that API issues name the same caller here.

use uuid::Uuid;

pub use crate::db::types::Role;
pub use crate::shared::security::api_key::Credential;
pub use crate::shared::security::jwt::{Claims, JwtKeys};

/// A signed-in caller with a live, active account.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthUser {
    /// The token's `sub`.
    pub id: Uuid,
    /// The account's role, from the database.
    pub role: Role,
    /// How the caller proved who they are: a session or an API key.
    pub credential: Credential,
}

/// A signed-in caller whose account has the `admin` role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminUser(pub AuthUser);
