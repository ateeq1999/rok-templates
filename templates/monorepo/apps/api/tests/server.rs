//! `app::serve`, what `main` runs: the application on a real socket with
//! the mail sender beside it, both stopped by the one shutdown signal.

mod support;

use std::sync::Arc;
use std::time::Duration;

use axum_api::app::{serve, serve_as};
use axum_api::config::AppRole;
use axum_api::shared::mail::{Mail, MemoryMailer};
use support::{Database, TestResult, state};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A mail to `to`.
fn mail(to: &str) -> Mail {
    Mail {
        to: to.to_owned(),
        subject: "Hello".to_owned(),
        text: "Hello there".to_owned(),
        html: "<p>Hello there</p>".to_owned(),
    }
}

/// Waits up to two seconds for `mailer` to have sent `count` mails.
async fn sent(mailer: &MemoryMailer, count: usize) -> Vec<Mail> {
    let mut waited = Duration::ZERO;
    while mailer.sent().len() < count && waited < Duration::from_secs(2) {
        tokio::time::sleep(Duration::from_millis(20)).await;
        waited += Duration::from_millis(20);
    }
    mailer.sent()
}

#[tokio::test]
async fn the_server_runs_the_mail_sender_and_both_stop_on_shutdown() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = state(&db.pool)?;
    let outbox = state.outbox.clone();
    let mailer = Arc::new(MemoryMailer::default());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, state, mailer.clone(), async {
        let _ = stopped.await;
    }));

    let mut stream = TcpStream::connect(addr).await?;
    stream
        .write_all(b"GET /health/live HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n")
        .await?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer).await?;
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");

    let mut tx = db.pool.begin().await?;
    outbox.enqueue(&mut tx, &mail("first@test.invalid")).await?;
    outbox.commit(tx).await?;
    assert_eq!(sent(&mailer, 1).await, vec![mail("first@test.invalid")]);

    let _ = stop.send(());
    tokio::time::timeout(Duration::from_secs(5), server).await???;
    assert!(TcpStream::connect(addr).await.is_err(), "still listening");

    // The sender stopped with the server: a later mail stays queued.
    let mut tx = db.pool.begin().await?;
    outbox
        .enqueue(&mut tx, &mail("second@test.invalid"))
        .await?;
    outbox.commit(tx).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(mailer.sent().len(), 1);
    db.drop().await
}

/// The status line `GET path` is answered on `addr`.
async fn status(addr: std::net::SocketAddr, path: &str) -> Result<String, std::io::Error> {
    let mut stream = TcpStream::connect(addr).await?;
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .await?;
    let mut answer = String::new();
    stream.read_to_string(&mut answer).await?;
    Ok(answer.lines().next().unwrap_or_default().to_owned())
}

/// `APP_ROLE=web` serves the API and runs no job: a mail it enqueues waits
/// for a worker. `APP_ROLE=worker` serves only `/health` and runs the job.
#[tokio::test]
async fn the_web_role_serves_the_api_and_the_worker_role_runs_the_jobs() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let mailer = Arc::new(MemoryMailer::default());
    let start = |role: AppRole| {
        let state = state(&db.pool);
        let mailer = mailer.clone();
        async move {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let addr = listener.local_addr()?;
            let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
            let server = tokio::spawn(serve_as(role, listener, state?, mailer, async {
                let _ = stopped.await;
            }));
            Ok::<_, Box<dyn std::error::Error>>((addr, stop, server))
        }
    };

    let (addr, stop, server) = start(AppRole::Web).await?;
    assert!(status(addr, "/health/live").await?.contains("200"));
    assert!(
        status(addr, "/api-docs/openapi.json")
            .await?
            .contains("200")
    );
    let outbox = state(&db.pool)?.outbox;
    let mut tx = db.pool.begin().await?;
    outbox
        .enqueue(&mut tx, &mail("queued@test.invalid"))
        .await?;
    outbox.commit(tx).await?;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(mailer.sent().is_empty(), "the web role ran a job");
    let _ = stop.send(());
    tokio::time::timeout(Duration::from_secs(5), server).await???;

    let (addr, stop, server) = start(AppRole::Worker).await?;
    assert!(status(addr, "/health/live").await?.contains("200"));
    assert!(
        status(addr, "/api-docs/openapi.json")
            .await?
            .contains("404")
    );
    assert_eq!(sent(&mailer, 1).await, vec![mail("queued@test.invalid")]);
    let _ = stop.send(());
    tokio::time::timeout(Duration::from_secs(5), server).await???;
    db.drop().await
}

/// On shutdown the server first drains: `/health/ready` answers `503`
/// (the load balancer stops sending requests) while every request is still
/// served, for `SHUTDOWN_DELAY_SECS`; then it stops.
#[tokio::test]
async fn on_shutdown_the_server_is_unready_but_serving_then_stops() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let state = support::state_with(&db.pool, &[("SHUTDOWN_DELAY_SECS", "1")])?;
    let mailer = Arc::new(MemoryMailer::default());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve(listener, state, mailer, async {
        let _ = stopped.await;
    }));
    assert!(status(addr, "/health/ready").await?.contains("200"));
    let _ = stop.send(());
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(status(addr, "/health/ready").await?.contains("503"));
    assert!(status(addr, "/health/live").await?.contains("200"));
    assert!(
        status(addr, "/api-docs/openapi.json")
            .await?
            .contains("200")
    );
    tokio::time::timeout(Duration::from_secs(5), server).await???;
    assert!(TcpStream::connect(addr).await.is_err(), "still listening");
    db.drop().await
}
