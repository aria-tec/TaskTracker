use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// Custom error types for the Task Tracker application.
#[derive(Debug)]
pub enum TaskError {
    /// A task with the specified ID was not found.
    TaskNotFound(u64),
    /// The description provided for a task is empty or whitespace only.
    EmptyDescription,
    /// An invalid status string was provided.
    InvalidStatus(String),
    /// An unknown or malformed CLI command was supplied.
    InvalidCommand(String),
    /// Invalid arguments were supplied to a command.
    InvalidArgument(String),
    /// Storage / File I/O failure.
    Storage(String),
    /// JSON serialization or deserialization failure.
    Serialization(String),
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotFound(id) => write!(f, "Task not found with ID: {id}"),
            Self::EmptyDescription => write!(f, "Task description cannot be empty"),
            Self::InvalidStatus(s) => write!(
                f,
                "Invalid status '{s}'. Allowed values are: 'todo', 'in-progress', 'done'"
            ),
            Self::InvalidCommand(cmd) => write!(
                f,
                "Unknown command '{cmd}'. Run with 'help' for usage instructions."
            ),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {msg}"),
            Self::Storage(msg) => write!(f, "Storage error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
        }
    }
}

impl std::error::Error for TaskError {}

impl From<std::io::Error> for TaskError {
    fn from(err: std::io::Error) -> Self {
        Self::Storage(err.to_string())
    }
}

impl From<serde_json::Error> for TaskError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl IntoResponse for TaskError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::TaskNotFound(_) | Self::InvalidCommand(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::EmptyDescription | Self::InvalidStatus(_) | Self::InvalidArgument(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            Self::Storage(_) | Self::Serialization(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
