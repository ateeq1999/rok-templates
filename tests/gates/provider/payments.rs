//! Payments: the provider the API charges through, which calls back when
//! a charge settles. The port ([`Payments`]), its transports chosen by
//! `PAYMENTS_TRANSPORT` ([`HttpPayments`], the provider's API, and
//! [`FakePayments`], which charges nothing and signs its callbacks exactly
//! as the provider does), and the callback's check ([`PaymentsSignature`]).

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::shared::security::webhook::{Delivery, WebhookSecret, WebhookVerifier};
use crate::shared::settings::{ConfigError, Vars};

/// One charge to make.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Charge {
    /// The amount, in the currency's smallest unit (cents).
    pub amount: i64,
    /// The ISO 4217 code, such as `usd`.
    pub currency: String,
    /// Your id for it (an order's), sent as the idempotency key, so a retry
    /// never charges twice.
    pub reference: String,
}

/// What the provider calls back with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentEvent {
    /// What happened, such as `charge.settled`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The charge it happened to: the id [`Payments::charge`] returned.
    pub charge_id: String,
}

/// Why a charge was not made. It goes to the log, never to an API client.
#[derive(Debug, thiserror::Error)]
#[error("the payment provider failed: {0}")]
pub struct PaymentError(String);

/// The port every transport implements.
#[async_trait]
pub trait Payments: Send + Sync {
    /// Makes `charge`, returning the provider's id for it. It settles
    /// later: the provider calls back.
    async fn charge(&self, charge: &Charge) -> Result<String, PaymentError>;

    /// The secret the provider signs its callbacks with.
    fn webhook_secret(&self) -> &WebhookSecret;
}

/// The check `SignedWebhook<PaymentsSignature>` makes: the callback is
/// signed with the secret of the transport in the state.
pub struct PaymentsSignature;

impl WebhookVerifier for PaymentsSignature {
    fn verify(state: &AppState, delivery: &Delivery<'_>) -> bool {
        state.payments.webhook_secret().verify(delivery)
    }
}

/// `PAYMENTS_TRANSPORT` and the settings it needs.
pub enum PaymentSettings {
    /// `fake` (the default): nothing is charged. For running locally.
    Fake,
    /// `http`: the provider's API at `PAYMENTS_URL`, with `PAYMENTS_API_KEY`;
    /// its callbacks are signed with `PAYMENTS_WEBHOOK_SECRET`.
    Http {
        /// `PAYMENTS_URL`.
        url: String,
        /// `PAYMENTS_API_KEY`.
        api_key: String,
        /// `PAYMENTS_WEBHOOK_SECRET`.
        webhook_secret: WebhookSecret,
    },
}

impl PaymentSettings {
    /// The settings `PAYMENTS_TRANSPORT` names, read in `Config::from_vars`.
    pub fn read(vars: &Vars<'_>, var: &impl Fn(&str) -> Option<String>) -> Self {
        match var("PAYMENTS_TRANSPORT").as_deref() {
            None | Some("fake") => PaymentSettings::Fake,
            Some("http") => {
                let url = vars.required("PAYMENTS_URL");
                let api_key = vars.required("PAYMENTS_API_KEY");
                let secret = vars.required("PAYMENTS_WEBHOOK_SECRET");
                let webhook_secret = WebhookSecret::new(&secret).unwrap_or_else(|_| {
                    // A missing one is reported already.
                    if !secret.is_empty() {
                        vars.fail(ConfigError::NotOneOf {
                            name: "PAYMENTS_WEBHOOK_SECRET",
                            value: "(hidden)".to_owned(),
                            expected: "`whsec_` and base64",
                        });
                    }
                    WebhookSecret::random()
                });
                PaymentSettings::Http {
                    url,
                    api_key,
                    webhook_secret,
                }
            }
            Some(other) => {
                vars.fail(ConfigError::NotOneOf {
                    name: "PAYMENTS_TRANSPORT",
                    value: other.to_owned(),
                    expected: "fake, http",
                });
                PaymentSettings::Fake
            }
        }
    }
}

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

/// The provider's API.
pub struct HttpPayments {
    http: reqwest::Client,
    url: String,
    api_key: String,
    webhook_secret: WebhookSecret,
}

impl HttpPayments {
    /// The API at `url`, called with `api_key`.
    #[must_use]
    pub fn new(url: &str, api_key: &str, webhook_secret: WebhookSecret) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        HttpPayments {
            http,
            url: url.trim_end_matches('/').to_owned(),
            api_key: api_key.to_owned(),
            webhook_secret,
        }
    }
}

/// The provider's answer to a charge.
#[derive(Deserialize)]
struct Charged {
    id: String,
}

#[async_trait]
impl Payments for HttpPayments {
    async fn charge(&self, charge: &Charge) -> Result<String, PaymentError> {
        let failed = |err: &dyn std::fmt::Display| PaymentError(err.to_string());
        let body = serde_json::to_vec(charge).map_err(|err| failed(&err))?;
        let response = self
            .http
            .post(format!("{}/charges", self.url))
            .bearer_auth(&self.api_key)
            .header("content-type", "application/json")
            .header("idempotency-key", &charge.reference)
            .body(body)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|err| failed(&err))?;
        let bytes = response.bytes().await.map_err(|err| failed(&err))?;
        let charged: Charged = serde_json::from_slice(&bytes).map_err(|err| failed(&err))?;
        Ok(charged.id)
    }

    fn webhook_secret(&self) -> &WebhookSecret {
        &self.webhook_secret
    }
}

/// Charges nothing: keeps each charge, so a test can read it, and makes
/// the callbacks the provider would send, signed with its own secret
/// exactly as the provider signs them.
pub struct FakePayments {
    secret: WebhookSecret,
    charges: Mutex<Vec<Charge>>,
}

impl Default for FakePayments {
    fn default() -> Self {
        FakePayments {
            secret: WebhookSecret::random(),
            charges: Mutex::default(),
        }
    }
}

impl FakePayments {
    /// Every charge made so far, oldest first.
    #[must_use]
    pub fn charges(&self) -> Vec<Charge> {
        self.charges
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

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
}

#[async_trait]
impl Payments for FakePayments {
    async fn charge(&self, charge: &Charge) -> Result<String, PaymentError> {
        let mut charges = self.charges.lock().unwrap_or_else(PoisonError::into_inner);
        charges.push(charge.clone());
        Ok(format!("ch_fake_{}", charges.len()))
    }

    fn webhook_secret(&self) -> &WebhookSecret {
        &self.secret
    }
}
