//! What happens in `auth`, in the past tense.

/// Events for `AuthBloc`.
#[derive(Clone, Debug)]
pub enum AuthEvent {
    /// Sign in submitted.
    SignInSubmitted {
        /// `email`.
        email: String,
        /// `password`.
        password: String,
    },
    /// Sign out requested.
    SignOutRequested,
    /// Profile requested.
    ProfileRequested,
}
