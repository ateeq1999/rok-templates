// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The ports features declare for each other, implemented: a feature
//! writes another feature's data only through such a port, inside the
//! caller's transaction. Framework-free, so both templates share it.

use async_trait::async_trait;
use sqlx::PgConnection;

use crate::db::audit_log_entry::models::NewAuditLogEntry;
use crate::error::AppError;
use crate::features::audit_log_entry::AuditLogEntryService;
use crate::features::auth::service::{AuditEntry, AuditTrail};

/// `auth`'s [`AuditTrail`] port onto the `audit_log` feature: each entry is
/// written by [`AuditLogEntryService::create_in`] on the flow's own
/// connection, so it commits with the change it records.
pub struct AuditLog(pub AuditLogEntryService);

#[async_trait]
impl AuditTrail for AuditLog {
    async fn record_in(&self, conn: &mut PgConnection, entry: AuditEntry) -> Result<(), AppError> {
        self.0
            .create_in(
                conn,
                NewAuditLogEntry {
                    id: None,
                    actor_user_id: entry.actor,
                    target_user_id: entry.target,
                    action: entry.action,
                    details: None,
                },
            )
            .await?;
        Ok(())
    }
}
