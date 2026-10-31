//! The API through its generated Rust client (`clients/rust`, workspace ADR
//! 0027): the flows the other suites drive through the router, sent here
//! over a real socket by the client the Rust callers use. The contract,
//! the client and the server are proved to agree: a drift between them
//! fails to compile here, or fails these flows.

#[path = "../support/mod.rs"]
mod support;

mod accounts;
mod admin;
mod files;
mod keys;
mod two_factor;

use std::sync::Arc;

use api_client::{Client, CredentialsDto, LoginResponse, VerifyEmailDto};
use axum_api::app::serve_as;
use axum_api::config::AppRole;
use axum_api::shared::mail::MemoryMailer;
use support::{Database, TestResult, mailed, mailed_token, state_with};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// What a helper returns.
type Error = Box<dyn std::error::Error>;

/// The API on a real socket over a fresh schema, its `/auth` rate limit
/// off, and a client of it with no session.
struct Api {
    client: Client,
    db: Database,
    stop: oneshot::Sender<()>,
    server: JoinHandle<std::io::Result<()>>,
}

impl Api {
    /// The API, or `None` (and a note) when `DATABASE_URL` is not set. It
    /// serves the `web` role: the tests deliver the mail queue themselves
    /// ([`mailed`]).
    async fn start() -> Result<Option<Api>, Error> {
        let Some(db) = Database::fresh().await? else {
            return Ok(None);
        };
        let state = state_with(&db.pool, &[("AUTH_RATE_LIMIT", "0")])?;
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let (stop, stopped) = oneshot::channel::<()>();
        let mailer = Arc::new(MemoryMailer::default());
        let server = tokio::spawn(serve_as(AppRole::Web, listener, state, mailer, async {
            let _ = stopped.await;
        }));
        Ok(Some(Api {
            client: Client::new(format!("http://{address}")),
            db,
            stop,
            server,
        }))
    }

    /// Stops the server and drops the schema.
    async fn stop(self) -> TestResult {
        let _ = self.stop.send(());
        self.server.await??;
        self.db.drop().await
    }

    /// Signs `email` up with `password`, verifies it with the mailed link
    /// and signs in, all through the client: the session's token.
    async fn register_verified(&self, email: &str, password: &str) -> Result<String, Error> {
        let credentials = credentials(email, password);
        self.client.auth().register(&credentials).await?;
        let token = mailed(&self.db.pool)
            .await?
            .iter()
            .rev()
            .find(|mail| mail.to == email)
            .and_then(mailed_token)
            .ok_or("no verification mail")?;
        self.client
            .auth()
            .verify_email(&VerifyEmailDto { token })
            .await?;
        session(self.client.auth().login(&credentials).await?)
    }
}

/// The body `POST /auth/register` and `/auth/login` take.
fn credentials(email: &str, password: &str) -> CredentialsDto {
    CredentialsDto {
        email: email.to_owned(),
        password: password.to_owned(),
    }
}

/// The session's token of a sign-in that needed no second factor.
fn session(answer: LoginResponse) -> Result<String, Error> {
    match answer {
        LoginResponse::TokenResponse(session) => Ok(session.access_token),
        LoginResponse::PendingToken(pending) => {
            Err(format!("a second factor is needed: {pending:?}").into())
        }
    }
}
