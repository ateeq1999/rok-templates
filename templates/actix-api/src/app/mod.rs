// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The wiring, the only place the parts meet: the state every worker
//! shares ([`state`]), the application's services and routes ([`router`]),
//! the server and what runs beside it ([`run`]), and the ports features
//! declare for each other, implemented ([`ports`]). Each part is one file;
//! add a capability's wiring to the part it belongs to.

mod ports;
mod router;
mod run;
mod state;

pub use ports::AuditLog;
pub use router::configure;
pub use run::run;
pub use state::AppState;
