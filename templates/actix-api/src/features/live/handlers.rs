// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `live` handlers: a ticket for a caller the guard lets in, and the
//! stream of notices a ticket opens.

use std::convert::Infallible;
use std::fmt::Display;
use std::time::Duration;

use actix_web::http::header;
use actix_web::web::{self, Bytes};
use actix_web::{HttpResponse, Responder};
use futures_util::StreamExt;

use super::dto::{LiveTicket, Subscribe};
use crate::app::AppState;
use crate::error::AppError;
use crate::shared::auth::{Allowed, Guard};
use crate::shared::live::Tickets;

/// How often an idle stream sends a comment, so proxies keep it open.
const KEEP_ALIVE: Duration = Duration::from_secs(15);

/// `POST /api/v1/live/tickets`: a ticket to the Topics of the tenant `G`
/// grants the caller, `200`; a caller `G` refuses is a `403`.
pub async fn ticket<G>(
    state: web::Data<AppState>,
    Allowed(tenant): Allowed<G>,
) -> Result<HttpResponse, AppError>
where
    G: Guard<Path = ()> + 'static,
    G::Grant: Display,
{
    Ok(HttpResponse::Ok().json(LiveTicket {
        ticket: state.live.tickets.issue(&tenant.to_string())?,
        expires_in: Tickets::TTL.as_secs(),
    }))
}

/// `GET /api/v1/live?topics=…&ticket=…`: the notices on `topics` for the
/// ticket's tenant, as server-sent events, kept alive with a comment every
/// 15 seconds. A missing, forged or expired ticket is a `401`.
pub async fn subscribe(
    state: web::Data<AppState>,
    query: web::Query<Subscribe>,
) -> Result<impl Responder, AppError> {
    let ticket = query
        .ticket
        .as_deref()
        .ok_or_else(|| AppError::Unauthorized("a ticket is required".to_owned()))?;
    let ticket = state.live.tickets.check(ticket)?;
    let notices = state
        .live
        .subscribe(&ticket.tenant, query.topics())
        .map(|notice| {
            let data = serde_json::to_string(&notice).unwrap_or_default();
            Bytes::from(format!("data: {data}\n\n"))
        });
    let keep_alive = futures_util::stream::unfold((), |()| async {
        tokio::time::sleep(KEEP_ALIVE).await;
        Some((Bytes::from_static(b": keep-alive\n\n"), ()))
    });
    // The notices end with an empty chunk, which ends the keep-alive too, so
    // a dropped subscriber's client reconnects.
    let notices = notices.chain(futures_util::stream::once(std::future::ready(Bytes::new())));
    let events = futures_util::stream::select(notices, keep_alive)
        .take_while(|bytes| std::future::ready(!bytes.is_empty()))
        .map(Ok::<_, Infallible>);
    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(events))
}
