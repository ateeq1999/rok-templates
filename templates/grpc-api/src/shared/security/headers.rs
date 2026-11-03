// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! What the browser is told on every answer: the security headers
//! ([`security_headers`]) and who may call the API from another origin
//! ([`CorsOrigins`], `CORS_ALLOWED_ORIGINS`). Framework-free, so both
//! templates share it; each template's `app/middleware.rs` applies it.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// How long `Strict-Transport-Security` tells a browser to use HTTPS only:
/// a year.
const HSTS_MAX_AGE_SECS: u64 = 365 * 24 * 60 * 60;

/// The headers every answer carries, `Strict-Transport-Security` among them
/// when the API is served over HTTPS (`https`, from `APP_URL`): answers are
/// never sniffed for another type, never framed, and send no referrer.
/// A header a handler set already is kept.
#[must_use]
pub fn security_headers(https: bool) -> Vec<(&'static str, String)> {
    let mut headers = vec![
        ("x-content-type-options", "nosniff".to_owned()),
        ("x-frame-options", "DENY".to_owned()),
        (
            "content-security-policy",
            "frame-ancestors 'none'".to_owned(),
        ),
        ("referrer-policy", "no-referrer".to_owned()),
    ];
    if https {
        headers.push((
            "strict-transport-security",
            format!("max-age={HSTS_MAX_AGE_SECS}; includeSubDomains"),
        ));
    }
    headers
}

/// The methods a cross-origin caller may use.
pub const CORS_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE"];

/// The request headers a cross-origin caller may send.
pub const CORS_HEADERS: &[&str] = &["authorization", "content-type", "x-api-key"];

/// The response headers a cross-origin caller may read.
pub const CORS_EXPOSED_HEADERS: &[&str] = &["x-request-id", "retry-after"];

/// How long a browser may cache a preflight answer.
pub const CORS_MAX_AGE: Duration = Duration::from_secs(600);

/// Who may call the API from another origin (`CORS_ALLOWED_ORIGINS`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CorsOrigins {
    /// Unset or empty: no other origin, only the API's own.
    #[default]
    SameOrigin,
    /// `*`: any origin. The API's credential is a bearer token, not a
    /// cookie, so this exposes nothing a script can't already send; still,
    /// prefer the list.
    Any,
    /// A comma-separated list of origins, such as
    /// `https://app.example.com,http://localhost:5173`.
    List(Vec<String>),
}

/// `CORS_ALLOWED_ORIGINS` names something that is not an origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAnOrigin(pub String);

impl fmt::Display for NotAnOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` is not an origin (a scheme and a host, such as https://app.example.com)",
            self.0
        )
    }
}

impl FromStr for CorsOrigins {
    type Err = NotAnOrigin;

    /// `*`, or origins separated by commas: each `http://` or `https://`, a
    /// host and maybe a port, with no path. Empty is [`CorsOrigins::SameOrigin`].
    fn from_str(value: &str) -> Result<Self, NotAnOrigin> {
        let value = value.trim();
        if value.is_empty() {
            return Ok(CorsOrigins::SameOrigin);
        }
        if value == "*" {
            return Ok(CorsOrigins::Any);
        }
        let mut origins = Vec::new();
        for origin in value.split(',').map(str::trim).filter(|o| !o.is_empty()) {
            let host = origin
                .strip_prefix("https://")
                .or_else(|| origin.strip_prefix("http://"));
            match host {
                Some(host) if is_host(host) => {
                    origins.push(origin.to_owned());
                }
                _ => return Err(NotAnOrigin(origin.to_owned())),
            }
        }
        Ok(CorsOrigins::List(origins))
    }
}

/// Whether `host` is a host and maybe a port (`app.example.com`,
/// `localhost:5173`, `[::1]:8080`): letters, digits, `.`, `-`, `:` and the
/// brackets of an IPv6 address, nothing else.
fn is_host(host: &str) -> bool {
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '[' | ']'))
}

impl fmt::Display for CorsOrigins {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CorsOrigins::SameOrigin => formatter.write_str("<same origin>"),
            CorsOrigins::Any => formatter.write_str("*"),
            CorsOrigins::List(origins) => formatter.write_str(&origins.join(",")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsts_is_sent_over_https_only() {
        let names = |https| -> Vec<&str> {
            security_headers(https)
                .into_iter()
                .map(|(name, _)| name)
                .collect()
        };
        assert!(!names(false).contains(&"strict-transport-security"));
        assert!(names(true).contains(&"strict-transport-security"));
        assert!(names(false).contains(&"x-content-type-options"));
    }

    #[test]
    fn origins_are_parsed_and_checked() {
        assert_eq!("".parse(), Ok(CorsOrigins::SameOrigin));
        assert_eq!(" * ".parse(), Ok(CorsOrigins::Any));
        assert_eq!(
            "https://app.example.com, http://localhost:5173,".parse(),
            Ok(CorsOrigins::List(vec![
                "https://app.example.com".to_owned(),
                "http://localhost:5173".to_owned(),
            ]))
        );
        for wrong in [
            "app.example.com",
            "https://app.example.com/",
            "ftp://x",
            "https://",
            "https://a b.example.com",
        ] {
            assert_eq!(
                wrong.parse::<CorsOrigins>(),
                Err(NotAnOrigin(wrong.to_owned())),
                "{wrong}"
            );
        }
    }
}
