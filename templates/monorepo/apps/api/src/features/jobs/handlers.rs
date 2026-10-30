// Scaffolded once by rok-data-sqlc; this file is yours. It is never regenerated. See docs/guide/architecture.md.

//! The `jobs` handlers: each parses the request, calls one [`JobService`]
//! method, and answers.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::dto::{JobResponse, ListJobsQuery};
use super::service::JobService;
use crate::db::jobs::models::JobId;
use crate::error::AppError;
use crate::shared::auth::AdminUser;
use crate::shared::response::{ErrorBody, PaginatedResponse};
use crate::shared::validation::Path;

/// `GET /api/v1/jobs`: one page of the jobs, newest first, `200`.
#[utoipa::path(
    get,
    path = "/api/v1/jobs",
    tag = "jobs",
    operation_id = "list",
    params(ListJobsQuery),
    responses(
        (status = 200, description = "The jobs, newest first", body = PaginatedResponse<JobResponse>),
        (status = 400, description = "A malformed query", body = ErrorBody),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 403, description = "The caller is no admin", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn list(
    State(service): State<Arc<JobService>>,
    _: AdminUser,
    Query(query): Query<ListJobsQuery>,
) -> Result<Response, AppError> {
    Ok(Json(service.list(&query).await?).into_response())
}

/// `POST /api/v1/jobs/{id}/retry`: puts a dead job back in the queue, due
/// now with its attempts at zero, `204`.
#[utoipa::path(
    post,
    path = "/api/v1/jobs/{id}/retry",
    tag = "jobs",
    operation_id = "retry",
    params(("id" = uuid::Uuid, Path, description = "Job id")),
    responses(
        (status = 204, description = "Queued again"),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 403, description = "The caller is no admin", body = ErrorBody),
        (status = 404, description = "No dead job has this id", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn retry(
    State(service): State<Arc<JobService>>,
    AdminUser(admin): AdminUser,
    Path(id): Path<JobId>,
) -> Result<Response, AppError> {
    service.retry(admin.id.into(), id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `DELETE /api/v1/jobs/{id}`: cancels a pending or dead job, `204`.
#[utoipa::path(
    delete,
    path = "/api/v1/jobs/{id}",
    tag = "jobs",
    operation_id = "cancel",
    params(("id" = uuid::Uuid, Path, description = "Job id")),
    responses(
        (status = 204, description = "Cancelled"),
        (status = 401, description = "No valid bearer token", body = ErrorBody),
        (status = 403, description = "The caller is no admin", body = ErrorBody),
        (status = 404, description = "No job has this id", body = ErrorBody),
        (status = 409, description = "A worker is running it right now", body = ErrorBody),
    ),
    security(("bearer_auth" = [])),
)]
pub async fn cancel(
    State(service): State<Arc<JobService>>,
    AdminUser(admin): AdminUser,
    Path(id): Path<JobId>,
) -> Result<Response, AppError> {
    service.cancel(admin.id.into(), id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
