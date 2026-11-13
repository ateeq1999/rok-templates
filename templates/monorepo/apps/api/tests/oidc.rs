//! Sign-in with an OpenID provider, against a fake provider: `start` sends
//! the browser to the provider, whose redirect brings a code to `callback`,
//! which sends the browser back to the App (`{APP_URL}/auth/oidc`) with a
//! session, a two-factor step, or why not in the fragment. A returning
//! identity signs in as its account; a new email signs up; an email an
//! account already has links only from a provider trusted with emails. A
//! signed-in account lists, links and unlinks its identities, but never
//! unlinks its last way in. A mobile App's sign-in comes back to its deep
//! link (`<APP_SCHEME>://auth/oidc`) with a one-minute, single-use code it
//! trades for a session. The Apps read the providers for their buttons.
//! With no provider configured, every route but that list answers 404.

mod support;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{Method, StatusCode};
use axum_api::app::AppState;
use axum_api::shared::oidc::{FakeIdentityProvider, Identity, IdentityProviders};
use serde_json::{Value, json};
use support::{
    APP_URL, Database, PASSWORD, TestResult, call_with, register_verified, send, state, state_with,
};
use totp_rs::{Algorithm, Secret, TOTP};

type Error = Box<dyn std::error::Error>;

/// The fake provider's name in the routes.
const PROVIDER: &str = "acme";

/// The application over `pool` with the fake provider `fake` configured as
/// [`PROVIDER`], trusted with emails when `trust_email`.
fn configured(
    pool: &sqlx::PgPool,
    fake: &FakeIdentityProvider,
    trust_email: bool,
) -> Result<AppState, Error> {
    let providers =
        IdentityProviders::default().with(PROVIDER, Arc::new(fake.clone()), trust_email);
    Ok(state(pool)?.with_identity_providers(providers))
}

/// An identity at the provider: `subject`, with a verified `email`.
fn person(subject: &str, email: &str) -> Identity {
    Identity {
        subject: subject.to_owned(),
        email: Some(email.to_owned()),
        email_verified: true,
    }
}

/// The query parameter `name` of `url`.
fn query_param(url: &str, name: &str) -> Result<String, Error> {
    let url = url::Url::parse(url)?;
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| format!("no `{name}` in {url}").into())
}

/// Brings `identity` back from the provider for the authorization at
/// `provider_url` (a `start` redirect or a `link` answer): the callback's
/// redirect to the App, which must be `{APP_URL}/auth/oidc`, as its
/// fragment's parameters.
async fn come_back(
    state: &AppState,
    fake: &FakeIdentityProvider,
    provider_url: &str,
    identity: Identity,
) -> Result<HashMap<String, String>, Error> {
    let csrf = query_param(provider_url, "state")?;
    let code = fake.code_for(identity);
    callback(state, &code, &csrf).await
}

/// `GET …/callback?code=&state=`: the redirect's fragment parameters.
async fn callback(
    state: &AppState,
    code: &str,
    csrf: &str,
) -> Result<HashMap<String, String>, Error> {
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/callback?code={code}&state={csrf}");
    let answer = send(state, None, Method::GET, &uri, &[], Vec::new()).await?;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{:?}", answer.json());
    let location = answer.header("location").ok_or("no location")?;
    let (base, fragment) = location
        .split_once('#')
        .ok_or_else(|| format!("no fragment in {location}"))?;
    assert_eq!(base, format!("{APP_URL}/auth/oidc"));
    Ok(url::form_urlencoded::parse(fragment.as_bytes())
        .into_owned()
        .collect())
}

/// `GET …/start`: where it sends the browser.
async fn start(state: &AppState) -> Result<String, Error> {
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/start");
    let answer = send(state, None, Method::GET, &uri, &[], Vec::new()).await?;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{:?}", answer.json());
    Ok(answer.header("location").ok_or("no location")?.to_owned())
}

/// Signs `identity` in at the provider, from `start` to the App.
async fn sign_in(
    state: &AppState,
    fake: &FakeIdentityProvider,
    identity: Identity,
) -> Result<HashMap<String, String>, Error> {
    let provider_url = start(state).await?;
    come_back(state, fake, &provider_url, identity).await
}

