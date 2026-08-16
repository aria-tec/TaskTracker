use super::status::TaskStatus;
use crate::error::TaskError;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Represents a task entity in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Unique numeric identifier for the task.
    pub id: u64,
    /// Brief text description of what the task involves.
    pub description: String,
    /// Current completion status of the task.
    pub status: TaskStatus,
    /// ISO 8601 formatted timestamp of when the task was created.
    pub created_at: String,
    /// ISO 8601 formatted timestamp of when the task was last updated.
    pub updated_at: String,
}

impl Task {
    /// Creates a new `Task` instance in `Todo` status.
    ///
    /// # Errors
    /// Returns `TaskError::EmptyDescription` if the description contains only whitespace.
    pub fn new(id: u64, description: impl Into<String>) -> Result<Self, TaskError> {
        let trimmed_desc = Self::validate_description(description.into().as_str())?;
        let now = Utc::now().to_rfc3339();

        Ok(Self {
            id,
            description: trimmed_desc,
            status: TaskStatus::Todo,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Creates a task with explicit timestamps (useful for loading from storage or testing).
    ///
    /// # Errors
    /// Returns `TaskError::EmptyDescription` if the description contains only whitespace.
    pub fn with_timestamps(
        id: u64,
        description: impl Into<String>,
        status: TaskStatus,
        created_at: String,
        updated_at: String,
    ) -> Result<Self, TaskError> {
        let trimmed_desc = Self::validate_description(description.into().as_str())?;
        Ok(Self {
            id,
            description: trimmed_desc,
            status,
            created_at,
            updated_at,
        })
    }

    /// Updates the task description and refreshes the `updated_at` timestamp.
    ///
    /// # Errors
    /// Returns `TaskError::EmptyDescription` if the new description is empty or whitespace.
    pub fn update_description(
        &mut self,
        new_description: impl Into<String>,
    ) -> Result<(), TaskError> {
        let trimmed_desc = Self::validate_description(new_description.into().as_str())?;
        self.description = trimmed_desc;
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Sets a new status and refreshes the `updated_at` timestamp.
    pub fn set_status(&mut self, new_status: TaskStatus) {
        self.status = new_status;
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validates that the description is non-empty.
    fn validate_description(description: &str) -> Result<String, TaskError> {
        let trimmed = description.trim();
        if trimmed.is_empty() {
            Err(TaskError::EmptyDescription)
        } else {
            Ok(trimmed.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation_valid() {
        let task = Task::new(1, "Buy groceries").unwrap();
        assert_eq!(task.id, 1);
        assert_eq!(task.description, "Buy groceries");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.created_at, task.updated_at);
    }

    #[test]
    fn test_task_creation_empty_description() {
        let err = Task::new(1, "   ").unwrap_err();
        assert!(matches!(err, TaskError::EmptyDescription));
    }

    #[test]
    fn test_task_update_description() {
        let mut task = Task::new(1, "Old task").unwrap();
        task.update_description("New task").unwrap();
        assert_eq!(task.description, "New task");
    }

    #[test]
    fn test_task_status_change() {
        let mut task = Task::new(1, "In progress task").unwrap();
        task.set_status(TaskStatus::InProgress);
        assert_eq!(task.status, TaskStatus::InProgress);
        task.set_status(TaskStatus::Done);
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_task_json_serialization_camel_case() {
        let task = Task::with_timestamps(
            1,
            "Buy groceries",
            TaskStatus::Todo,
            "2026-08-17T00:00:00Z".to_string(),
            "2026-08-17T00:00:00Z".to_string(),
        )
        .unwrap();

        let json = serde_json::to_string_pretty(&task).unwrap();
        assert!(json.contains("\"createdAt\": \"2026-08-17T00:00:00Z\""));
        assert!(json.contains("\"updatedAt\": \"2026-08-17T00:00:00Z\""));
        assert!(json.contains("\"status\": \"todo\""));
    }
}
