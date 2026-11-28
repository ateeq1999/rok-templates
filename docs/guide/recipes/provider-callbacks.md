# Recipe: a provider that calls back

A payment provider, a courier, an SMS gateway: you call its API, and later
it calls yours back to say what happened. Mail already has the shape this
takes (`src/shared/mail/`), and this recipe repeats it for a provider of
your own. Each one gets:

- **a port**: a trait the features call, never a vendor's SDK;
- **a real and a fake transport**, chosen by `<NAME>_TRANSPORT`;
- **a fake that signs its callbacks exactly as the provider does**, so a
  test drives the real signature check;
- **a test helper** that swaps the fake into `AppState`.

The example is a made-up payment provider, `payments`. The templates ship
no vendor: swap in yours, and its signing scheme if it isn't
[Standard Webhooks](https://www.standardwebhooks.com/). The code below is
compiled and tested by the axum template's gate
(`tests/gates/provider/`).

**Tests and transports.** Mail's `log` transport is for running locally
(`MAIL_TRANSPORT=log`, the default); tests read the queued mail through
the memory transport (`MemoryMailer`, which `tests/support`'s
`mailed(pool)` sends the queue through). An SMS port is built the same
way: a `log` transport for running locally, a memory one tests read the
queued messages from. The `fake` below plays both parts for payments.

## 1. The port

`src/shared/payments.rs`. Features charge through the trait; the
transport holds the secret the callbacks are signed with.

<!-- source: tests/gates/provider/payments.rs -->
```rust
/// The port every transport implements.
#[async_trait]
pub trait Payments: Send + Sync {
    /// Makes `charge`, returning the provider's id for it. It settles
    /// later: the provider calls back.
    async fn charge(&self, charge: &Charge) -> Result<String, PaymentError>;

    /// The secret the provider signs its callbacks with.
    fn webhook_secret(&self) -> &WebhookSecret;
}
```

## 2. The transports, chosen by `PAYMENTS_TRANSPORT`

`fake` is the default, so the API runs locally with no account at the
provider; `http` needs the provider's URL, key and webhook secret, and a
missing one stops the start-up with the others (`shared/settings.rs`).
Add `pub payments: PaymentSettings` to `Config` and read it in
`Config::from_vars` with `PaymentSettings::read(&vars, &var)`.

<!-- source: tests/gates/provider/payments.rs -->
```rust
/// The transport `settings` name.
pub fn from_settings(settings: &PaymentSettings) -> Arc<dyn Payments> {
    match settings {
        PaymentSettings::Http {
            url,
            api_key,
            webhook_secret,
        } => Arc::new(HttpPayments::new(url, api_key, webhook_secret.clone())),
        PaymentSettings::Fake => {
            tracing::warn!("PAYMENTS_TRANSPORT=fake: nothing is charged");
            Arc::new(FakePayments::default())
        }
    }
}
```

`HttpPayments` posts to the provider with a timeout, and sends your
reference as the idempotency key, so a retried job never charges twice.
The fake keeps each charge for a test to read, and makes the callback the
provider would send, signed with its own secret by the same
`WebhookSecret` the check uses:

<!-- source: tests/gates/provider/payments.rs -->
```rust
    /// The callback the provider sends once the charge `charge_id` has
    /// settled: its headers and its body.
    #[must_use]
    pub fn settled(&self, charge_id: &str) -> (Vec<(&'static str, String)>, String) {
        let event = PaymentEvent {
            kind: "charge.settled".to_owned(),
            charge_id: charge_id.to_owned(),
        };
        let body = serde_json::to_string(&event).unwrap_or_default();
        let id = format!("evt_{charge_id}");
        let now = chrono::Utc::now().timestamp();
        let headers = self.secret.headers(&id, now, body.as_bytes()).to_vec();
        (headers, body)
    }
```

## 3. In the state

`src/app/state.rs`: a field, `payments: Arc<dyn Payments>`, set in
`AppState::new` with `payments::from_settings(&config.payments)`, and the
helper a test swaps the fake in with:

<!-- source: tests/gates/provider/state.rs -->
```rust
impl AppState {
    /// This state, charging through `payments` instead of the configured
    /// transport: a test's `FakePayments`.
    #[must_use]
    pub fn with_payments(mut self, payments: Arc<dyn Payments>) -> Self {
        self.payments = payments;
        self
    }
}
```

## 4. The callback

The check is a `WebhookVerifier` ([Security](../security.md#signed-webhooks)),
reading the secret of whichever transport the state holds, so the fake's
callbacks pass in a test and a real one's in production:

<!-- source: tests/gates/provider/payments.rs -->
```rust
impl WebhookVerifier for PaymentsSignature {
    fn verify(state: &AppState, delivery: &Delivery<'_>) -> bool {
        state.payments.webhook_secret().verify(delivery)
    }
}
```

The handler takes `SignedWebhook<PaymentsSignature>`: it runs only for a
callback signed by the provider, and parses the raw body itself. Route it
outside `AuthUser` (the provider has no session), and make it idempotent:
a provider delivers at least once.

<!-- source: tests/gates/provider/recipe.rs -->
```rust
pub async fn payment_callback(
    State(_state): State<AppState>,
    webhook: SignedWebhook<PaymentsSignature>,
) -> Result<Json<Value>, AppError> {
    let event: PaymentEvent = serde_json::from_slice(&webhook.body)
        .map_err(|err| AppError::BadRequest(err.to_string()))?;
    // Mark the order paid here, through the feature's service.
    Ok(Json(json!({ "settled": event.charge_id })))
}
```

## 5. The test

<!-- source: tests/gates/provider/recipe.rs -->
```rust
    let fake = Arc::new(FakePayments::default());
    let state = state(&fake)?;
    let charge = Charge {
        amount: 1_500,
        currency: "usd".to_owned(),
        reference: "order-1".to_owned(),
    };
    let id = state.payments.charge(&charge).await?;
    assert_eq!(fake.charges(), [charge]);

    let (headers, body) = fake.settled(&id);
    let answer = call_back(&state, &headers, &body).await?;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body, json!({ "settled": id }));
```

`state(&fake)` is the test's `state_with(&pool, &[])?.with_payments(fake.clone())`.
A body changed after signing, or a callback signed by another provider's
secret, is a `401` (the rest of `tests/gates/provider/recipe.rs`).