/// The session token a sign-in's fragment carries.
fn session(fragment: &HashMap<String, String>) -> Result<String, Error> {
    assert_eq!(
        fragment.get("token_type").map(String::as_str),
        Some("Bearer")
    );
    fragment
        .get("access_token")
        .cloned()
        .ok_or_else(|| format!("no session in {fragment:?}").into())
}

/// `GET /auth/me` with `token`.
async fn me(state: &AppState, token: &str) -> Result<Value, Error> {
    let answer = call_with(state, Some(token), Method::GET, "/api/v1/auth/me", None).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    Ok(answer.body)
}

/// `GET /auth/oidc/identities` with `token`.
async fn identities(state: &AppState, token: &str) -> Result<Vec<Value>, Error> {
    let uri = "/api/v1/auth/oidc/identities";
    let answer = call_with(state, Some(token), Method::GET, uri, None).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    Ok(answer.body.as_array().ok_or("not a list")?.clone())
}

/// `POST /auth/oidc/{provider}/link` with `token`: the provider URL.
async fn link(state: &AppState, token: &str) -> Result<String, Error> {
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/link");
    let answer = call_with(state, Some(token), Method::POST, &uri, None).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    Ok(answer.body["url"].as_str().ok_or("no url")?.to_owned())
}

/// `DELETE /auth/oidc/identities/{id}` with `token`.
async fn unlink(state: &AppState, token: &str, id: &Value) -> Result<support::Answer, Error> {
    let id = id.as_str().ok_or("no id")?;
    let uri = format!("/api/v1/auth/oidc/identities/{id}");
    call_with(state, Some(token), Method::DELETE, &uri, None).await
}

#[tokio::test]
async fn a_new_email_signs_up_and_the_identity_signs_in_again() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;

    let provider_url = start(&state).await?;
    assert!(provider_url.starts_with("https://provider.test/authorize?"));
    assert_eq!(query_param(&provider_url, "code_challenge_method")?, "S256");
    assert_eq!(
        query_param(&provider_url, "redirect_uri")?,
        format!("{APP_URL}/api/v1/auth/oidc/{PROVIDER}/callback")
    );
    let ada = person("ada-at-acme", "Ada@Example.com");
    let fragment = come_back(&state, &fake, &provider_url, ada.clone()).await?;
    let account = me(&state, &session(&fragment)?).await?;
    assert_eq!(account["email"], "ada@example.com");
    assert_eq!(account["email_verified"], true);
    assert_eq!(account["has_password"], false);

    // The same person again, whatever email the provider now gives: the
    // same account.
    let again = Identity {
        email: Some("ada@elsewhere.example".to_owned()),
        ..ada
    };
    let fragment = sign_in(&state, &fake, again).await?;
    let token = session(&fragment)?;
    assert_eq!(me(&state, &token).await?["id"], account["id"]);
    let linked = identities(&state, &token).await?;
    assert_eq!(linked.len(), 1, "{linked:?}");
    assert_eq!(linked[0]["provider"], PROVIDER);
    assert_eq!(linked[0]["email"], "ada@example.com");
    db.drop().await
}

#[tokio::test]
async fn an_accounts_email_is_refused_from_a_provider_not_trusted_with_emails() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let token = register_verified(&db.pool, "grace@example.com", PASSWORD).await?;

    let fragment = sign_in(&state, &fake, person("grace", "grace@example.com")).await?;
    assert_eq!(
        fragment.get("error").map(String::as_str),
        Some("email_in_use")
    );
    assert!(
        fragment.get("message").is_some_and(|m| m.contains("link")),
        "{fragment:?}"
    );
    assert!(!fragment.contains_key("access_token"), "{fragment:?}");
    assert!(identities(&state, &token).await?.is_empty());
    db.drop().await
}

#[tokio::test]
async fn a_provider_trusted_with_emails_links_a_verified_one() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, true)?;
    let token = register_verified(&db.pool, "grace@example.com", PASSWORD).await?;

    // An unverified email is no proof, even from a trusted provider.
    let unverified = Identity {
        email_verified: false,
        ..person("grace", "grace@example.com")
    };
    let fragment = sign_in(&state, &fake, unverified).await?;
    assert_eq!(
        fragment.get("error").map(String::as_str),
        Some("email_in_use")
    );

    let fragment = sign_in(&state, &fake, person("grace", "GRACE@example.com")).await?;
    let signed_in = me(&state, &session(&fragment)?).await?;
    assert_eq!(signed_in["id"], me(&state, &token).await?["id"]);
    assert_eq!(identities(&state, &token).await?.len(), 1);
    db.drop().await
}

