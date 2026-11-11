// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` feature: Topics as server-sent events. A signed-in caller
//! the [`Subscriber`] guard lets in asks `POST /api/v1/live/tickets` for a
//! short-lived ticket to its tenant's Topics, then opens
//! `GET /api/v1/live?topics=orders.new,bills.paid&ticket=…` (a browser's
//! `EventSource` can't send a bearer token). Each notice a feature
//! publishes there (`shared::live::publish`) arrives as one event whose
//! data is the notice: `{"topic": "orders.new", "payload": {…}}`.
//!
//! The routes are left out of the `OpenAPI` document: a stream of events
//! has no response schema a generated client could call.

/// Request and response bodies.
pub mod dto;
/// The handlers: parse, call the hub, answer.
pub mod handlers;
/// The URL to handler mapping.
pub mod routes;
/// [`Subscriber`]: who may subscribe, and to which tenant's Topics.
pub mod service;

pub use routes::routes;
pub use service::Subscriber;
