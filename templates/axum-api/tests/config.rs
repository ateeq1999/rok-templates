//! The settings `Config::from_vars` reads: defaults, values, every wrong
//! one reported at once, and the summary the server logs, secrets left out.

use std::collections::HashMap;
use std::time::Duration;

use axum_api::config::{Config, ConfigErrors, MailTransport, log_settings_from};
use axum_api::shared::storage::StorageUrl;
use axum_api::shared::telemetry::LogFormat;

/// The settings read from `vars` alone.
fn read(vars: &[(&str, &str)]) -> Result<Config, ConfigErrors> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    Config::from_vars(|name| vars.get(name).cloned())
}

/// What reading `vars` reports, if it fails.
fn refusal(vars: &[(&str, &str)]) -> Option<String> {
    read(vars).err().map(|err| err.to_string())
}

/// Base64 of the 32 bytes `01234567890123456789012345678901`.
const TOTP_KEY: &str = "MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDE=";

const REQUIRED: &[(&str, &str)] = &[
    ("DATABASE_URL", "postgres://db"),
    ("JWT_SECRET", "a test secret of at least 32 bytes"),
    ("TOTP_ENCRYPTION_KEY", TOTP_KEY),
];

/// `REQUIRED` and `extra`.
fn with(extra: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    let mut vars = REQUIRED.to_vec();
    vars.extend_from_slice(extra);
    vars
}

#[test]
fn the_auth_settings_default_for_development() -> Result<(), ConfigErrors> {
    let config = read(REQUIRED)?;
    assert_eq!(config.jwt_ttl, Duration::from_secs(3600));
    assert!(config.require_verified_email);
    assert_eq!(config.app_url, "http://localhost:8080");
    assert_eq!(config.mail_transport, MailTransport::Log);
    assert_eq!(config.smtp_url, None);
    assert!(config.mail_from.ends_with(" API <no-reply@localhost>"));
    assert!(!config.breach_check);
    assert!(!config.trust_proxy);
    assert_eq!(config.auth_rate_limit, 20);
    Ok(())
}

#[test]
fn the_auth_settings_are_read() -> Result<(), ConfigErrors> {
    let config = read(&with(&[
        ("JWT_TTL", "900"),
        ("REQUIRE_VERIFIED_EMAIL", "false"),
        ("APP_URL", "https://app.example.com"),
        ("MAIL_TRANSPORT", "smtp"),
        ("SMTP_URL", "smtps://u:p@mail.example.com"),
        ("MAIL_FROM", "App <hi@example.com>"),
        ("BREACH_CHECK", "1"),
        ("TRUST_PROXY", "true"),
        ("AUTH_RATE_LIMIT", "5"),
    ]))?;
    assert_eq!(config.jwt_ttl, Duration::from_secs(900));
    assert!(!config.require_verified_email);
    assert_eq!(config.app_url, "https://app.example.com");
    assert_eq!(config.mail_transport, MailTransport::Smtp);
    assert_eq!(
        config.smtp_url.as_deref(),
        Some("smtps://u:p@mail.example.com")
    );
    assert_eq!(config.mail_from, "App <hi@example.com>");
    assert!(config.breach_check);
    assert!(config.trust_proxy);
    assert_eq!(config.auth_rate_limit, 5);
    Ok(())
}

#[test]
fn a_jwt_secret_under_32_bytes_is_refused() {
    assert_eq!(
        refusal(&[
            ("DATABASE_URL", "postgres://db"),
            ("JWT_SECRET", "change-me"),
            ("TOTP_ENCRYPTION_KEY", TOTP_KEY),
        ])
        .as_deref(),
        Some(
            "environment variable `JWT_SECRET` is 9 bytes long; it needs at least 32 (try `openssl rand -base64 48`)"
        )
    );
    let secret = "x".repeat(32);
    assert!(
        read(&[
            ("DATABASE_URL", "postgres://db"),
            ("JWT_SECRET", &secret),
            ("TOTP_ENCRYPTION_KEY", TOTP_KEY),
        ])
        .is_ok()
    );
}

#[test]
fn smtp_needs_its_url_and_values_are_checked() {
    assert_eq!(
        refusal(&with(&[("MAIL_TRANSPORT", "smtp")])).as_deref(),
        Some("environment variable `SMTP_URL` is not set")
    );
    assert_eq!(
        refusal(&with(&[("TRUST_PROXY", "yes")])).as_deref(),
        Some("environment variable `TRUST_PROXY` is `yes`, which is not one of true, false")
    );
    assert_eq!(
        refusal(&with(&[("MAIL_TRANSPORT", "carrier-pigeon")])).as_deref(),
        Some(
            "environment variable `MAIL_TRANSPORT` is `carrier-pigeon`, which is not one of log, smtp"
        )
    );
}

/// A start-up with several wrong settings names every one, in the order
/// they are read, so one restart fixes them all.
#[test]
fn every_wrong_setting_is_reported_at_once() {
    assert_eq!(
        refusal(&[
            ("DB_MAX_CONNECTIONS", "many"),
            ("JWT_SECRET", "short"),
            ("TRUST_PROXY", "yes"),
        ])
        .as_deref(),
        Some(
            "5 settings are wrong:\n  \
             - environment variable `DB_MAX_CONNECTIONS` is `many`, which is not a whole number\n  \
             - environment variable `DATABASE_URL` is not set\n  \
             - environment variable `JWT_SECRET` is 5 bytes long; it needs at least 32 (try `openssl rand -base64 48`)\n  \
             - environment variable `TOTP_ENCRYPTION_KEY` is not set\n  \
             - environment variable `TRUST_PROXY` is `yes`, which is not one of true, false"
        )
    );
}

