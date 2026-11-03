// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The wiring, the only place the parts meet: the state the services are
//! built from ([`state`]), the services the server serves ([`routes`]),
//! the caller check in front of the API's ([`auth`]), and the server and
//! what runs beside it ([`run`]). Each part is one file; add a
//! capability's wiring to the part it belongs to.

mod auth;
mod routes;
mod run;
mod state;

pub use auth::authenticate;
pub use routes::{FILE_DESCRIPTOR_SET, routes, services};
pub use run::{run, schedule, serve, serve_as, worker};
pub use state::AppState;
