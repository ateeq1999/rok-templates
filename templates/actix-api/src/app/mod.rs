// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The wiring, the only place the parts meet: the state every worker
//! shares ([`state`]), the application, its services and routes ([`router`])
//! and the middleware around them ([`middleware`]),
//! the server and what runs beside it ([`run`]), and the ports features
//! declare for each other, implemented ([`ports`]). Each part is one file;
//! add a capability's wiring to the part it belongs to.

pub mod middleware;
mod ports;
mod router;
mod run;
mod state;

pub use ports::AuditLog;
pub use router::{app, configure};
pub use run::{run, schedule, worker};
pub use state::AppState;
