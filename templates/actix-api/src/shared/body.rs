// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Request bodies: held back until the caller is let in ([`hold`]), and
//! raw ones (uploads) read in the handler's body ([`read`]).
//!
//! actix-web polls a handler's extractors together, whatever their order,
//! so a body extractor (`ValidatedJson`, `web::Json`, `web::Bytes`) would
//! read the body while [`AuthUser`](crate::shared::auth::AuthUser) is still
//! checking the credential against the database: a bad body could answer
//! `422` before the caller's `401` or `403`, showing a route's rules to
//! callers who can't use it, and a refused caller could make the server
//! read up to the route's limit. So the caller extractors (`AuthUser`,
//! `AdminUser`, `Allowed<G>`) each [`hold`] the body: it reads nothing
//! until every one of them has let the caller in, and nothing at all if one
//! refuses. [`hold_body`](crate::app::middleware::hold_body), which `app`
//! wraps every request in, makes that hold work for a body extractor taken
//! before the caller too; without it (an `App` built by hand), only the
//! extractors after the caller are held back.
//!
//! What actix-web checks before reading is still answered first: a wrong
//! `Content-Type` (`415`) or a `Content-Length` over the limit (`413`) can
//! come before the caller's `401` or `403`, since neither reads the body.

use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

use actix_web::dev::{Payload, ServiceRequest};
use actix_web::error::PayloadError;
use actix_web::{HttpMessage, HttpRequest, web};
use futures_util::Stream;

use crate::error::AppError;

/// The body of `payload`, at most `limit` bytes. More is
/// [`AppError::too_large`] (the `what` over `limit` bytes), in the wire
/// standard's body rather than actix-web's plain text; a body that can't
/// be read is a `400`. A `web::Payload` parameter reads nothing until it is
/// called, once the handler's extractors have succeeded.
pub async fn read(payload: web::Payload, limit: usize, what: &str) -> Result<web::Bytes, AppError> {
    match payload.to_bytes_limited(limit).await {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(AppError::BadRequest(error.to_string())),
        Err(_) => Err(AppError::too_large(what, limit)),
    }
}

/// A hold on the request's body, from [`hold`]: the body is read once every
/// hold has been [`Pass::open`]ed. Dropped unopened (the caller was
/// refused, and the handler won't be called), the body reads as empty.
#[must_use = "the body is read only once the pass is opened"]
pub struct Pass {
    gate: Rc<RefCell<Gate>>,
    opened: bool,
}

impl Pass {
    /// Lets the body be read, as far as this hold goes.
    pub fn open(mut self) {
        self.opened = true;
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        let mut gate = self.gate.borrow_mut();
        gate.holds -= 1;
        gate.refused |= !self.opened;
        if let Some(reader) = gate.reader.take() {
            reader.wake();
        }
    }
}

/// Holds `req`'s body back until the returned [`Pass`] is opened. A caller
/// extractor takes it in `from_request`, before any extractor's future is
/// polled, and opens it once it has let the caller in.
///
/// The body is the one [`hold_body`](crate::app::middleware::hold_body)
/// gated, so a body extractor taken before the caller waits too; when no
/// middleware did, `payload` is gated here, for the extractors after it.
pub fn hold(req: &HttpRequest, payload: &mut Payload) -> Pass {
    let gated = req
        .extensions()
        .get::<Gated>()
        .map(|gated| Rc::clone(&gated.0));
    let gate = gated.unwrap_or_else(|| {
        let gate = Rc::default();
        *payload = gated_payload(payload.take(), &gate);
        req.extensions_mut().insert(Gated(Rc::clone(&gate)));
        gate
    });
    gate.borrow_mut().holds += 1;
    Pass {
        gate,
        opened: false,
    }
}

/// Gates `req`'s body, so that a [`hold`] taken by any of the handler's
/// extractors, before or after the body's, holds it back.
pub fn gate(req: &mut ServiceRequest) {
    let gate = Rc::default();
    let payload = req.take_payload();
    req.set_payload(gated_payload(payload, &gate));
    req.extensions_mut().insert(Gated(gate));
}

/// The gate of a request's body, in its extensions.
struct Gated(Rc<RefCell<Gate>>);

/// The holds on a body, and its reader to wake when one goes.
#[derive(Default)]
struct Gate {
    holds: usize,
    refused: bool,
    reader: Option<Waker>,
}

/// `payload`, read only while `gate` has no holds on it, and as empty once
/// one was dropped unopened.
fn gated_payload(payload: Payload, gate: &Rc<RefCell<Gate>>) -> Payload {
    Payload::Stream {
        payload: Box::pin(GatedBody {
            gate: Rc::clone(gate),
            payload,
        }),
    }
}

/// The body [`gated_payload`] makes.
struct GatedBody {
    gate: Rc<RefCell<Gate>>,
    payload: Payload,
}

impl Stream for GatedBody {
    type Item = Result<web::Bytes, PayloadError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        {
            let mut gate = self.gate.borrow_mut();
            if gate.refused {
                return Poll::Ready(None);
            }
            if gate.holds > 0 {
                gate.reader = Some(cx.waker().clone());
                return Poll::Pending;
            }
        }
        Pin::new(&mut self.payload).poll_next(cx)
    }
}
