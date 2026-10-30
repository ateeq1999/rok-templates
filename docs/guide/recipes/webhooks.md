# Recipe: signed outbound webhooks

Telling another system that something happened, such as an order placed
or a user invited, is a job ([Jobs](../jobs.md)). The queue gives you the
rest: the event is enqueued on the same transaction as the change, so a
rolled-back request sends nothing. A failed delivery is retried with
backoff, and one that keeps failing is `dead`, visible and retryable from
`/api/v1/jobs`. The format below follows
[Standard Webhooks](https://www.standardwebhooks.com/), so receivers can
verify it with existing libraries.

## 1. Where they go

A migration with the subscribers. Each one has its own secret: rotating a
secret affects only that receiver, and a leaked secret forges only that
receiver's events.

```sql
CREATE TABLE webhook_endpoints (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url        TEXT NOT NULL,
    secret     TEXT NOT NULL,          -- "whsec_" + base64 of 32 random bytes
    events     TEXT[] NOT NULL,        -- {"order.placed", ...}
    is_active  BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Seal `secret` at rest the way `TOTP_ENCRYPTION_KEY` seals two-factor
secrets (`shared/security/totp.rs`) if the database is less trusted than
the receivers. Accept only `https://` URLs. Refuse a URL whose host
resolves to a private, loopback or link-local address, so a subscriber
cannot use the API to call your own network.

## 2. The job

One job per event and endpoint. A failure then retries only the receiver
that failed. The message id stays the same across retries, so a receiver
can drop a repeat.

<!-- not compiled: a recipe; a feature's service.rs -->
```rust
#[derive(Serialize, Deserialize)]
pub struct DeliverWebhook {
    pub endpoint_id: Uuid,
    pub message_id: Uuid, // the same for every attempt
    pub event: String,
    pub data: serde_json::Value,
}

#[async_trait]
impl Job for DeliverWebhook {
    const KIND: &'static str = "webhook.deliver";
    const MAX_ATTEMPTS: i32 = 8;
    type Context = WebhookService; // the pool and a reqwest::Client with a timeout

    async fn run(self, service: &WebhookService) -> Result<(), JobError> {
        let Some(endpoint) = service.active_endpoint(self.endpoint_id).await? else {
            return Ok(()); // removed or disabled since: nothing to send
        };
        let body = serde_json::to_string(&json!({ "type": self.event, "data": self.data }))?;
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let id = self.message_id.to_string();
        let signature = sign(&endpoint.secret, &id, &timestamp, &body)?;
        let response = service
            .http
            .post(&endpoint.url)
            .header("content-type", "application/json")
            .header("webhook-id", &id)
            .header("webhook-timestamp", &timestamp)
            .header("webhook-signature", format!("v1,{signature}"))
            .body(body)
            .send()
            .await?;
        response.error_for_status()?; // a non-2xx is a failed attempt, retried
        Ok(())
    }
}

/// HMAC-SHA256 over `{id}.{timestamp}.{body}`, keyed by the secret's bytes.
fn sign(secret: &str, id: &str, timestamp: &str, body: &str) -> Result<String, JobError> {
    let key = BASE64.decode(secret.trim_start_matches("whsec_"))?;
    let mut mac = Hmac::<Sha256>::new_from_slice(&key)?;
    mac.update(format!("{id}.{timestamp}.{body}").as_bytes());
    Ok(BASE64.encode(mac.finalize().into_bytes()))
}
```

`hmac`, `sha2`, `base64` and `reqwest` are dependencies already. Give the
client a timeout (`reqwest::Client::builder().timeout(Duration::from_secs(10))`)
and no redirects (`redirect(Policy::none())`), so a slow or redirecting
receiver cannot hold a worker or reach your network. Make sure the
conversions `?` uses exist, or map each error to a `JobError` first.

## 3. Register and enqueue

Register the kind in `app/run.rs`'s `worker`:
`.register::<DeliverWebhook>(webhook_service)`. Where the event happens,
enqueue one job per subscribed endpoint, on the transaction that makes the
change:

<!-- not compiled: a recipe; inside the service method that places the order -->
```rust
let message_id = Uuid::new_v4();
for endpoint in webhooks::subscribed(&mut *tx, "order.placed").await? {
    queue.enqueue(&mut tx, &DeliverWebhook {
        endpoint_id: endpoint.id,
        message_id,
        event: "order.placed".to_owned(),
        data: json!({ "id": order.id }),
    }).await?;
}
queue.commit(tx).await?;
```

Send ids, not whole rows. The receiver fetches what it needs through the
API, so a payload never outlives a change to permissions.

## 4. Tell the receivers

They verify the signature over the raw body, compare it in constant time,
reject a `webhook-timestamp` more than five minutes old, and use
`webhook-id` to ignore a repeat. Delivery is at least once and not in
order. Test the job with `Worker::run_due()` against a local HTTP server,
as `tests/jobs.rs` does for mail.
