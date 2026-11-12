// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` feature's request and response bodies.

use serde::{Deserialize, Serialize};

/// `POST /api/v1/live/tickets`'s answer: what opens a subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTicket {
    /// The ticket, for `GET /api/v1/live?ticket=…`.
    pub ticket: String,
    /// How many seconds the ticket opens subscriptions for.
    pub expires_in: u64,
}

/// `GET /api/v1/live`'s query.
#[derive(Debug, Clone, Deserialize)]
pub struct Subscribe {
    /// The Topics, comma-separated: `orders.new,bills.paid`.
    #[serde(default)]
    pub topics: String,
    /// A ticket from `POST /api/v1/live/tickets`.
    pub ticket: Option<String>,
}

impl Subscribe {
    /// The Topics named, each once, blanks left out.
    #[must_use]
    pub fn topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = self
            .topics
            .split(',')
            .map(str::trim)
            .filter(|topic| !topic.is_empty())
            .map(str::to_owned)
            .collect();
        topics.sort();
        topics.dedup();
        topics
    }
}
