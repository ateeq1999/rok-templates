// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`OpenIdProvider`]: an [`IdentityProvider`] speaking `OpenID` Connect,
//! through the `openidconnect` crate.

use std::borrow::Cow;

use async_trait::async_trait;
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse,
};
use tokio::sync::OnceCell;

use super::{Authorization, Identity, IdentityProvider, Redemption};
use crate::config::OidcProviderSettings;
use crate::error::AppError;

/// A client built from the provider's discovery document.
type DiscoveredClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// An `OpenID` Connect provider, by its issuer: its endpoints and signing
/// keys come from the issuer's discovery document
/// (`/.well-known/openid-configuration`), fetched on first use and kept.
pub struct OpenIdProvider {
    issuer: String,
    client_id: String,
    client_secret: String,
    http: reqwest::Client,
    client: OnceCell<DiscoveredClient>,
}

impl OpenIdProvider {
    /// The provider `settings` describe; nothing is fetched yet.
    #[must_use]
    pub fn new(settings: &OidcProviderSettings) -> Self {
        // No redirects: a provider's endpoints answer directly, and
        // following one could reach somewhere the issuer didn't name.
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap_or_default();
        OpenIdProvider {
            issuer: settings.issuer.clone(),
            client_id: settings.client_id.clone(),
            client_secret: settings.client_secret.clone(),
            http,
            client: OnceCell::new(),
        }
    }

    /// The client, discovering the provider the first time.
    async fn client(&self) -> Result<&DiscoveredClient, AppError> {
        self.client
            .get_or_try_init(|| async {
                let issuer = IssuerUrl::new(self.issuer.clone()).map_err(internal)?;
                let metadata = CoreProviderMetadata::discover_async(issuer, &self.http)
                    .await
                    .map_err(internal)?;
                Ok(CoreClient::from_provider_metadata(
                    metadata,
                    ClientId::new(self.client_id.clone()),
                    Some(ClientSecret::new(self.client_secret.clone())),
                ))
            })
            .await
    }
}

#[async_trait]
impl IdentityProvider for OpenIdProvider {
    async fn authorize_url(&self, authorization: Authorization<'_>) -> Result<String, AppError> {
        let client = self.client().await?;
        let redirect = RedirectUrl::new(authorization.redirect_uri.to_owned()).map_err(internal)?;
        let state = authorization.state.to_owned();
        let nonce = authorization.nonce.to_owned();
        let challenge = PkceCodeChallenge::from_code_verifier_sha256(&PkceCodeVerifier::new(
            authorization.pkce_verifier.to_owned(),
        ));
        let (url, _, _) = client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                move || CsrfToken::new(state),
                move || Nonce::new(nonce),
            )
            .add_scope(Scope::new("email".to_owned()))
            .set_pkce_challenge(challenge)
            .set_redirect_uri(Cow::Owned(redirect))
            .url();
        Ok(url.into())
    }

    async fn redeem(&self, redemption: Redemption<'_>) -> Result<Identity, AppError> {
        let client = self.client().await?;
        let redirect = RedirectUrl::new(redemption.redirect_uri.to_owned()).map_err(internal)?;
        let response = client
            .exchange_code(AuthorizationCode::new(redemption.code.to_owned()))
            .map_err(internal)?
            .set_pkce_verifier(PkceCodeVerifier::new(redemption.pkce_verifier.to_owned()))
            .set_redirect_uri(Cow::Owned(redirect))
            .request_async(&self.http)
            .await
            .map_err(|_| AppError::BadRequest("the provider refused the code".to_owned()))?;
        let id_token = response
            .id_token()
            .ok_or_else(|| AppError::BadRequest("the provider sent no ID token".to_owned()))?;
        let claims = id_token
            .claims(
                &client.id_token_verifier(),
                &Nonce::new(redemption.nonce.to_owned()),
            )
            .map_err(|_| AppError::BadRequest("the provider's ID token is not valid".to_owned()))?;
        Ok(Identity {
            subject: claims.subject().as_str().to_owned(),
            email: claims.email().map(|email| email.as_str().to_owned()),
            email_verified: claims.email_verified().unwrap_or(false),
        })
    }
}

/// A `500` caused by `error`.
fn internal(error: impl std::error::Error + Send + Sync + 'static) -> AppError {
    AppError::Internal(Box::new(error))
}