#[tokio::test]
async fn a_signed_in_account_links_and_unlinks_an_identity() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let token = register_verified(&db.pool, "linus@example.com", PASSWORD).await?;
    let account = me(&state, &token).await?;

    // Another email at the provider is fine: linking is by who signed in.
    let provider_url = link(&state, &token).await?;
    let linus = person("linus", "linus@elsewhere.example");
    let fragment = come_back(&state, &fake, &provider_url, linus.clone()).await?;
    assert_eq!(fragment.get("linked").map(String::as_str), Some(PROVIDER));
    let linked = identities(&state, &token).await?;
    assert_eq!(linked.len(), 1, "{linked:?}");

    let fragment = sign_in(&state, &fake, linus.clone()).await?;
    assert_eq!(me(&state, &session(&fragment)?).await?["id"], account["id"]);

    // Linked to one account, an identity can't be linked to another.
    let other = register_verified(&db.pool, "other@example.com", PASSWORD).await?;
    let provider_url = link(&state, &other).await?;
    let fragment = come_back(&state, &fake, &provider_url, linus).await?;
    assert_eq!(
        fragment.get("error").map(String::as_str),
        Some("identity_in_use")
    );

    // With a password, the account keeps a way in.
    let answer = unlink(&state, &token, &linked[0]["id"]).await?;
    assert_eq!(answer.status, StatusCode::NO_CONTENT, "{}", answer.body);
    assert!(identities(&state, &token).await?.is_empty());
    let answer = unlink(&state, &token, &linked[0]["id"]).await?;
    assert_eq!(answer.status, StatusCode::NOT_FOUND, "{}", answer.body);
    db.drop().await
}

#[tokio::test]
async fn unlinking_the_last_way_in_is_a_422_saying_why() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let fragment = sign_in(&state, &fake, person("first", "mo@example.com")).await?;
    let token = session(&fragment)?;
    let first = identities(&state, &token).await?;

    let answer = unlink(&state, &token, &first[0]["id"]).await?;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    let message = answer.body["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("password"), "{message}");

    let provider_url = link(&state, &token).await?;
    come_back(
        &state,
        &fake,
        &provider_url,
        person("second", "mo@else.example"),
    )
    .await?;
    let answer = unlink(&state, &token, &first[0]["id"]).await?;
    assert_eq!(answer.status, StatusCode::NO_CONTENT, "{}", answer.body);
    let last = identities(&state, &token).await?;
    assert_eq!(last.len(), 1);
    let answer = unlink(&state, &token, &last[0]["id"]).await?;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    db.drop().await
}

/// The authenticator code of the base32 `secret` now.
fn totp_code(secret: &str) -> Result<String, Error> {
    let bytes = Secret::Encoded(secret.to_owned()).to_bytes()?;
    let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 0, 30, bytes, None, String::new());
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(totp.generate(now))
}

#[tokio::test]
async fn an_account_with_two_factor_gives_the_code_after_the_provider() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let token = register_verified(&db.pool, "tess@example.com", PASSWORD).await?;
    let provider_url = link(&state, &token).await?;
    come_back(
        &state,
        &fake,
        &provider_url,
        person("tess", "tess@example.com"),
    )
    .await?;
    let setup = call_with(
        &state,
        Some(&token),
        Method::POST,
        "/api/v1/auth/2fa/setup",
        None,
    )
    .await?;
    let secret = setup.body["secret"].as_str().ok_or("no secret")?.to_owned();
    let body = json!({"code": totp_code(&secret)?});
    let enable = "/api/v1/auth/2fa/enable";
    let answer = call_with(&state, Some(&token), Method::POST, enable, Some(body)).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);

    let fragment = sign_in(&state, &fake, person("tess", "tess@example.com")).await?;
    assert_eq!(
        fragment.get("requires_totp").map(String::as_str),
        Some("true")
    );
    assert!(!fragment.contains_key("access_token"), "{fragment:?}");
    let pending = fragment.get("pending_token").ok_or("no pending token")?;

    // The code of the next step: the current one enabled two-factor.
    let bytes = Secret::Encoded(secret.clone()).to_bytes()?;
    let totp = TOTP::new_unchecked(Algorithm::SHA1, 6, 0, 30, bytes, None, String::new());
    let next = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 30;
    let body = json!({"pending_token": pending, "code": totp.generate(next)});
    let verify = "/api/v1/auth/2fa/verify";
    let answer = call_with(&state, None, Method::POST, verify, Some(body)).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert!(answer.body["access_token"].is_string(), "{}", answer.body);
    db.drop().await
}

