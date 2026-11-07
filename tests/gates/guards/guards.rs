//! The project's guards: each authorization rule written once, backed by
//! one query in `queries/shared/access.sql`, and required by type in the
//! handlers (`Allowed<Finance>`, `Allowed<PartnerAccess>`,
//! `Allowed<PartnerAccess<Organizer>>`).

use std::marker::PhantomData;

use serde::Deserialize;

use crate::app::AppState;
use crate::db::partners::models::PartnerId;
use crate::db::queries::shared::access::{partner_access, staff_role};
use crate::db::types::{PartnerKind, PartnerRole, StaffRole};
use crate::error::AppError;
use crate::shared::auth::{AuthUser, Guard};

/// Finance staff: a yes or no.
pub struct Finance;

impl Guard for Finance {
    type Grant = ();
    type Path = ();
    const REFUSAL: &'static str = "this needs the finance staff role";

    async fn check(caller: &AuthUser, state: &AppState, _: &()) -> Result<Option<()>, AppError> {
        let role = staff_role(&state.pool, caller.id.into()).await?;
        Ok((role == Some(StaffRole::Finance)).then_some(()))
    }
}

/// How a caller may act for a partner: as one of its members, with their
/// role there, or as staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerGrant {
    /// A member of the partner, with this role.
    Member(PartnerRole),
    /// Staff, with this staff role.
    Staff(StaffRole),
}

/// The path of a partner's routes: `/partners/{partner_id}/…`.
#[derive(Debug, Deserialize)]
pub struct PartnerPath {
    /// The partner the route acts for.
    pub partner_id: PartnerId,
}

/// The kind of partner a route serves: [`AnyKind`], [`Organizer`] or
/// [`Venue`].
pub trait Kind {
    /// The kind, or `None` for any.
    const KIND: Option<PartnerKind>;
    /// What a caller is told at a partner of another kind.
    const WRONG_KIND: &'static str;
}

/// A partner of any kind.
pub struct AnyKind;

impl Kind for AnyKind {
    const KIND: Option<PartnerKind> = None;
    const WRONG_KIND: &'static str = "";
}

/// An organizer.
pub struct Organizer;

impl Kind for Organizer {
    const KIND: Option<PartnerKind> = Some(PartnerKind::Organizer);
    const WRONG_KIND: &'static str = "this isn't an organizer";
}

/// A venue.
pub struct Venue;

impl Kind for Venue {
    const KIND: Option<PartnerKind> = Some(PartnerKind::Venue);
    const WRONG_KIND: &'static str = "this isn't a venue";
}

/// A member of the partner at `{partner_id}`, or staff; with a kind,
/// `PartnerAccess<Organizer>`, only at a partner of that kind. One query
/// checks the membership, the staff role and the kind together.
pub struct PartnerAccess<K = AnyKind>(PhantomData<K>);

impl<K: Kind> Guard for PartnerAccess<K> {
    type Grant = PartnerGrant;
    type Path = PartnerPath;

    async fn check(
        caller: &AuthUser,
        state: &AppState,
        path: &PartnerPath,
    ) -> Result<Option<PartnerGrant>, AppError> {
        let Some(access) = partner_access(&state.pool, caller.id.into(), path.partner_id).await?
        else {
            return Ok(None);
        };
        let grant = match (access.member_role, access.staff_role) {
            (Some(role), _) => PartnerGrant::Member(role),
            (None, Some(role)) => PartnerGrant::Staff(role),
            (None, None) => return Ok(None),
        };
        if K::KIND.is_some_and(|kind| kind != access.kind) {
            return Err(AppError::Forbidden(K::WRONG_KIND.to_owned()));
        }
        Ok(Some(grant))
    }
}
