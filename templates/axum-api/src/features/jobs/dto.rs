// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `jobs` request and response bodies.

use chrono::{DateTime, Utc};

use crate::db::jobs::models::JobId;
use crate::db::queries::jobs::ListJobsRow;
use crate::db::types::JobStatus;
use crate::shared::pagination::Pagination;

/// `GET /jobs`'s query string.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListJobsQuery {
    /// Only the jobs with this status: `pending` (waiting, or running) or
    /// `dead` (out of attempts). Every status when absent.
    pub status: Option<JobStatus>,
    /// Only the jobs of this kind, such as `mail.send`.
    pub kind: Option<String>,
    /// The page, from 1. Defaults to 1.
    pub page: Option<u32>,
    /// The jobs a page holds, 1 to 100. Defaults to 20.
    pub per_page: Option<u32>,
}

impl ListJobsQuery {
    /// The page asked for, with the defaults and limits applied.
    #[must_use]
    pub fn pagination(&self) -> Pagination {
        Pagination::new(self.page, self.per_page)
    }

    /// The status filter, as the `jobs.status` column spells it.
    #[must_use]
    pub fn status_filter(&self) -> Option<String> {
        self.status.map(|status| {
            match status {
                JobStatus::Pending => "pending",
                JobStatus::Dead => "dead",
            }
            .to_owned()
        })
    }
}

/// A job, as an admin sees it. Its payload is left out: a pending mail's
/// holds the link it carries.
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
pub struct JobResponse {
    /// The job's id.
    #[schema(value_type = uuid::Uuid)]
    pub id: JobId,
    /// Its kind, such as `mail.send`.
    pub kind: String,
    /// `pending` (waiting, or running) or `dead` (out of attempts).
    pub status: JobStatus,
    /// The attempts it took so far.
    pub attempts: i32,
    /// When it is due: its next attempt, for a pending job.
    pub run_at: DateTime<Utc>,
    /// Why its last attempt failed, if one did.
    pub last_error: Option<String>,
    /// When it was enqueued.
    pub created_at: DateTime<Utc>,
    /// When it last changed.
    pub updated_at: DateTime<Utc>,
}

impl From<ListJobsRow> for JobResponse {
    fn from(row: ListJobsRow) -> Self {
        JobResponse {
            id: row.id,
            kind: row.kind,
            status: row.status,
            attempts: row.attempts,
            run_at: row.run_at,
            last_error: row.last_error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
