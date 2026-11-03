// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `user_events` feature: `UserEventsService.Subscribe`
//! (`proto/user_events.proto`), a server-streaming RPC fed by `PostgreSQL`
//! `LISTEN`/`NOTIFY`. Each `NOTIFY user_events, '<user id>'`, sent by
//! anyone sharing the database ([`UserEventsService::publish`], a trigger,
//! another process), is one event to every subscriber. Hand-written in the
//! generated features' layout: rok-data-sqlc generates no streaming RPC.

/// The gRPC service: check the caller, subscribe, turn each delivery into
/// a message.
pub mod grpc;
/// The rows and typed ids, from the data layer.
pub mod model;
/// The channel: listening on it and notifying it.
pub mod repo;
/// `UserEventsService`: the subscription's rules.
pub mod service;

pub use repo::CHANNEL;
pub use service::{Delivery, UserEventsService};
