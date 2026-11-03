// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `user_events` gRPC service: `Subscribe` checks the caller, starts a
//! subscription ([`UserEventsService::listen`]) and streams each
//! [`Delivery`] as a message. `shared/grpc.rs` reads the caller and turns
//! the values and the errors.

use std::pin::Pin;

use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use super::super::proto;
use super::super::proto::user_events_service_server::UserEventsService as UserEventsRpc;
use super::service::{BUFFER, Delivery, UserEventsService};
use crate::error::AppError;
use crate::shared::grpc::{admin, write};

/// The events a subscriber receives, until the stream ends.
pub type Events = Pin<Box<dyn Stream<Item = Result<proto::UserEvent, Status>> + Send>>;

#[tonic::async_trait]
impl UserEventsRpc for UserEventsService {
    type SubscribeStream = Events;

    async fn subscribe(
        &self,
        request: Request<proto::SubscribeUserEventsRequest>,
    ) -> Result<Response<Events>, Status> {
        admin(&request)?;
        let deliveries = self.listen().await?;
        let events = ReceiverStream::new(deliveries).map(event);
        Ok(Response::new(Box::pin(events)))
    }
}

/// `delivery` as the stream's next item: a change is an event; falling
/// behind ends the stream with `RESOURCE_EXHAUSTED`, a lost connection with
/// `UNAVAILABLE`, and a failure with its status. After either, subscribe
/// again and re-read what was missed.
fn event(delivery: Result<Delivery, AppError>) -> Result<proto::UserEvent, Status> {
    match delivery? {
        Delivery::Changed { id, user } => Ok(proto::UserEvent {
            user_id: write(id)?,
            user: user.map(|user| proto::User::try_from(*user)).transpose()?,
        }),
        Delivery::FellBehind => Err(Status::resource_exhausted(format!(
            "the subscriber fell {BUFFER} events behind: subscribe again"
        ))),
        Delivery::Interrupted => Err(Status::unavailable(
            "the database connection was lost and events were missed: subscribe again",
        )),
    }
}
