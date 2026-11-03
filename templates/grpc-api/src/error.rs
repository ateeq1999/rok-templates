// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`AppError`]: every error a service returns. The generated
//! `shared/grpc.rs` answers each with its gRPC status, by its
//! [`code`](AppError::code): the axum template's variants and codes, so
//! the code the two templates share returns the same errors. A new error
//! case is a variant here, never an error type in a feature; one
//! `shared/grpc.rs` doesn't name is `INTERNAL`.

use rok_data::ValidationErrors;
use sqlx::error::ErrorKind;

/// An error a service returns. The message of each variant but `Internal`
/// is sent to the caller.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// `bad_request` (`INVALID_ARGUMENT`): a malformed request.
    #[error("{0}")]
    BadRequest(String),
    /// `invalid_token`: a mailed link's token that is unknown, expired or
    /// already used.
    #[error("{0}")]
    InvalidToken(String),
    /// `wrong_password`: the current password a flow asks for again
    /// doesn't match.
    #[error("{0}")]
    WrongPassword(String),
    /// `breached_password`: a new password found in a known breach
    /// (`BREACH_CHECK`).
    #[error("{0}")]
    BreachedPassword(String),
    /// `unauthorized` (`UNAUTHENTICATED`): no valid credentials.
    #[error("{0}")]
    Unauthorized(String),
    /// `forbidden` (`PERMISSION_DENIED`): not allowed.
    #[error("{0}")]
    Forbidden(String),
    /// `not_found` (`NOT_FOUND`): no such row, or another caller's.
    #[error("{0}")]
    NotFound(String),
    /// `request_timeout`: the call took longer than allowed.
    #[error("{0}")]
    RequestTimeout(String),
    /// `conflict` (`ALREADY_EXISTS`): a unique or foreign-key violation,
    /// naming the constraint.
    #[error("{0}")]
    Conflict(String),
    /// `payload_too_large` (`RESOURCE_EXHAUSTED`): a message over the limit.
    #[error("{0}")]
    PayloadTooLarge(String),
    /// `unsupported_media_type`: an upload whose type, detected from its
    /// bytes, is unknown or not allowed.
    #[error("{0}")]
    UnsupportedMediaType(String),
    /// `validation_failed` (`INVALID_ARGUMENT`, the field errors in the
    /// details): broken rules, with their details by field path when there
    /// are any; also a check violation.
    #[error("{message}")]
    Validation {
        /// What is wrong.
        message: String,
        /// The broken rules by field.
        details: Option<ValidationErrors>,
    },
    /// `precondition_required` (`FAILED_PRECONDITION`): the version a write
    /// on a versioned table needs is missing.
    #[error("{0}")]
    PreconditionRequired(String),
    /// `version_conflict` (`ABORTED`): a stale version.
    #[error("{0}")]
    VersionConflict(String),
    /// `too_many_requests`: over the rate limit.
    #[error("too many requests; retry in {retry_after_secs}s")]
    TooManyRequests {
        /// Seconds until the caller may try again.
        retry_after_secs: u64,
    },
    /// `internal_error` (`INTERNAL`): anything else. Logged with `tracing`
    /// when answered; the caller gets a generic message.
    #[error("internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl AppError {
    /// The `payload_too_large` of a `what` (such as `file`) over `limit`
    /// bytes.
    #[must_use]
    pub fn too_large(what: &str, limit: usize) -> Self {
        AppError::PayloadTooLarge(format!("the {what} is larger than {limit} bytes"))
    }

    /// The error's `snake_case` code, the one the axum template answers
    /// with.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "bad_request",
            AppError::InvalidToken(_) => "invalid_token",
            AppError::WrongPassword(_) => "wrong_password",
            AppError::BreachedPassword(_) => "breached_password",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::Forbidden(_) => "forbidden",
            AppError::NotFound(_) => "not_found",
            AppError::RequestTimeout(_) => "request_timeout",
            AppError::Conflict(_) => "conflict",
            AppError::PayloadTooLarge(_) => "payload_too_large",
            AppError::UnsupportedMediaType(_) => "unsupported_media_type",
            AppError::Validation { .. } => "validation_failed",
            AppError::PreconditionRequired(_) => "precondition_required",
            AppError::VersionConflict(_) => "version_conflict",
            AppError::TooManyRequests { .. } => "too_many_requests",
            AppError::Internal(_) => "internal_error",
        }
    }
}

/// Broken validation rules, as `input.validate(&db)` reports them.
impl From<ValidationErrors> for AppError {
    fn from(details: ValidationErrors) -> Self {
        AppError::Validation {
            message: "the request failed validation".to_owned(),
            details: Some(details),
        }
    }
}

/// A unique or foreign-key violation is a `Conflict` naming the constraint,
/// a check violation is a `Validation`, `RowNotFound` is a `NotFound`, and
/// anything else is `Internal`.
impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = error {
            return AppError::NotFound("no such row".to_owned());
        }
        if let sqlx::Error::Database(database) = &error {
            let constraint = database.constraint().unwrap_or("unnamed");
            match database.kind() {
                ErrorKind::UniqueViolation => {
                    return AppError::Conflict(format!(
                        "the value is already taken (constraint `{constraint}`)"
                    ));
                }
                ErrorKind::ForeignKeyViolation => {
                    return AppError::Conflict(format!(
                        "the change breaks a reference (constraint `{constraint}`)"
                    ));
                }
                ErrorKind::CheckViolation => {
                    return AppError::Validation {
                        message: format!("the value breaks the check `{constraint}`"),
                        details: None,
                    };
                }
                _ => {}
            }
        }
        AppError::Internal(Box::new(error))
    }
}

/// The generated data layer's error: rows map into `NotFound`, concurrent
/// writes into `VersionConflict`, and everything else reuses the
/// `sqlx::Error` mapping where a driver error wrapped it.
impl From<rok_data::Error> for AppError {
    fn from(error: rok_data::Error) -> Self {
        match error {
            rok_data::Error::NotFound { .. } => AppError::NotFound("no such row".to_owned()),
            rok_data::Error::Conflict { .. } => {
                AppError::VersionConflict("the record changed before the write".to_owned())
            }
            rok_data::Error::Validation(details) => AppError::from(details),
            rok_data::Error::Database(sqlx_error) => AppError::from(sqlx_error),
            other => AppError::Internal(Box::new(other)),
        }
    }
}
