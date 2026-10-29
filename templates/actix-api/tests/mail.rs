//! The mail outbox: a flow enqueues a mail (a `mail.send` job) on its
//! transaction, and a worker sends it once the transaction commits,
//! retrying a failing transport until the job is `dead`, its bodies gone.

mod support;

use std::sync::Arc;
use std::time::Duration;

use actix_api::shared::jobs::{Queue, Worker};
use actix_api::shared::mail::{Mail, MailError, Mailer, MemoryMailer, Outbox, SendMail, Templates};
use async_trait::async_trait;
use support::{APP_URL, Database, TestResult, mailed, mailed_token};

/// A mail to `to`.
fn mail(to: &str) -> Mail {
    Mail {
        to: to.to_owned(),
        subject: "Hello".to_owned(),
        text: "Hello there".to_owned(),
        html: "<p>Hello there</p>".to_owned(),
    }
}

/// A transport that always fails.
struct Failing;

#[async_trait]
impl Mailer for Failing {
    async fn send(&self, _mail: &Mail) -> Result<(), MailError> {
        Err(MailError::Delivery("the relay said no".to_owned()))
    }
}

/// A worker sending mail through `mailer`, retrying at once.
fn worker(pool: &sqlx::PgPool, mailer: Arc<dyn Mailer>) -> Worker {
    Worker::new(pool.clone(), &Queue::new())
        .with_backoff(Duration::ZERO)
        .register::<SendMail>(mailer)
}

/// Enqueues `mail` on its own transaction.
async fn enqueue(pool: &sqlx::PgPool, mail: &Mail) -> TestResult {
    let outbox = Outbox::new();
    let mut tx = pool.begin().await?;
    outbox.enqueue(&mut tx, mail).await?;
    outbox.commit(tx).await?;
    Ok(())
}

#[actix_web::test]
async fn an_enqueued_mail_is_sent_then_deleted() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    enqueue(&db.pool, &mail("ana@test.invalid")).await?;
    let mailer = Arc::new(MemoryMailer::default());
    let ran = worker(&db.pool, mailer.clone()).run_due().await?;
    assert_eq!((ran.done, ran.failed), (1, 0));
    assert_eq!(mailer.sent(), vec![mail("ana@test.invalid")]);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM jobs")
        .fetch_one(&db.pool)
        .await?;
    assert_eq!(left, 0);
    db.drop().await
}

/// After its fifth failure a mail is `dead`: kept with its address, its
/// subject and the error, but not the bodies (and the link they carry).
#[actix_web::test]
async fn a_mail_that_keeps_failing_is_dead_without_its_bodies() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let mut reset = mail("ana@test.invalid");
    reset.text = "Reset: http://app.test/reset-password?token=secret".to_owned();
    enqueue(&db.pool, &reset).await?;
    let ran = worker(&db.pool, Arc::new(Failing)).run_due().await?;
    assert_eq!((ran.done, ran.failed), (0, 5));
    let (status, attempts, last_error, payload): (String, i32, Option<String>, serde_json::Value) =
        sqlx::query_as("SELECT status, attempts, last_error, payload FROM jobs")
            .fetch_one(&db.pool)
            .await?;
    assert_eq!((status.as_str(), attempts), ("dead", 5));
    assert_eq!(last_error.as_deref(), Some("the relay said no"));
    assert_eq!(payload["to"], "ana@test.invalid");
    assert_eq!(payload["subject"], "Hello");
    assert!(!payload.to_string().contains("secret"), "{payload}");
    db.drop().await
}

#[actix_web::test]
async fn a_rolled_back_transaction_sends_nothing() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let mut tx = db.pool.begin().await?;
    Outbox::new()
        .enqueue(&mut tx, &mail("nobody@test.invalid"))
        .await?;
    tx.rollback().await?;
    let mailer = Arc::new(MemoryMailer::default());
    assert_eq!(worker(&db.pool, mailer.clone()).run_due().await?.done, 0);
    assert!(mailer.sent().is_empty());
    db.drop().await
}

#[actix_web::test]
async fn a_test_reads_the_token_a_delivered_link_carries() -> TestResult {
    let Some(db) = Database::fresh().await? else {
        return Ok(());
    };
    let templates = Templates::new(APP_URL);
    let mail = templates.password_reset("ana@test.invalid", "raw-Token_1");
    enqueue(&db.pool, &mail).await?;

    let sent = mailed(&db.pool).await?;
    assert_eq!(sent, vec![mail]);
    assert_eq!(mailed_token(&sent[0]).as_deref(), Some("raw-Token_1"));
    assert!(
        sent[0]
            .text
            .contains("http://app.test/reset-password?token=raw-Token_1")
    );
    db.drop().await
}
