//! API keys through the client: created, listed, used in place of a
//! session (as a bearer token or the `X-API-Key` header) and revoked — what
//! `api_keys.rs` drives through the router.

use api_client::reqwest::header::{HeaderName, HeaderValue};
use api_client::{ApiKeyScope, Client, CreateApiKeyDto};

use crate::Api;
use crate::support::{TestResult, user};

#[tokio::test]
async fn a_key_is_shown_once_works_for_its_owner_and_is_revoked() -> TestResult {
    let Some(api) = Api::start().await? else {
        return Ok(());
    };
    api.client.set_token(Some(user(&api.db.pool).await?));
    let created = api
        .client
        .api_keys()
        .create(&CreateApiKeyDto {
            name: "ci".to_owned(),
            scope: Some(Some(ApiKeyScope::Write)),
            expires_in_days: None,
        })
        .await?;
    assert!(created.key.starts_with(&created.key_prefix), "{created:?}");
    assert_eq!(created.scope, ApiKeyScope::Write);
    assert_eq!(created.last_used_at, None);

    let me = api.client.users().me().await?;
    let bearer = Client::new(api.client.base_url());
    bearer.set_token(Some(created.key.clone()));
    assert_eq!(bearer.users().me().await?, me);
    let header = Client::new(api.client.base_url()).with_header(
        HeaderName::from_static("x-api-key"),
        HeaderValue::from_str(&created.key)?,
    );
    assert_eq!(header.users().me().await?, me);

    let listed = api.client.api_keys().list().await?;
    assert_eq!(listed.len(), 1, "{listed:?}");
    assert_eq!(listed[0].id, created.id);
    assert!(listed[0].last_used_at.is_some(), "{listed:?}");

    api.client.api_keys().revoke(&created.id).await?;
    let err = bearer
        .users()
        .me()
        .await
        .err()
        .ok_or("a revoked key worked")?;
    assert_eq!(err.status, 401);
    assert!(api.client.api_keys().list().await?.is_empty());
    api.stop().await
}
