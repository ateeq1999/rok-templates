//! The development seed (`app::seed`, `cargo run --bin seed`): two
//! verified accounts the axum API's sign-in accepts, new passwords on every
//! run, and `--promote`.

mod support;

use grpc_api::app::seed::{self, ADMIN_EMAIL, USER_EMAIL};
use grpc_api::db::queries::auth::find_login;
use grpc_api::db::types::Role;
use grpc_api::shared::security::password;
use sqlx::PgPool;
use support::{Database, TestResult};

/// Whether `email`'s live account is verified and active, has `role`, and
/// takes `password`.
async fn signs_in(
    pool: &PgPool,
    email: &str,
    password: &str,
    role: Role,
) -> Result<bool, Box<dyn std::error::Error>> {
    let Some(login) = find_login(pool, email).await? else {
        return Ok(false);
    };
    let matches = password::verify(password.to_owned(), login.password_hash).await?;
    Ok(matches && login.email_verified && login.is_active && login.role == role)
}

#[tokio::test]
async fn the_seeded_accounts_sign_in_with_their_roles() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let seeded = seed::seed(&db.pool).await?;
    let found: Vec<(&str, Role)> = seeded
        .iter()
        .map(|account| (account.email.as_str(), account.role))
        .collect();
    assert_eq!(
        found,
        [(ADMIN_EMAIL, Role::Admin), (USER_EMAIL, Role::User)]
    );
    for account in &seeded {
        assert!(signs_in(&db.pool, &account.email, &account.password, account.role).await?);
    }

    // Again: the same accounts, new passwords; the old ones no longer work.
    let again = seed::seed(&db.pool).await?;
    let (old, new) = (&seeded[0], &again[0]);
    assert!(!signs_in(&db.pool, &old.email, &old.password, old.role).await?);
    assert!(signs_in(&db.pool, &new.email, &new.password, new.role).await?);
    db.drop().await
}

#[tokio::test]
async fn promote_makes_an_existing_account_an_admin() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let seeded = seed::seed(&db.pool).await?;
    let user = &seeded[1];
    let promoted = seed::promote(&db.pool, &user.email.to_uppercase()).await?;
    assert_eq!(promoted.as_deref(), Some(USER_EMAIL));
    assert!(signs_in(&db.pool, &user.email, &user.password, Role::Admin).await?);
    assert_eq!(seed::promote(&db.pool, "nobody@test.invalid").await?, None);
    db.drop().await
}
