// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Reading settings: [`Vars`], the readers `config.rs` reads each variable
//! with, and the errors they report ([`ConfigError`], every one of them
//! at once in [`ConfigErrors`]). It reads no environment itself: `Vars`
//! looks variables up through the function `config.rs` gives it, which is
//! what lets a test pass its own. Framework-free, so both templates share
//! it.

use std::cell::RefCell;
use std::fmt;
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// A setting missing from, or invalid in, the environment.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required variable is not set.
    #[error("environment variable `{0}` is not set")]
    Missing(&'static str),
    /// A key is too short to resist guessing.
    #[error(
        "environment variable `{name}` is {len} bytes long; it needs at least {min} (try `openssl rand -base64 48`)"
    )]
    TooShort {
        /// The variable.
        name: &'static str,
        /// Its length, in bytes.
        len: usize,
        /// The least it may be.
        min: usize,
    },
    /// A key is not base64 of the bytes it needs.
    #[error(
        "environment variable `{name}` is not {bytes} bytes in base64 (try `openssl rand -base64 {bytes}`)"
    )]
    NotAKey {
        /// The variable.
        name: &'static str,
        /// The bytes it needs.
        bytes: usize,
    },
    /// A variable is not a whole number.
    #[error("environment variable `{name}` is `{value}`, which is not a whole number")]
    NotANumber {
        /// The variable.
        name: &'static str,
        /// Its value.
        value: String,
    },
    /// A variable is none of the values it takes.
    #[error("environment variable `{name}` is `{value}`, which is not one of {expected}")]
    NotOneOf {
        /// The variable.
        name: &'static str,
        /// Its value.
        value: String,
        /// The values it takes.
        expected: &'static str,
    },
}

/// Every setting missing from, or invalid in, the environment, in the
/// order they are read: one start-up names all there is to fix. One error
/// reads as itself; several as a count and a line each.
#[derive(Debug)]
pub struct ConfigErrors(pub Vec<ConfigError>);

impl fmt::Display for ConfigErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.as_slice() {
            [one] => write!(formatter, "{one}"),
            all => {
                write!(formatter, "{} settings are wrong:", all.len())?;
                for error in all {
                    write!(formatter, "\n  - {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigErrors {}

/// The variables, read through a lookup, and every failure met reading
/// them. A reader that fails records it and answers a stand-in, so the
/// reading goes on to the next variable.
pub struct Vars<'a> {
    var: &'a dyn Fn(&str) -> Option<String>,
    errors: RefCell<Vec<ConfigError>>,
}

impl<'a> Vars<'a> {
    /// The readers, over `var`, which gives a variable's value when it is
    /// set.
    pub fn new(var: &'a dyn Fn(&str) -> Option<String>) -> Self {
        Vars {
            var,
            errors: RefCell::new(Vec::new()),
        }
    }

    /// `value`, read whole, unless a reader failed: then every failure.
    pub fn finish<T>(self, value: T) -> Result<T, ConfigErrors> {
        let errors = self.errors.into_inner();
        if errors.is_empty() {
            Ok(value)
        } else {
            Err(ConfigErrors(errors))
        }
    }

    /// Records `error`.
    pub fn fail(&self, error: ConfigError) {
        self.errors.borrow_mut().push(error);
    }

    /// A variable that must be set; empty when it isn't.
    pub fn required(&self, name: &'static str) -> String {
        (self.var)(name).unwrap_or_else(|| {
            self.fail(ConfigError::Missing(name));
            String::new()
        })
    }

    /// A key that must be set, at least `min` bytes long.
    pub fn secret(&self, name: &'static str, min: usize) -> String {
        let Some(value) = (self.var)(name) else {
            self.fail(ConfigError::Missing(name));
            return String::new();
        };
        if value.len() < min {
            self.fail(ConfigError::TooShort {
                name,
                len: value.len(),
                min,
            });
        }
        value
    }

    /// A key that may be unset, but when set is at least `min` bytes long.
    pub fn optional_secret(&self, name: &'static str, min: usize) -> Option<String> {
        let value = (self.var)(name).filter(|value| !value.is_empty())?;
        if value.len() < min {
            self.fail(ConfigError::TooShort {
                name,
                len: value.len(),
                min,
            });
        }
        Some(value)
    }

    /// A key that must be set: base64 (standard, padded) of exactly
    /// `N` bytes; zeros when it isn't.
    pub fn key<const N: usize>(&self, name: &'static str) -> [u8; N] {
        let Some(value) = (self.var)(name) else {
            self.fail(ConfigError::Missing(name));
            return [0; N];
        };
        STANDARD
            .decode(value.trim())
            .ok()
            .and_then(|bytes| <[u8; N]>::try_from(bytes).ok())
            .unwrap_or_else(|| {
                self.fail(ConfigError::NotAKey { name, bytes: N });
                [0; N]
            })
    }

    /// A whole number, when the variable is set and is one.
    pub fn number<T: FromStr>(&self, name: &'static str) -> Option<T> {
        let value = (self.var)(name)?;
        if let Ok(number) = value.parse() {
            Some(number)
        } else {
            self.fail(ConfigError::NotANumber { name, value });
            None
        }
    }

    /// A boolean, when the variable is set and is one.
    pub fn flag(&self, name: &'static str) -> Option<bool> {
        match (self.var)(name).as_deref() {
            None => None,
            Some("true" | "1") => Some(true),
            Some("false" | "0") => Some(false),
            Some(other) => {
                self.fail(ConfigError::NotOneOf {
                    name,
                    value: other.to_owned(),
                    expected: "true, false",
                });
                None
            }
        }
    }
}