#[tokio::test]
async fn a_callback_is_single_use() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let provider_url = start(&state).await?;
    let csrf = query_param(&provider_url, "state")?;
    let code = fake.code_for(person("once", "once@example.com"));
    let fragment = callback(&state, &code, &csrf).await?;
    session(&fragment)?;

    let code = fake.code_for(person("once", "once@example.com"));
    let fragment = callback(&state, &code, &csrf).await?;
    assert_eq!(
        fragment.get("error").map(String::as_str),
        Some("invalid_request")
    );
    let fragment = callback(&state, "any", "a-state-never-issued").await?;
    assert_eq!(
        fragment.get("error").map(String::as_str),
        Some("invalid_request")
    );
    db.drop().await
}

#[tokio::test]
async fn with_no_provider_configured_every_route_answers_404() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let token = register_verified(&db.pool, "nobody@example.com", PASSWORD).await?;
    let fake = FakeIdentityProvider::new();
    for (state, provider) in [
        (state(&db.pool)?, PROVIDER),
        (configured(&db.pool, &fake, false)?, "unknown"),
    ] {
        for (method, uri) in [
            (Method::GET, format!("/api/v1/auth/oidc/{provider}/start")),
            (
                Method::GET,
                format!("/api/v1/auth/oidc/{provider}/callback?code=c&state=s"),
            ),
            (Method::POST, format!("/api/v1/auth/oidc/{provider}/link")),
        ] {
            let answer = call_with(&state, Some(&token), method, &uri, None).await?;
            assert_eq!(
                answer.status,
                StatusCode::NOT_FOUND,
                "{uri}: {}",
                answer.body
            );
        }
    }
    let unconfigured = state(&db.pool)?;
    let uri = "/api/v1/auth/oidc/identities";
    let answer = call_with(&unconfigured, Some(&token), Method::GET, uri, None).await?;
    assert_eq!(answer.status, StatusCode::NOT_FOUND, "{}", answer.body);
    let uri = format!("{uri}/{}", uuid::Uuid::now_v7());
    let answer = call_with(&unconfigured, Some(&token), Method::DELETE, &uri, None).await?;
    assert_eq!(answer.status, StatusCode::NOT_FOUND, "{}", answer.body);
    db.drop().await
}

/// The mobile App's deep-link scheme in these tests.
const APP_SCHEME: &str = "myapp";

/// The application over `pool` with the fake provider `fake` configured as
/// [`PROVIDER`] and the mobile App's scheme [`APP_SCHEME`].
fn configured_for_mobile(
    pool: &sqlx::PgPool,
    fake: &FakeIdentityProvider,
) -> Result<AppState, Error> {
    let providers = IdentityProviders::default().with(PROVIDER, Arc::new(fake.clone()), false);
    Ok(state_with(pool, &[("APP_SCHEME", APP_SCHEME)])?.with_identity_providers(providers))
}

/// Signs `identity` in from the mobile App (`start?client=mobile`): the
/// callback's deep link, which must be `myapp://auth/oidc`, as its query
/// parameters.
async fn sign_in_from_mobile(
    state: &AppState,
    fake: &FakeIdentityProvider,
    identity: Identity,
) -> Result<HashMap<String, String>, Error> {
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/start?client=mobile");
    let answer = send(state, None, Method::GET, &uri, &[], Vec::new()).await?;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{:?}", answer.json());
    let provider_url = answer.header("location").ok_or("no location")?.to_owned();
    let csrf = query_param(&provider_url, "state")?;
    let code = fake.code_for(identity);
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/callback?code={code}&state={csrf}");
    let answer = send(state, None, Method::GET, &uri, &[], Vec::new()).await?;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{:?}", answer.json());
    let location = answer.header("location").ok_or("no location")?;
    let (base, query) = location
        .split_once('?')
        .ok_or_else(|| format!("no query in {location}"))?;
    assert_eq!(base, format!("{APP_SCHEME}://auth/oidc"));
    Ok(url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect())
}

