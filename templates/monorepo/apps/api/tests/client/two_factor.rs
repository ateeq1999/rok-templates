//! Two-factor sign-in through the client: set up, enabled with a code,
//! signed in with a second step, turned off with the password — what
//! `two_factor.rs` drives through the router.

use std::time::{SystemTime, UNIX_EPOCH};

use api_client::{DisableTotpDto, EnableTotpDto, LoginResponse, VerifyTotpDto};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::support::{PASSWORD, TestResult};
use crate::{Api, Error, credentials, session};

const EMAIL: &str = "ada@example.com";

/// The current code of the base32 `secret`, as an authenticator app
/// computes it.
fn code(secret: &str) -> Result<String, Error> {
    let bytes = Secret::Encoded(secret.to_owned()).to_bytes()?;
    let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 0, 30, bytes, None, String::new());
    let step = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 30;
    Ok(totp.generate(step * 30))
}

#[tokio::test]
async fn two_factor_is_set_up_used_to_sign_in_and_turned_off() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    let token = api.register_verified(EMAIL, PASSWORD).await?;
    api.client.set_token(Some(token));
    let setup = api.client.auth().setup().await?;
    assert!(
        setup.provisioning_uri.starts_with("otpauth://totp/"),
        "{setup:?}"
    );
    let enabled = api
        .client
        .auth()
        .enable(&EnableTotpDto {
            code: code(&setup.secret)?,
        })
        .await?;
    assert_eq!(enabled.recovery_codes.len(), 10);

    // The right password now asks for the second factor; a recovery code
    // stands in for the app's code (the current one is spent).
    let pending = match api
        .client
        .auth()
        .login(&credentials(EMAIL, PASSWORD))
        .await?
    {
        LoginResponse::PendingToken(pending) => pending,
        LoginResponse::TokenResponse(_) => return Err("signed in without a second factor".into()),
    };
    assert!(pending.requires_totp);
    let verified = api
        .client
        .auth()
        .verify2fa(&VerifyTotpDto {
            pending_token: pending.pending_token,
            code: enabled.recovery_codes[0].clone(),
        })
        .await?;
    api.client.set_token(Some(verified.access_token));

    // A wrong password is an answer, not the end of the session.
    let wrong = DisableTotpDto {
        current_password: "not the password".to_owned(),
    };
    let err = api
        .client
        .keep_session()
        .auth()
        .disable(&wrong)
        .await
        .err()
        .ok_or("disabled with a wrong password")?;
    assert_eq!((err.status, err.code.as_str()), (400, "wrong_password"));
    api.client
        .auth()
        .disable(&DisableTotpDto {
            current_password: PASSWORD.to_owned(),
        })
        .await?;
    session(
        api.client
            .auth()
            .login(&credentials(EMAIL, PASSWORD))
            .await?,
    )?;
    api.stop().await
}
