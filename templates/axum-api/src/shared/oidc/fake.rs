// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`FakeIdentityProvider`]: an [`IdentityProvider`] in memory, for tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use super::{Authorization, Identity, IdentityProvider, Redemption, pkce_challenge};
use crate::error::AppError;
use crate::shared::security::token::SecretToken;

/// Where [`FakeIdentityProvider`]'s authorization URLs point.
pub const FAKE_AUTHORIZE_URL: &str = "https://provider.test/authorize";

/// An identity provider in memory: a test says who signs in
/// ([`FakeIdentityProvider::code_for`]) and brings the code it answers to
/// the callback, as the provider's redirect would. It checks what a real
/// one checks: a code works once, for the PKCE verifier whose challenge
/// went out with the same `state`.
#[derive(Clone, Default)]
pub struct FakeIdentityProvider {
    inner: Arc<Mutex<Fake>>,
}

#[derive(Default)]
struct Fake {
    /// The PKCE challenge each authorization URL carried, by its state.
    challenges: HashMap<String, String>,
    /// The identity each code signs in as.
    codes: HashMap<String, Identity>,
}

impl FakeIdentityProvider {
    /// A provider no one has signed in at yet.
    #[must_use]
    pub fn new() -> Self {
        FakeIdentityProvider::default()
    }

    /// A single-use code the callback redeems for `identity`.
    #[must_use]
    pub fn code_for(&self, identity: Identity) -> String {
        let code = SecretToken::generate().raw;
        if let Ok(mut fake) = self.inner.lock() {
            fake.codes.insert(code.clone(), identity);
        }
        code
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Fake>, AppError> {
        self.inner
            .lock()
            .map_err(|_| AppError::Internal("the fake provider's lock is poisoned".into()))
    }
}

#[async_trait]
impl IdentityProvider for FakeIdentityProvider {
    async fn authorize_url(&self, authorization: Authorization<'_>) -> Result<String, AppError> {
        let challenge = pkce_challenge(authorization.pkce_verifier);
        self.lock()?
            .challenges
            .insert(authorization.state.to_owned(), challenge.clone());
        let mut url = url::Url::parse(FAKE_AUTHORIZE_URL)
            .map_err(|error| AppError::Internal(Box::new(error)))?;
        url.query_pairs_mut()
            .append_pair("redirect_uri", authorization.redirect_uri)
            .append_pair("state", authorization.state)
            .append_pair("nonce", authorization.nonce)
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256");
        Ok(url.into())
    }

    async fn redeem(&self, redemption: Redemption<'_>) -> Result<Identity, AppError> {
        let mut fake = self.lock()?;
        let challenge = pkce_challenge(redemption.pkce_verifier);
        if !fake.challenges.values().any(|sent| *sent == challenge) {
            return Err(AppError::BadRequest(
                "the PKCE verifier matches no authorization".to_owned(),
            ));
        }
        fake.codes
            .remove(redemption.code)
            .ok_or_else(|| AppError::BadRequest("the provider refused the code".to_owned()))
    }
}
