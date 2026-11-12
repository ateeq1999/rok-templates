// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` handlers: a ticket for a caller the guard lets in, and the
//! stream of notices a ticket opens.

use std::convert::Infallible;
use std::fmt::Display;

use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::{Stream, StreamExt};

use super::dto::{LiveTicket, Subscribe};
use crate::error::AppError;
use crate::shared::auth::{Allowed, Guard};
use crate::shared::live::{Tickets, Topics};
use crate::shared::validation::Query;

/// `POST /api/v1/live/tickets`: a ticket to the Topics of the tenant `G`
/// grants the caller, `200`; a caller `G` refuses is a `403`.
pub async fn ticket<G>(
    State(topics): State<Topics>,
    Allowed(tenant): Allowed<G>,
) -> Result<Json<LiveTicket>, AppError>
where
    G: Guard<Path = ()>,
    G::Grant: Display,
{
    Ok(Json(LiveTicket {
        ticket: topics.tickets.issue(&tenant.to_string())?,
        expires_in: Tickets::TTL.as_secs(),
    }))
}

/// `GET /api/v1/live?topics=…&ticket=…`: the notices on `topics` for the
/// ticket's tenant, as server-sent events, kept alive with a comment every
/// 15 seconds. A missing, forged or expired ticket is a `401`.
pub async fn subscribe(
    State(topics): State<Topics>,
    Query(query): Query<Subscribe>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let ticket = query
        .ticket
        .as_deref()
        .ok_or_else(|| AppError::Unauthorized("a ticket is required".to_owned()))?;
    let ticket = topics.tickets.check(ticket)?;
    let notices = topics.subscribe(&ticket.tenant, query.topics());
    let events = notices.map(|notice| {
        Ok(Event::default()
            .json_data(&notice)
            .unwrap_or_else(|_| Event::default().comment("an unreadable notice")))
    });
    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}
