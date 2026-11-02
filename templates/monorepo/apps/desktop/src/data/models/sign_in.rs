//! What signing in answers: the user, or a request for a two-factor code.

use std::fmt;

use super::user::User;

/// What the right email and password answer.
#[derive(Clone, Debug, PartialEq)]
pub enum SignIn {
    /// Signed in as this user.
    SignedIn(User),
    /// The account has two-factor on: send a code with this sign-in to finish it.
    CodeRequired(PendingSignIn),
}

/// A sign-in waiting for its two-factor code: `apps/api`'s pending token, valid 5 minutes
/// and once. `Debug` never shows it.
#[derive(Clone, PartialEq, Eq)]
pub struct PendingSignIn(String);

impl PendingSignIn {
    /// The sign-in `apps/api` answered with `pending_token`.
    #[must_use]
    pub fn new(pending_token: impl Into<String>) -> Self {
        Self(pending_token.into())
    }

    /// The pending token, to send with the code.
    #[must_use]
    pub fn token(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PendingSignIn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PendingSignIn(<redacted>)")
    }
}
