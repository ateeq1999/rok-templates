// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! Guards: an authorization rule written once, as one [`Guard`], and
//! required by type wherever it applies.
//!
//! A guard looks at the caller, the application's state and the route's
//! path parameters, and answers how the caller is let in (its
//! [`Guard::Grant`]: a role, or "staff") or that it is not (`None`, a
//! `403`). A handler takes `Allowed<G>` (`shared/auth.rs`), which runs the
//! guard before the request's body is read and hands the handler the
//! grant:
//!
//! ```text
//! pub async fn update(
//!     Allowed(access): Allowed<PartnerAccess>,
//!     ValidatedJson(body): ValidatedJson<UpdatePartner>,
//! ) -> Result<Json<Partner>, AppError>
//! ```
//!
//! A plain yes-or-no rule grants `()` (`_: Allowed<Finance>`). Back each
//! rule with one read query in `queries/shared/`, which any feature may
//! call, so the rule's SQL is written once too. Outside HTTP (a gRPC
//! service, a job), [`require`] answers the same way. The template ships
//! no guard of its own: the rules are the project's (the guide's Guards
//! page shows some).

use std::future::Future;

use serde::de::DeserializeOwned;

use crate::app::AppState;
use crate::error::AppError;
use crate::shared::auth::AuthUser;

/// A named authorization rule. Implement it on a unit struct named after
/// the rule (`Finance`, `PartnerAccess`), with `async fn check`.
pub trait Guard {
    /// How the caller was let in, handed to the handler: `()` for a plain
    /// yes or no, an enum when there are several ways in (a member's role,
    /// or staff).
    type Grant: Send + 'static;

    /// The path parameters the rule reads: a struct deriving `Deserialize`
    /// whose fields are named after the route's parameters (`{partner_id}`
    /// is `partner_id`), the others ignored; `()` for none.
    type Path: DeserializeOwned + Send + Sync + 'static;

    /// What a refused caller is told, in the `403`'s message.
    const REFUSAL: &'static str = "you don't have access to this";

    /// How `caller` is let in at `path`, or `None` for a `403` with
    /// [`Self::REFUSAL`]. A refusal with its own message is an
    /// `Err(AppError::Forbidden(..))`.
    fn check(
        caller: &AuthUser,
        state: &AppState,
        path: &Self::Path,
    ) -> impl Future<Output = Result<Option<Self::Grant>, AppError>> + Send;
}

/// `G`'s grant for `caller` at `path`.
///
/// # Errors
///
/// A `403` (`AppError::Forbidden`, with [`Guard::REFUSAL`]) when `G`
/// refuses the caller, or whatever `G::check` fails with.
pub async fn require<G: Guard>(
    caller: &AuthUser,
    state: &AppState,
    path: &G::Path,
) -> Result<G::Grant, AppError> {
    G::check(caller, state, path)
        .await?
        .ok_or_else(|| AppError::Forbidden(G::REFUSAL.to_owned()))
}
