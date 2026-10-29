//! Direct uploads (engine ADR 0024): an upload intent, its bytes put to a
//! presigned URL, the completion that checks them, and the download URL.
//! The tests' store is `memory://`, whose URLs the API signs and serves at
//! `/api/v1/storage/{token}`, as a client would use an S3 URL.

mod support;

use actix_web::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Database, Raw, TestResult, send, state, user};

const MEDIA: &str = "/api/v1/media";

/// A PNG's first bytes: enough for type detection.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0";

/// A PDF's first bytes.
const PDF: &[u8] = b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\n";

/// `POST /media/uploads` of `body`, as `token`.
async fn start(
    state: &actix_api::app::AppState,
    token: &str,
    body: &Value,
) -> Result<Raw, Box<dyn std::error::Error>> {
    let headers = [("content-type", "application/json")];
    send(
        state,
        Some(token),
        Method::POST,
        &format!("{MEDIA}/uploads"),
        &headers,
        serde_json::to_vec(body)?,
    )
    .await
}

/// Puts `bytes` to a ticket's URL, with its headers and no credentials.
async fn put(
    state: &actix_api::app::AppState,
    ticket: &Value,
    bytes: &[u8],
) -> Result<Raw, Box<dyn std::error::Error>> {
    let url = ticket["upload_url"].as_str().ok_or("no upload_url")?;
    let content_type = ticket["headers"]["Content-Type"]
        .as_str()
        .ok_or("no Content-Type")?;
    send(
        state,
        None,
        Method::PUT,
        url,
        &[("content-type", content_type)],
        bytes.to_vec(),
    )
    .await
}

/// `method uri` with no body, as `token`.
async fn fetch(
    state: &actix_api::app::AppState,
    token: Option<&str>,
    method: Method,
    uri: &str,
) -> Result<Raw, Box<dyn std::error::Error>> {
    send(state, token, method, uri, &[], Vec::new()).await
}

#[actix_web::test]
async fn a_direct_upload_is_put_completed_listed_and_downloaded() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let token = user(&db.pool).await?;

    let answer = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": PNG.len(), "filename": "dot.png"}),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::CREATED, "{:?}", answer.json());
    let ticket = answer.json()?;
    assert_eq!(ticket["method"], "PUT");
    let id = ticket["media_id"].as_str().ok_or("no media_id")?.to_owned();
    let url = ticket["upload_url"].as_str().ok_or("no upload_url")?;
    assert!(url.starts_with("/api/v1/storage/"), "{url}");

    // Until it is completed, the file is no one's to read.
    let listed = fetch(&state, Some(&token), Method::GET, MEDIA).await?;
    assert_eq!(listed.json()?["total"], 0);
    let early = fetch(
        &state,
        Some(&token),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(
        early.status,
        StatusCode::CONFLICT,
        "the bytes aren't up yet"
    );

    assert_eq!(
        put(&state, &ticket, PNG).await?.status,
        StatusCode::NO_CONTENT
    );
    let completed = fetch(
        &state,
        Some(&token),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(completed.status, StatusCode::OK, "{:?}", completed.json());
    let media = completed.json()?;
    assert_eq!(media["id"], id.as_str());
    assert_eq!(media["content_type"], "image/png");
    assert_eq!(media["original_filename"], "dot.png");

    let listed = fetch(&state, Some(&token), Method::GET, MEDIA).await?;
    assert_eq!(listed.json()?["total"], 1);
    let content = fetch(
        &state,
        Some(&token),
        Method::GET,
        &format!("{MEDIA}/{id}/content"),
    )
    .await?;
    assert_eq!(content.bytes, PNG, "/content still serves the bytes");

    // The download URL works without credentials, for GET only.
    let answer = fetch(
        &state,
        Some(&token),
        Method::GET,
        &format!("{MEDIA}/{id}/url"),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::OK);
    let download = answer.json()?;
    let url = download["url"].as_str().ok_or("no url")?;
    let fetched = fetch(&state, None, Method::GET, url).await?;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.bytes, PNG);
    assert_eq!(fetched.header("content-type"), Some("image/png"));
    let put_with_download = send(&state, None, Method::PUT, url, &[], PNG.to_vec()).await?;
    assert_eq!(put_with_download.status, StatusCode::FORBIDDEN);
    Ok(())
}

#[actix_web::test]
async fn bytes_that_are_not_what_was_declared_remove_the_upload() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let token = user(&db.pool).await?;

    // Declared a PNG, sent a PDF: `422`, and the upload is gone.
    let ticket = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": PDF.len()}),
    )
    .await?
    .json()?;
    let id = ticket["media_id"].as_str().ok_or("no media_id")?.to_owned();
    assert_eq!(
        put(&state, &ticket, PDF).await?.status,
        StatusCode::NO_CONTENT
    );
    let answer = fetch(
        &state,
        Some(&token),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::UNPROCESSABLE_ENTITY);
    let again = fetch(
        &state,
        Some(&token),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(again.status, StatusCode::NOT_FOUND);

    // Declared one size, sent another: `422` too.
    let ticket = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": PNG.len() + 1}),
    )
    .await?
    .json()?;
    let id = ticket["media_id"].as_str().ok_or("no media_id")?.to_owned();
    put(&state, &ticket, PNG).await?;
    let answer = fetch(
        &state,
        Some(&token),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(answer.status, StatusCode::UNPROCESSABLE_ENTITY);
    Ok(())
}

#[actix_web::test]
async fn an_upload_is_checked_before_it_starts_and_is_its_owners_alone() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let token = user(&db.pool).await?;

    let unallowed = start(
        &state,
        &token,
        &json!({"content_type": "application/x-msdownload", "size_bytes": 10}),
    )
    .await?;
    assert_eq!(unallowed.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let too_large = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": 1_000_000_000}),
    )
    .await?;
    assert_eq!(too_large.status, StatusCode::PAYLOAD_TOO_LARGE);
    let empty = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": 0}),
    )
    .await?;
    assert_eq!(empty.status, StatusCode::UNPROCESSABLE_ENTITY);

    let ticket = start(
        &state,
        &token,
        &json!({"content_type": "image/png", "size_bytes": PNG.len()}),
    )
    .await?
    .json()?;
    let id = ticket["media_id"].as_str().ok_or("no media_id")?.to_owned();
    put(&state, &ticket, PNG).await?;
    let other = user(&db.pool).await?;
    let theirs = fetch(
        &state,
        Some(&other),
        Method::POST,
        &format!("{MEDIA}/{id}/complete"),
    )
    .await?;
    assert_eq!(theirs.status, StatusCode::NOT_FOUND);

    // A forged token stores nothing.
    let forged = send(
        &state,
        None,
        Method::PUT,
        "/api/v1/storage/not-a-token",
        &[],
        PNG.to_vec(),
    )
    .await?;
    assert_eq!(forged.status, StatusCode::FORBIDDEN);
    Ok(())
}
