//! `openapi.json`, beside this crate's manifest, is the document the API
//! serves at `/api-docs/openapi.json`, written out: the contract the
//! monorepo's Clients are generated from (workspace ADR 0027). `rok gen`
//! re-exports it (`examples/openapi.rs`); this test fails when a handler or
//! a schema changed and the file was not re-exported, or when the file was
//! edited by hand.

use axum::body::{Body, to_bytes};
use axum::http::Request;
use axum_api::app::{AppState, router};
use axum_api::config::Config;
use serde_json::Value;
use tower::ServiceExt;

/// The exported document.
const ARTIFACT: &str = include_str!("../openapi.json");

/// An unused `TOTP_ENCRYPTION_KEY`: base64 of 32 bytes.
const TOTP_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

/// The document the application serves. The pool never connects: serving
/// the document needs no database.
async fn served_document() -> Result<Value, Box<dyn std::error::Error>> {
    let pool = sqlx::PgPool::connect_lazy("postgres://localhost/unused")?;
    let config = Config::from_vars(|name| match name {
        "DATABASE_URL" => Some("postgres://localhost/unused".to_owned()),
        "JWT_SECRET" => Some("unused, but at least 32 bytes long".to_owned()),
        "TOTP_ENCRYPTION_KEY" => Some(TOTP_KEY.to_owned()),
        "STORAGE_URL" => Some("memory://".to_owned()),
        _ => None,
    })?;
    let state = AppState::new(pool, &config)?;
    let request = Request::get("/api-docs/openapi.json").body(Body::empty())?;
    let response = router(state).oneshot(request).await?;
    assert!(response.status().is_success(), "{}", response.status());
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX).await?,
    )?)
}

#[tokio::test]
async fn the_exported_document_is_the_served_one() -> Result<(), Box<dyn std::error::Error>> {
    let artifact: Value = serde_json::from_str(ARTIFACT)?;
    let served = served_document().await?;
    assert!(
        artifact == served,
        "openapi.json is not the document the API serves: run `rok gen` from the \
         monorepo's root to re-export it (never edit it by hand)"
    );
    Ok(())
}

/// The exported file is the served document as `examples/openapi.rs`
/// writes it, byte for byte: pretty-printed, with a final newline.
#[test]
fn the_exported_document_is_written_as_the_example_writes_it()
-> Result<(), Box<dyn std::error::Error>> {
    let expected = format!("{}\n", axum_api::openapi::document().to_pretty_json()?);
    assert!(
        ARTIFACT.replace("\r\n", "\n") == expected,
        "openapi.json is not what `cargo run --example openapi` writes: run `rok gen` \
         from the monorepo's root to re-export it"
    );
    Ok(())
}
