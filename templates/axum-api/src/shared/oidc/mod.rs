// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Sign-in with an `OpenID` provider, the outside half: [`IdentityProvider`],
//! the seam a provider is reached through, and [`IdentityProviders`], the
//! ones `OIDC_PROVIDERS` configures, by name. [`OpenIdProvider`] speaks
//! `OpenID` Connect (discovery, the authorization code flow with PKCE, a
//! verified ID token); [`FakeIdentityProvider`] stands in for one in tests.
//! The rules (who a returning identity signs in as, when an email links)
//! are the `oidc` feature's. Framework-free, so both templates share it.

mod fake;
mod openid;

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

pub use fake::FakeIdentityProvider;
pub use openid::OpenIdProvider;

use crate::config::OidcProviderSettings;
use crate::error::AppError;

/// Who the person signing in is at a provider, from its verified ID token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The provider's id for the person (`sub`): what a Linked identity is
    /// matched by.
    pub subject: String,
    /// Their email, when the provider shares one.
    pub email: Option<String>,
    /// Whether the provider says it verified that email (`email_verified`).
    pub email_verified: bool,
}

/// What the provider's authorization URL carries: where to come back, and
/// the single-use values the callback is checked against.
#[derive(Debug, Clone, Copy)]
pub struct Authorization<'a> {
    /// The callback URL, as registered with the provider.
    pub redirect_uri: &'a str,
    /// The `state` the callback must bring back.
    pub state: &'a str,
    /// The `nonce` the ID token must carry.
    pub nonce: &'a str,
    /// The PKCE verifier; the URL carries its S256 challenge
    /// ([`pkce_challenge`]).
    pub pkce_verifier: &'a str,
}

/// What redeeming the callback's code takes.
#[derive(Debug, Clone, Copy)]
pub struct Redemption<'a> {
    /// The code the callback brought.
    pub code: &'a str,
    /// The callback URL the authorization named.
    pub redirect_uri: &'a str,
    /// The PKCE verifier whose challenge the authorization carried.
    pub pkce_verifier: &'a str,
    /// The nonce the ID token must carry.
    pub nonce: &'a str,
}

/// An `OpenID` provider: where to send the person, and who they turned out
/// to be.
#[async_trait]
pub trait IdentityProvider: Send + Sync {
    /// The provider's authorization URL for `authorization`.
    async fn authorize_url(&self, authorization: Authorization<'_>) -> Result<String, AppError>;

    /// Redeems the callback's code for the person's identity, from an ID
    /// token whose signature, audience, expiry and nonce are checked. A
    /// code the provider refuses is a `400`.
    async fn redeem(&self, redemption: Redemption<'_>) -> Result<Identity, AppError>;
}

/// One configured provider.
#[derive(Clone)]
pub struct Provider {
    /// How it is reached.
    pub connection: Arc<dyn IdentityProvider>,
    /// `OIDC_<NAME>_TRUST_EMAIL`: an unknown identity whose verified email
    /// belongs to an account links to it. Set it only for a provider that
    /// owns the emails it verifies, such as Google's for Gmail addresses.
    pub trust_email: bool,
}

/// The configured providers, by name: none, and sign-in with a provider is
/// off (its routes answer `404`).
#[derive(Clone, Default)]
pub struct IdentityProviders {
    providers: BTreeMap<String, Provider>,
}

impl IdentityProviders {
    /// The providers `settings` configure, each an [`OpenIdProvider`]
    /// coming back to `{app_url}/api/v1/auth/oidc/{name}/callback`. Each
    /// discovers its endpoints on first use, so starting needs no network.
    #[must_use]
    pub fn from_settings(settings: &[OidcProviderSettings]) -> Self {
        let mut providers = IdentityProviders::default();
        for provider in settings {
            providers = providers.with(
                &provider.name,
                Arc::new(OpenIdProvider::new(provider)),
                provider.trust_email,
            );
        }
        providers
    }

    /// These providers, and `connection` as `name` (replacing one so
    /// named).
    #[must_use]
    pub fn with(
        mut self,
        name: &str,
        connection: Arc<dyn IdentityProvider>,
        trust_email: bool,
    ) -> Self {
        self.providers.insert(
            name.to_owned(),
            Provider {
                connection,
                trust_email,
            },
        );
        self
    }

    /// Whether no provider is configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// The provider `name`, or a `404`: none so named, or none at all.
    pub fn get(&self, name: &str) -> Result<&Provider, AppError> {
        if self.providers.is_empty() {
            return Err(not_configured());
        }
        self.providers
            .get(name)
            .ok_or_else(|| AppError::NotFound(format!("no OpenID provider is named `{name}`")))
    }

    /// The configured providers' names, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.providers.keys().map(String::as_str)
    }
}

/// The `404` of every route while no provider is configured.
#[must_use]
pub fn not_configured() -> AppError {
    AppError::NotFound("sign-in with an OpenID provider is not configured".to_owned())
}

/// The S256 PKCE challenge of `verifier`: its SHA-256, base64url without
/// padding (RFC 7636).
#[must_use]
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}
