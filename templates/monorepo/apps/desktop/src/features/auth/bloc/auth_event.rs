//! What happens in `auth`, in the past tense.

use std::fmt;

/// Events for `AuthBloc`. `Debug` (what rok-ui's `LogObserver` logs) never shows a password
/// or a code.
#[derive(Clone)]
pub enum AuthEvent {
    /// The sign-up form was submitted.
    SignUpSubmitted {
        /// `email`.
        email: String,
        /// `password`.
        password: String,
    },
    /// The sign-in form was submitted.
    SignInSubmitted {
        /// `email`.
        email: String,
        /// `password`.
        password: String,
    },
    /// The two-factor code was submitted, finishing the pending sign-in.
    CodeSubmitted {
        /// An authenticator code, or an unused recovery code.
        code: String,
    },
    /// The user gave up on the pending sign-in, back to the email and password.
    CodeCancelled,
    /// Sign out requested.
    SignOutRequested,
    /// The signed-in user's profile was requested (at start, with a kept token).
    ProfileRequested,
}

/// What `Debug` shows in place of a secret.
const REDACTED: &str = "<redacted>";

impl fmt::Debug for AuthEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignUpSubmitted { email, .. } => formatter
                .debug_struct("SignUpSubmitted")
                .field("email", email)
                .field("password", &format_args!("{REDACTED}"))
                .finish(),
            Self::SignInSubmitted { email, .. } => formatter
                .debug_struct("SignInSubmitted")
                .field("email", email)
                .field("password", &format_args!("{REDACTED}"))
                .finish(),
            Self::CodeSubmitted { .. } => formatter
                .debug_struct("CodeSubmitted")
                .field("code", &format_args!("{REDACTED}"))
                .finish(),
            Self::CodeCancelled => formatter.write_str("CodeCancelled"),
            Self::SignOutRequested => formatter.write_str("SignOutRequested"),
            Self::ProfileRequested => formatter.write_str("ProfileRequested"),
        }
    }
}
