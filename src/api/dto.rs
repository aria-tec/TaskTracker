use serde::{Deserialize, Serialize};

/// Request payload for creating a new task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub description: String,
}

/// Request payload for updating an existing task's description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub description: String,
}

/// Query parameters for listing and filtering tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFilterQuery {
    pub status: Option<String>,
}
