# Guards

A **Guard** is an authorization rule a project writes once and a handler
requires by type: `_: Allowed<Finance>`, or
`Allowed(access): Allowed<PartnerAccess>` when the handler needs to know
how the caller was let in. The templates' security Capability ships the
trait and the extractor; the rules themselves are the project's. Guards
are in the axum and actix templates and the monorepo's API; the gRPC
template has the trait and `require` (below).

Write a role check as a guard, not in a service: the extractor runs before
the body is read, so a caller without permission gets the `403` before a
`422` reveals the body's rules, and the rule lives in one place instead of
being copied into every handler that needs it.

## The trait

<!-- source: templates/axum-api/src/shared/security/guard.rs -->
```rust
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
```

`Allowed<G>` (in `shared/auth.rs`, next to `AuthUser`) reads the caller
(none is a `401`), deserializes `G::Path` from the route's parameters (a
malformed one is a `400`), calls `G::check` and answers `None` with a `403`
carrying `G::REFUSAL`. Otherwise the handler gets the grant:
`Allowed(grant)`.

In axum, put it ahead of the body extractor (the body is the last argument
anyway). actix-web polls a handler's extractors together, so `Allowed`,
like `AuthUser` and `AdminUser`, holds the body back until it lets the
caller in (`shared::body::hold`, with the `hold_body` middleware `app`
wraps every request in), whichever order the handler takes them in;
refused, the body is never read. A wrong `Content-Type` or an over-limit
`Content-Length` is still answered first. A guard reads the
`AppState`, so in axum it serves the hand-written routes (a
`Router<AppState>`), and in actix-web the state is app data
(`app::configure` registers it).

Outside HTTP, `shared::security::guard::require::<G>(&caller, &state,
&path)` answers the same way: the grant, or `AppError::Forbidden`, which a
gRPC service turns into `PERMISSION_DENIED` with `?`.

## A recipe: staff, members and kinds of partner

Karibu Pass sells tickets for partners: organizers that run events and
venues that host them. Its staff have a staff role, and each partner's
members have a role there. Three rules cover its routes: finance staff
only; a partner's member or staff; and a partner's member or staff, at an
organizer only.

The tables, a migration of the project's:

<!-- source: tests/gates/guards/schema.sql -->
```sql
CREATE TYPE staff_role AS ENUM ('finance', 'support');
CREATE TYPE partner_kind AS ENUM ('organizer', 'venue');
CREATE TYPE partner_role AS ENUM ('owner', 'manager', 'scanner');

CREATE TABLE staff_members (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users (id),
    role staff_role NOT NULL
);

CREATE TABLE partners (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    kind partner_kind NOT NULL
);

CREATE TABLE partner_members (
    id UUID PRIMARY KEY,
    partner_id UUID NOT NULL REFERENCES partners (id),
    user_id UUID NOT NULL REFERENCES users (id),
    role partner_role NOT NULL,
    UNIQUE (partner_id, user_id)
);
```

Each rule is backed by one read query in `queries/shared/`, which any
feature may call (`db::queries::shared::access`), so its SQL is written
once too:

<!-- source: tests/gates/guards/access.sql -->
```sql
-- name: StaffRole :maybe_one
-- The staff role of the account `$1`, if it is staff.
SELECT role
FROM staff_members
WHERE user_id = $1;

-- name: PartnerAccess :maybe_one
-- How the account `$1` may act for the partner `$2`: its role there as a
-- member, its staff role, and the partner's kind; none when there is no
-- such partner.
SELECT p.kind, m.role AS member_role, s.role AS staff_role
FROM partners p
LEFT JOIN partner_members m ON m.partner_id = p.id AND m.user_id = $1
LEFT JOIN staff_members s ON s.user_id = $1
WHERE p.id = $2;
```

The guards go in `src/shared/guards.rs` (`pub mod guards;` in
`shared/mod.rs`): code every feature needs, with the data layer under it.
A staff role is a yes or no, so it grants `()`:

<!-- source: tests/gates/guards/guards.rs -->
```rust
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
```

Access to a partner from the path is a member's role or staff, so its
grant says which. The guard is generic over a marker type for the kind of
partner a route serves, so the same query checks membership and kind
together, and `PartnerAccess` alone (the default, `AnyKind`) takes any
kind. The wrong kind is refused with its own message:

<!-- source: tests/gates/guards/guards.rs -->
```rust
/// How a caller may act for a partner: as one of its members, with their
/// role there, or as staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerGrant {
    /// A member of the partner, with this role.
    Member(PartnerRole),
    /// Staff, with this staff role.
    Staff(StaffRole),
}
```

<!-- source: tests/gates/guards/guards.rs -->
```rust
/// An organizer.
pub struct Organizer;

impl Kind for Organizer {
    const KIND: Option<PartnerKind> = Some(PartnerKind::Organizer);
    const WRONG_KIND: &'static str = "this isn't an organizer";
}
```

<!-- source: tests/gates/guards/guards.rs -->
```rust
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
```

The handlers say which rule they need, and match on the grant when it
matters:

<!-- source: tests/gates/guards/recipe.rs -->
```rust

pub async fn payouts(_: Allowed<Finance>) -> Json<Value> {
    Json(json!({"payouts": []}))
}

pub async fn rename_partner(
    Allowed(access): Allowed<PartnerAccess>,
    Json(body): Json<Rename>,
) -> Json<Value> {
    let by = match access {
        PartnerGrant::Member(role) => format!("member:{role:?}"),
        PartnerGrant::Staff(role) => format!("staff:{role:?}"),
    };
    Json(json!({"name": body.name, "by": by}))
}

pub async fn create_event(
    Allowed(_access): Allowed<PartnerAccess<Organizer>>,
    Json(body): Json<Rename>,
) -> Json<Value> {
    Json(json!({"event": body.name}))
}

```

The rok-templates gates add this recipe to a project made from the axum
template, generate it, run `check-arch` and the lint gate, and drive the
three guards over HTTP (`tests/gates/guards/`), so the code above compiles
and holds.
