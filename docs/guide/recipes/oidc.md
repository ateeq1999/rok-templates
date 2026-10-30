# Recipe: sign-in with OIDC

Sign-in with Google, Microsoft Entra, Okta, Keycloak or any other OpenID
Connect provider, beside the templates' passwords
([Accounts and sign-in](../auth.md)). The provider proves who the user is.
The API then issues its own session, the same bearer token a password
sign-in gets, so nothing after sign-in changes: roles, `token_version`,
API keys and the audit log all work as before.

## 1. The crate, the settings, the table

```toml
openidconnect = { version = "4", default-features = false, features = ["reqwest", "rustls-tls"] }
```

Settings: `OIDC_ISSUER` (such as `https://accounts.google.com`),
`OIDC_CLIENT_ID` and `OIDC_CLIENT_SECRET` (a secret: left out of the
summary), and the redirect URL you register with the provider,
`{APP_URL}/api/v1/auth/oidc/callback`.

An account is linked to the provider's `(issuer, subject)`, never to an
email address. A user can change their email at the provider, and another
user can later be given the old one.

```sql
CREATE TABLE user_identities (
    issuer     TEXT NOT NULL,
    subject    TEXT NOT NULL,
    user_id    UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (issuer, subject)
);
```

`users.password_hash` is nullable already, so an account made through the
provider has no password (`has_password` is false), and the password flows
ask it to set one first.

## 2. The flow

Use the authorization code flow with PKCE. The ID token comes from the
provider's token endpoint, never from the browser.

1. **`GET /auth/oidc/start`**: discover the provider (cache the metadata
   at start-up), create the PKCE verifier, a `state` and a `nonce`, and
   store all three for ten minutes keyed by `state`. Store them in a table,
   or in `auth_tokens` with a new purpose. Then redirect to the provider's
   authorization URL.
2. **`GET /auth/oidc/callback?code&state`**: take the stored entry for
   `state` and delete it, so it is single use. A missing entry is `400`.
   Exchange the code with the verifier, then verify the ID token's
   signature, issuer, audience, expiry and `nonce`. `openidconnect` does
   all of this when you pass the nonce to `id_token_verifier`.
3. **Find the account**:
   - The `(issuer, sub)` is linked: that account.
   - Not linked, and no account has the token's email: create one. Its
     email is verified only if the token says `email_verified: true`.
   - Not linked, but an account has that email: **do not link it
     silently**. Anyone who can make the provider claim that email would
     take over the account. Answer with "sign in with your password, then
     link the provider from your account settings", and add a link route
     (`POST /account/identities`, signed in, the same flow) that inserts
     the identity for the signed-in user.
4. **The checks a password sign-in makes**: a disabled account is refused
   and `REQUIRE_VERIFIED_EMAIL` applies. Whether two-factor still applies
   is your decision; the provider may enforce its own. Record the outcome
   with `metrics::record_login`, like `login` does.
5. **Hand over the session**: the callback runs in the browser, but the
   API answers bearer tokens in JSON. Do not put the token in the redirect
   URL, where it lands in history and logs. Issue a single-use code that
   lasts one minute (`auth_tokens` again), redirect to
   `{APP_URL}/signed-in?code=…`, and let the front end trade it at
   `POST /auth/oidc/session` for the same `TokenResponse` that `login`
   answers.

<!-- not compiled: a recipe; features/auth/service.rs beside `login` -->
```rust
pub async fn sign_in_with_identity(
    &self,
    issuer: &str,
    claims: &CoreIdTokenClaims,
) -> Result<TokenResponse, AppError> {
    let subject = claims.subject().as_str();
    let account = match queries::find_by_identity(&self.pool, issuer, subject).await? {
        Some(account) => account,
        None => self.create_from_identity(issuer, claims).await?, // refuses an email in use
    };
    if !account.is_active {
        metrics::record_login(LoginOutcome::Forbidden);
        return Err(AppError::Forbidden("the account is disabled".to_owned()));
    }
    metrics::record_login(LoginOutcome::Success);
    self.session(account.id, account.role, account.token_version)
}
```

## 3. Test it

Use a real provider in a container: Keycloak or Dex in `compose.yaml`, with
a realm and a user. Add an integration test that runs the whole redirect
flow with `reqwest` and cookies disabled. Then test the refusals: a reused
`state`, a wrong `nonce`, and the third case of step 3 (the email is
already in use).
