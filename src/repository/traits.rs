use crate::domain::Task;
use crate::error::TaskError;

/// Trait defining the persistence operations for `Task` entities.
pub trait TaskRepository: Send + Sync {
    /// Retrieves all tasks from storage.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` if reading fails.
    fn find_all(&self) -> Result<Vec<Task>, TaskError>;

    /// Retrieves a task by its unique ID.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` if reading fails.
    fn find_by_id(&self, id: u64) -> Result<Option<Task>, TaskError>;

    /// Saves a new or updated task to storage.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` if writing fails.
    fn save(&self, task: Task) -> Result<Task, TaskError>;

    /// Deletes a task by its ID. Returns `true` if a task was deleted, `false` if not found.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` if storage access fails.
    fn delete(&self, id: u64) -> Result<bool, TaskError>;

    /// Computes the next available unique task ID.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` if storage access fails.
    fn next_id(&self) -> Result<u64, TaskError>;
}
