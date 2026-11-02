//! What happens in `auth`, in the past tense.

/// Events for `AuthBloc`.
#[derive(Clone, Debug)]
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
