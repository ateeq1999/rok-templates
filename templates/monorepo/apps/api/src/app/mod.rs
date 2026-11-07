// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The wiring, the only place the parts meet: the state every handler
//! reaches ([`state`]), the router ([`router`]) and the layers it applies
//! ([`middleware`]), the server and what runs beside it ([`run`]), and the
//! ports features declare for each other, implemented ([`ports`]). Each
//! part is one file; add a capability's wiring to the part it belongs to.
//! [`seed`] makes local accounts to sign in with (`cargo run --bin seed`).

pub mod middleware;
mod ports;
mod router;
mod run;
pub mod seed;
mod state;

pub use ports::AuditLog;
pub use router::{health_routes, router, time_out};
pub use run::{run, schedule, serve, serve_as, worker};
pub use state::AppState;
