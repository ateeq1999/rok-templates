//! Signing up, in and out, the profile and the password, through the
//! client: what `sign_up.rs`, `login.rs`, `auth.rs`, `account.rs` and
//! `password.rs` drive through the router.

use std::sync::{Arc, Mutex};

use api_client::{
    ChangePasswordDto, Client, ForgotPasswordDto, ResetPasswordDto, UpdateProfileDto,
    VerifyEmailDto,
};

use crate::support::{PASSWORD, TestResult, mailed, mailed_token};
use crate::{Api, credentials, session};

const EMAIL: &str = "ada@example.com";

#[tokio::test]
async fn sign_up_verify_sign_in_and_edit_the_profile() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.health().liveness().await?;
    api.client.health().readiness().await?;

    let user = api
        .client
        .auth()
        .register(&credentials(EMAIL, PASSWORD))
        .await?;
    assert_eq!(user.email, EMAIL);
    assert!(!user.email_verified);
    assert!(user.has_password);

    // Unverified: signing in is forbidden.
    let err = api
        .client
        .auth()
        .login(&credentials(EMAIL, PASSWORD))
        .await
        .err()
        .ok_or("an unverified account signed in")?;
    assert_eq!((err.status, err.code.as_str()), (403, "forbidden"));

    let token = mailed(&api.db.pool)
        .await?
        .iter()
        .find(|mail| mail.to == EMAIL)
        .and_then(mailed_token)
        .ok_or("no verification mail")?;
    api.client
        .auth()
        .verify_email(&VerifyEmailDto { token })
        .await?;
    let token = session(
        api.client
            .auth()
            .login(&credentials(EMAIL, PASSWORD))
            .await?,
    )?;
    api.client.set_token(Some(token));

    let me = api.client.auth().me().await?;
    assert_eq!((me.email.as_str(), me.email_verified), (EMAIL, true));
    assert_eq!(api.client.users().me().await?.id, me.id);

    // A property left out keeps its value, `null` clears it.
    let named = UpdateProfileDto {
        display_name: Some(Some("Ada".to_owned())),
    };
    let updated = api.client.users().update_me(&named).await?;
    assert_eq!(updated.display_name.as_deref(), Some("Ada"));
    let kept = api
        .client
        .users()
        .update_me(&UpdateProfileDto::default())
        .await?;
    assert_eq!(kept.display_name.as_deref(), Some("Ada"));
    let cleared = UpdateProfileDto {
        display_name: Some(None),
    };
    let cleared = api.client.users().update_me(&cleared).await?;
    assert_eq!(cleared.display_name, None);
    api.stop().await
}

#[tokio::test]
async fn a_rejected_body_names_its_fields() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    let err = api
        .client
        .auth()
        .register(&credentials("not an email", "short"))
        .await
        .err()
        .ok_or("a broken body was accepted")?;
    assert_eq!(err.status, 422, "{err}");
    assert!(!err.field("password").is_empty(), "{err:?}");
    assert!(!err.field("email").is_empty(), "{err:?}");
    api.stop().await
}

/// A `401` to a call that sent the session's token ends the session; one
/// to a sign-in, or through `keep_session`, does not.
#[tokio::test]
async fn only_a_rejected_session_token_calls_the_handler() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.register_verified(EMAIL, PASSWORD).await?;
    let rejected: Arc<Mutex<Vec<String>>> = Arc::default();
    let seen = rejected.clone();
    let client = Client::new(api.client.base_url()).on_unauthorized(move |token| {
        if let Ok(mut seen) = seen.lock() {
            seen.push(token.to_owned());
        }
    });

    let err = client
        .auth()
        .login(&credentials(EMAIL, "not the password"))
        .await
        .err()
        .ok_or("a wrong password signed in")?;
    assert_eq!((err.status, err.code.as_str()), (401, "unauthorized"));

    client.set_token(Some("not a token".to_owned()));
    let err = client
        .keep_session()
        .auth()
        .me()
        .await
        .err()
        .ok_or("a bad token was accepted")?;
    assert_eq!(err.status, 401);
    assert!(rejected.lock().map_err(|e| e.to_string())?.is_empty());

    let err = client.users().me().await.err().ok_or("a bad token")?;
    assert_eq!(err.status, 401);
    assert_eq!(
        *rejected.lock().map_err(|e| e.to_string())?,
        vec!["not a token".to_owned()]
    );
    api.stop().await
}

#[tokio::test]
async fn the_password_is_changed_and_reset() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    let token = api.register_verified(EMAIL, PASSWORD).await?;
    api.client.set_token(Some(token));
    let changed = "another long and unusual passphrase";
    api.client
        .auth()
        .change(&ChangePasswordDto {
            current_password: PASSWORD.to_owned(),
            new_password: changed.to_owned(),
        })
        .await?;
    api.client.set_token(None);
    session(
        api.client
            .auth()
            .login(&credentials(EMAIL, changed))
            .await?,
    )?;

    mailed(&api.db.pool).await?;
    api.client
        .auth()
        .forgot(&ForgotPasswordDto {
            email: EMAIL.to_owned(),
        })
        .await?;
    let token = mailed(&api.db.pool)
        .await?
        .iter()
        .find(|mail| mail.to == EMAIL)
        .and_then(mailed_token)
        .ok_or("no reset mail")?;
    let reset = "a third long and unusual passphrase";
    api.client
        .auth()
        .reset(&ResetPasswordDto {
            token,
            new_password: reset.to_owned(),
        })
        .await?;
    session(api.client.auth().login(&credentials(EMAIL, reset)).await?)?;
    api.stop().await
}