#[test]
fn the_media_settings_default_and_are_read() -> Result<(), ConfigErrors> {
    let config = read(REQUIRED)?;
    assert_eq!(config.storage_url, StorageUrl::default_for_development());
    assert_eq!(config.media_max_upload_bytes, 10 * 1024 * 1024);
    assert_eq!(
        config.media_allowed_types,
        [
            "image/jpeg",
            "image/png",
            "image/webp",
            "image/gif",
            "application/pdf"
        ]
    );

    let config = read(&with(&[
        ("STORAGE_URL", "s3://uploads"),
        ("MEDIA_MAX_UPLOAD_BYTES", "1024"),
        ("MEDIA_ALLOWED_TYPES", " image/png , application/pdf,"),
    ]))?;
    assert_eq!(config.storage_url, StorageUrl::S3("uploads".to_owned()));
    assert_eq!(config.media_max_upload_bytes, 1024);
    assert_eq!(config.media_allowed_types, ["image/png", "application/pdf"]);

    assert_eq!(
        refusal(&with(&[("STORAGE_URL", "ftp://files")])).as_deref(),
        Some(
            "environment variable `STORAGE_URL` is `ftp://files`, which is not one of file://<dir>, s3://<bucket>, memory://"
        )
    );
    Ok(())
}

#[test]
fn the_totp_key_is_32_bytes_in_base64() -> Result<(), ConfigErrors> {
    assert_eq!(
        &read(REQUIRED)?.totp_encryption_key,
        b"01234567890123456789012345678901"
    );
    let not_a_key = Some(
        "environment variable `TOTP_ENCRYPTION_KEY` is not 32 bytes in base64 (try `openssl rand -base64 32`)",
    );
    for key in [
        "",
        "not base64!",
        "c2hvcnQ=",
        "MDEyMzQ1Njc4OTAxMjM0NTY3ODkwMTIzNDU2Nzg5MDEy",
    ] {
        let mut vars = REQUIRED.to_vec();
        vars.retain(|(name, _)| *name != "TOTP_ENCRYPTION_KEY");
        vars.push(("TOTP_ENCRYPTION_KEY", key));
        assert_eq!(refusal(&vars).as_deref(), not_a_key, "{key}");
    }
    let mut vars = REQUIRED.to_vec();
    vars.retain(|(name, _)| *name != "TOTP_ENCRYPTION_KEY");
    assert_eq!(
        refusal(&vars).as_deref(),
        Some("environment variable `TOTP_ENCRYPTION_KEY` is not set")
    );
    Ok(())
}

/// The summary names every setting, but no secret: keys by their length,
/// URLs without their password.
#[test]
fn the_summary_leaves_the_secrets_out() -> Result<(), ConfigErrors> {
    let config = read(&[
        ("DATABASE_URL", "postgres://app:db-password@db:5432/app"),
        ("JWT_SECRET", "a test secret of at least 32 bytes"),
        ("TOTP_ENCRYPTION_KEY", TOTP_KEY),
        ("MAIL_TRANSPORT", "smtp"),
        (
            "SMTP_URL",
            "smtps://mailer:smtp-password@mail.example.com:465",
        ),
    ])?;
    let summary = config.summary();
    for secret in ["db-password", "a test secret", TOTP_KEY, "smtp-password"] {
        assert!(!summary.contains(secret), "{secret} in:\n{summary}");
    }
    for line in [
        "DATABASE_URL=postgres://app:<redacted>@db:5432/app",
        "JWT_SECRET=<34 bytes>",
        "TOTP_ENCRYPTION_KEY=<32 bytes>",
        "SMTP_URL=smtps://mailer:<redacted>@mail.example.com:465",
        "STORAGE_PUBLIC_ENDPOINT=<unset>",
        "DB_MAX_CONNECTIONS=20",
    ] {
        assert!(
            summary.lines().any(|l| l == line),
            "no {line} in:\n{summary}"
        );
    }
    // A URL without a password is shown as it is.
    let config = read(REQUIRED)?;
    assert!(
        config
            .summary()
            .lines()
            .any(|l| l == "DATABASE_URL=postgres://db")
    );
    Ok(())
}

/// Logs are pretty and spans stay in the process by default; `LOG_FORMAT`
/// and `OTEL_EXPORTER_OTLP_ENDPOINT` switch them, and a wrong format is
/// refused.
#[test]
fn the_log_settings_default_to_pretty_lines_and_no_export() -> Result<(), ConfigErrors> {
    let read = |vars: &'static [(&'static str, &'static str)]| {
        log_settings_from(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    };
    let settings = read(&[])?;
    assert_eq!(settings.format, LogFormat::Pretty);
    assert_eq!(settings.filter, "info");
    assert!(!settings.export_traces);
    assert_eq!(settings.service_name, None);
    let settings = read(&[
        ("LOG_FORMAT", "json"),
        ("RUST_LOG", "debug,sqlx=warn"),
        ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318"),
        ("OTEL_SERVICE_NAME", "shop-api"),
    ])?;
    assert_eq!(settings.format, LogFormat::Json);
    assert_eq!(settings.filter, "debug,sqlx=warn");
    assert!(settings.export_traces);
    assert_eq!(settings.service_name.as_deref(), Some("shop-api"));
    // An empty endpoint is no endpoint.
    assert!(!read(&[("OTEL_EXPORTER_OTLP_ENDPOINT", " ")])?.export_traces);
    assert_eq!(
        read(&[("LOG_FORMAT", "xml")])
            .err()
            .map(|err| err.to_string())
            .as_deref(),
        Some("environment variable `LOG_FORMAT` is `xml`, which is not one of pretty, json")
    );
    Ok(())
}
