// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! [`Subscriber`]: the guard the `live` routes ask who may subscribe, and
//! to which tenant's Topics.

use uuid::Uuid;

use crate::app::AppState;
use crate::error::AppError;
use crate::shared::auth::{AuthUser, Guard};

/// Lets every signed-in caller subscribe to their own Topics: until the
/// project has tenants of its own, each account is its own tenant, so a
/// feature publishes to the user it concerns (`publish(&mut tx, user_id,
/// …)`). With organizations, answer the caller's organization here (one
/// query in `queries/shared/`), or `None` for a `403`.
pub struct Subscriber;

impl Guard for Subscriber {
    type Grant = Uuid;
    type Path = ();
    const REFUSAL: &'static str = "you can't subscribe to live updates";

    async fn check(
        caller: &AuthUser,
        _state: &AppState,
        _path: &(),
    ) -> Result<Option<Uuid>, AppError> {
        Ok(Some(caller.id))
    }
}
