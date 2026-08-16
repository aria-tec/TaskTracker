use super::dto::{CreateTaskRequest, TaskFilterQuery, UpdateTaskRequest};
use crate::domain::{Task, TaskStatus};
use crate::error::TaskError;
use crate::service::TaskManager;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::str::FromStr;

/// Handler for checking API health.
pub async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "healthy")
}

/// Handler for creating a new task.
///
/// # Errors
/// Returns `TaskError::EmptyDescription` if description is empty, or `TaskError::Storage` on storage errors.
pub async fn create_task(
    State(manager): State<TaskManager>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), TaskError> {
    let task = manager.add_task(payload.description)?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// Handler for listing tasks, with optional status filtering.
///
/// # Errors
/// Returns `TaskError::InvalidStatus` if the query status is unknown, or `TaskError::Storage` on storage errors.
pub async fn list_tasks(
    State(manager): State<TaskManager>,
    Query(query): Query<TaskFilterQuery>,
) -> Result<Json<Vec<Task>>, TaskError> {
    let status_filter = match query.status {
        Some(s) if !s.trim().is_empty() => Some(TaskStatus::from_str(&s)?),
        _ => None,
    };

    let tasks = manager.list_tasks(status_filter)?;
    Ok(Json(tasks))
}

/// Handler for retrieving a single task by ID.
///
/// # Errors
/// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on storage errors.
pub async fn get_task(
    State(manager): State<TaskManager>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, TaskError> {
    let task = manager.get_task(id)?;
    Ok(Json(task))
}

/// Handler for updating a task's description.
///
/// # Errors
/// Returns `TaskError::TaskNotFound` if the task is not found, `TaskError::EmptyDescription` if description is empty, or `TaskError::Storage` on storage errors.
pub async fn update_task(
    State(manager): State<TaskManager>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, TaskError> {
    let task = manager.update_task(id, payload.description)?;
    Ok(Json(task))
}

/// Handler for marking a task as in progress.
///
/// # Errors
/// Returns `TaskError::TaskNotFound` if the task is not found, or `TaskError::Storage` on storage errors.
pub async fn mark_in_progress(
    State(manager): State<TaskManager>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, TaskError> {
    let task = manager.mark_in_progress(id)?;
    Ok(Json(task))
}

/// Handler for marking a task as done.
///
/// # Errors
/// Returns `TaskError::TaskNotFound` if the task is not found, or `TaskError::Storage` on storage errors.
pub async fn mark_done(
    State(manager): State<TaskManager>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, TaskError> {
    let task = manager.mark_done(id)?;
    Ok(Json(task))
}

/// Handler for deleting a task.
///
/// # Errors
/// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on storage errors.
pub async fn delete_task(
    State(manager): State<TaskManager>,
    Path(id): Path<u64>,
) -> Result<StatusCode, TaskError> {
    manager.delete_task(id)?;
    Ok(StatusCode::NO_CONTENT)
}
