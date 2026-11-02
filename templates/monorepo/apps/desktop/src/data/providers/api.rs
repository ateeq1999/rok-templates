//! `Api`: `apps/api`, through its generated client (`clients/rust`, the `api_client` crate).
//!
//! The client is the data source: every path, verb and wire name lives in it, and `rok gen`
//! regenerates it from the API's contract. This file wires it to the app's [`Session`] and
//! turns its errors into the [`ApiError`] blocs and forms read.

use api_client::Client;
use rok_ui::http::{ApiError, FieldDetails, FieldIssue, Session};

/// The API's client, signed in as the app's [`Session`] says. Clones share both.
#[derive(Clone, Debug)]
pub struct Api {
    client: Client,
    session: Session,
}

impl Api {
    /// The API at `base_url` (such as `http://127.0.0.1:8080`), signed in if `session` is. A
    /// 401 to a call that sent the session's token expires `session`, which sends the user
    /// back to sign in.
    #[must_use]
    pub fn new(base_url: impl Into<String>, session: Session) -> Self {
        let client = Client::new(base_url);
        client.set_token(session.token());
        let rejected = client.clone();
        let expired = session.clone();
        let client = client.on_unauthorized(move |token| {
            // A call sent with an older token must not sign out a newer sign-in.
            if expired.token().as_deref() == Some(token) {
                rejected.set_token(None);
                expired.expire();
            }
        });
        Self { client, session }
    }

    /// The generated client: `api.client().users().list(&query)`.
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Signs in with `token`: the client sends it, and the session holds it.
    pub fn sign_in(&self, token: String) {
        self.client.set_token(Some(token.clone()));
        self.session.set_token(token);
    }

    /// Signs out: the client stops sending the token, and the session forgets it.
    pub fn sign_out(&self) {
        self.client.set_token(None);
        self.session.clear();
    }
}

/// The client's `error` as the app's [`ApiError`], its field issues (a 422's broken rules)
/// kept by field, so forms show them on their fields.
#[must_use]
pub fn app_error(error: api_client::ApiError) -> ApiError {
    let details: Option<FieldDetails> = error.details.map(|details| {
        details
            .into_iter()
            .map(|(field, issues)| {
                let issues = issues
                    .into_iter()
                    .map(|issue| FieldIssue {
                        code: issue.code,
                        message: issue.message,
                        params: issue.params.unwrap_or_default().into_iter().collect(),
                    })
                    .collect();
                (field, issues)
            })
            .collect()
    });
    let app = ApiError::new(error.status, error.code, error.message);
    match details {
        Some(details) => app.with_details(details),
        None => app,
    }
}
