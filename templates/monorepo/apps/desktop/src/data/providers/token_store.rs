//! Where the session's token is kept between launches: the OS keychain, or memory in tests.
//!
//! [`keep`] ties a [`TokenStore`] to the app's [`Session`]: the session starts with the
//! stored token, and every change is written back, so signing in saves the token, and
//! signing out or a 401 (the API rejected it) deletes it. The token is never logged, and no
//! error here carries it.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use rok_ui::bloc::{Observable, Subscription};
use rok_ui::http::Session;

/// Why a [`TokenStore`] failed. It never carries the token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenStoreError(String);

impl TokenStoreError {
    /// A failure explained by `reason`, which must not contain the token.
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

impl fmt::Display for TokenStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "the token store failed: {}", self.0)
    }
}

impl std::error::Error for TokenStoreError {}

/// Keeps one bearer token. A trait, so tests keep it in memory and never touch the keychain.
pub trait TokenStore: Send + Sync {
    /// The kept token, `None` when there is none.
    ///
    /// # Errors
    ///
    /// When the store cannot be read.
    fn load(&self) -> Result<Option<String>, TokenStoreError>;

    /// Keeps `token`, replacing any other.
    ///
    /// # Errors
    ///
    /// When the store cannot be written.
    fn save(&self, token: &str) -> Result<(), TokenStoreError>;

    /// Forgets the token; having none is not an error.
    ///
    /// # Errors
    ///
    /// When the store cannot be written.
    fn clear(&self) -> Result<(), TokenStoreError>;
}

/// The OS keychain (macOS Keychain, Windows Credential Manager, the Linux kernel keyring),
/// through the `keyring` crate: one entry, under this App's package name, per API address.
///
/// Nothing probes the keychain: a call is tried, and one that cannot reach it fails.
#[derive(Clone, Debug)]
pub struct KeychainTokenStore {
    account: String,
}

/// The keychain service the token is kept under: this App's package name.
const SERVICE: &str = env!("CARGO_PKG_NAME");

impl KeychainTokenStore {
    /// The token for the API at `base_url`, so each API keeps its own.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            account: base_url.into(),
        }
    }

    fn entry(&self) -> Result<keyring::Entry, TokenStoreError> {
        keyring::Entry::new(SERVICE, &self.account).map_err(|error| failure(&error))
    }
}

/// `error` as a [`TokenStoreError`]. A badly encoded entry's error holds the entry's bytes,
/// so its reason is written here instead of shown.
fn failure(error: &keyring::Error) -> TokenStoreError {
    match error {
        keyring::Error::BadEncoding(_) => TokenStoreError::new("the kept token is not UTF-8"),
        other => TokenStoreError::new(other.to_string()),
    }
}

impl TokenStore for KeychainTokenStore {
    fn load(&self) -> Result<Option<String>, TokenStoreError> {
        match self.entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(failure(&error)),
        }
    }

    fn save(&self, token: &str) -> Result<(), TokenStoreError> {
        self.entry()?
            .set_password(token)
            .map_err(|error| failure(&error))
    }

    fn clear(&self) -> Result<(), TokenStoreError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(failure(&error)),
        }
    }
}

/// A token kept in memory, for tests. Clones share it.
#[derive(Clone, Default)]
pub struct MemoryTokenStore {
    token: Arc<Mutex<Option<String>>>,
}

impl MemoryTokenStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A store already keeping `token`, as after an earlier launch.
    #[must_use]
    pub fn with_token(token: impl Into<String>) -> Self {
        let store = Self::new();
        *store.lock() = Some(token.into());
        store
    }

    /// The kept token.
    #[must_use]
    pub fn token(&self) -> Option<String> {
        self.lock().clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<String>> {
        self.token.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for MemoryTokenStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MemoryTokenStore")
            .field("token", &self.lock().as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl TokenStore for MemoryTokenStore {
    fn load(&self) -> Result<Option<String>, TokenStoreError> {
        Ok(self.token())
    }

    fn save(&self, token: &str) -> Result<(), TokenStoreError> {
        *self.lock() = Some(token.to_owned());
        Ok(())
    }

    fn clear(&self) -> Result<(), TokenStoreError> {
        *self.lock() = None;
        Ok(())
    }
}

/// A session that starts with `store`'s token and writes every change back to it, until the
/// returned [`Subscription`] is dropped.
///
/// A store that cannot be read starts signed out; one that cannot be written leaves the
/// session in memory only, so the user signs in again at the next launch. Either failure is
/// logged to stderr with its reason (never the token), so a session that is not kept shows.
pub fn keep(store: Arc<dyn TokenStore>) -> (Session, Subscription) {
    let session = match store.load() {
        Ok(Some(token)) => Session::with_token(token),
        Ok(None) => Session::new(),
        Err(error) => {
            eprintln!("[session] the kept token was not read, starting signed out: {error}");
            Session::new()
        }
    };
    let saving = session.subscribe(move |state| {
        // Best effort: the session works without the store (see above).
        let written = match &state.token {
            Some(token) => store
                .save(token)
                .map_err(|error| ("the token was not kept for the next launch", error)),
            None => store
                .clear()
                .map_err(|error| ("the kept token was not deleted", error)),
        };
        if let Err((what, error)) = written {
            eprintln!("[session] {what}: {error}");
        }
    });
    (session, saving)
}
