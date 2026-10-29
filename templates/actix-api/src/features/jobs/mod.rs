// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `jobs` feature: the job queue, for admins. `GET /api/v1/jobs` lists
//! jobs by status and kind, `POST /api/v1/jobs/{id}/retry` puts a dead one
//! back in the queue, `DELETE /api/v1/jobs/{id}` cancels one; the last two
//! are recorded in the audit log. Hand-written in the generated features'
//! layout; `check-arch` applies to it in full. Its SQL is `queries/jobs.sql`
//! (`db::queries::jobs`); running the jobs is `shared/jobs/`'s.

/// Request and response bodies.
pub mod dto;
/// The handlers: parse, call one service method, answer.
pub mod handlers;
/// The URL to handler mapping.
pub mod routes;
/// `JobService`: what an admin does with the queue.
pub mod service;

pub use routes::routes;
pub use service::{JobAudit, JobAuditEntry, JobService};
