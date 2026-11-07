// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The development seed, which `src/bin/seed.rs` runs: [`seed`] makes two
//! verified accounts to sign in with on the first run, an admin and an
//! ordinary user, and [`promote`] makes an existing account an admin, so
//! the first admin needs no `psql`. Sign-in still needs a verified email
//! (`REQUIRE_VERIFIED_EMAIL`): the seeded accounts have one.
//!
//! It is for local development: each run gives both accounts a new random
//! password (and ends their sessions), so never point it at a database
//! whose `admin@example.com` someone uses.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::queries::seed::{
    CreateSeedAccountParams, create_seed_account, promote_to_admin, reset_seed_account,
};
use crate::db::types::Role;
use crate::error::AppError;
use crate::shared::security::password;

/// The seeded admin's email.
pub const ADMIN_EMAIL: &str = "admin@example.com";

/// The seeded ordinary user's email.
pub const USER_EMAIL: &str = "user@example.com";

/// A seeded account and the password to sign in with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seeded {
    /// The email to sign in with.
    pub email: String,
    /// The password to sign in with, new on every run.
    pub password: String,
    /// The account's role.
    pub role: Role,
}

/// Makes [`ADMIN_EMAIL`] an admin's and [`USER_EMAIL`] an ordinary user's
/// verified, active account with a new random password each, creating
/// them or, when they exist, resetting them. Returns them, the admin first.
///
/// # Errors
///
/// The database's.
pub async fn seed(pool: &PgPool) -> Result<Vec<Seeded>, AppError> {
    let mut seeded = Vec::new();
    for (email, role) in [(ADMIN_EMAIL, Role::Admin), (USER_EMAIL, Role::User)] {
        seeded.push(seed_account(pool, email, role).await?);
    }
    Ok(seeded)
}

/// `email`'s verified, active account with `role` and a new password.
async fn seed_account(pool: &PgPool, email: &str, role: Role) -> Result<Seeded, AppError> {
    let bytes: [u8; 18] = rand::random();
    let new_password = URL_SAFE_NO_PAD.encode(bytes);
    let hash = password::hash(new_password.clone()).await?;
    let reset = reset_seed_account(pool, email, Some(&hash), role).await?;
    let email = if let Some(row) = reset {
        row.email
    } else {
        let params = CreateSeedAccountParams {
            id: Uuid::now_v7().into(),
            email: email.to_owned(),
            password_hash: Some(hash),
            role,
        };
        create_seed_account(pool, params).await?.email
    };
    Ok(Seeded {
        email,
        password: new_password,
        role,
    })
}

/// Makes the live account with `email` (whatever its case) an admin.
/// Returns its email, or `None` when there is no such account.
///
/// # Errors
///
/// The database's.
pub async fn promote(pool: &PgPool, email: &str) -> Result<Option<String>, AppError> {
    Ok(promote_to_admin(pool, email.trim())
        .await?
        .map(|row| row.email))
}