/// `POST /auth/oidc/handoff` with the deep link's `code`.
async fn exchange(state: &AppState, code: &str) -> Result<support::Answer, Error> {
    let uri = "/api/v1/auth/oidc/handoff";
    call_with(state, None, Method::POST, uri, Some(json!({"code": code}))).await
}

/// Moves every hand-over of `pool` `seconds` into the past, as if that long
/// had gone by since the callback.
async fn age_handoffs(pool: &sqlx::PgPool, seconds: i32) -> Result<(), Error> {
    sqlx::query(
        "UPDATE oidc_handoffs SET created_at = created_at - make_interval(secs => $1), \
         expires_at = expires_at - make_interval(secs => $1)",
    )
    .bind(f64::from(seconds))
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn a_mobile_sign_in_hands_over_a_code_that_works_once() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured_for_mobile(&db.pool, &fake)?;

    let link = sign_in_from_mobile(&state, &fake, person("ada", "ada@example.com")).await?;
    assert!(!link.contains_key("access_token"), "{link:?}");
    let code = link.get("code").ok_or("no code")?;

    let answer = exchange(&state, code).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body["token_type"], "Bearer");
    let token = answer.body["access_token"].as_str().ok_or("no token")?;
    assert_eq!(me(&state, token).await?["email"], "ada@example.com");

    let again = exchange(&state, code).await?;
    assert_eq!(again.status, StatusCode::BAD_REQUEST, "{}", again.body);
    assert_eq!(again.body["error"]["code"], "invalid_token");
    let never = exchange(&state, "a-code-never-issued").await?;
    assert_eq!(never.status, StatusCode::BAD_REQUEST, "{}", never.body);
    db.drop().await
}

#[tokio::test]
async fn a_hand_over_code_expires_after_a_minute() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured_for_mobile(&db.pool, &fake)?;
    let ada = person("ada", "ada@example.com");

    let link = sign_in_from_mobile(&state, &fake, ada.clone()).await?;
    age_handoffs(&db.pool, 55).await?;
    let answer = exchange(&state, link.get("code").ok_or("no code")?).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);

    let link = sign_in_from_mobile(&state, &fake, ada).await?;
    age_handoffs(&db.pool, 61).await?;
    let answer = exchange(&state, link.get("code").ok_or("no code")?).await?;
    assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{}", answer.body);
    assert_eq!(answer.body["error"]["code"], "invalid_token");
    db.drop().await
}

#[tokio::test]
async fn a_mobile_sign_in_hands_over_a_refusal() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured_for_mobile(&db.pool, &fake)?;
    register_verified(&db.pool, "grace@example.com", PASSWORD).await?;

    let link = sign_in_from_mobile(&state, &fake, person("grace", "grace@example.com")).await?;
    assert_eq!(link.get("error").map(String::as_str), Some("email_in_use"));
    assert!(!link.contains_key("code"), "{link:?}");
    db.drop().await
}

#[tokio::test]
async fn a_mobile_sign_in_needs_the_apps_scheme() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let uri = format!("/api/v1/auth/oidc/{PROVIDER}/start?client=mobile");
    let answer = call_with(&state, None, Method::GET, &uri, None).await?;
    assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{}", answer.body);
    assert!(
        answer.body["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("APP_SCHEME")),
        "{}",
        answer.body
    );
    db.drop().await
}

#[tokio::test]
async fn the_apps_read_the_providers_for_their_buttons() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let uri = "/api/v1/auth/oidc/providers";
    let answer = call_with(&state(&db.pool)?, None, Method::GET, uri, None).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!([]));

    let fake = FakeIdentityProvider::new();
    let state = configured(&db.pool, &fake, false)?;
    let answer = call_with(&state, None, Method::GET, uri, None).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!([{"name": PROVIDER}]));
    db.drop().await
}
