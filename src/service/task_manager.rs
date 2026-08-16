use crate::domain::{Task, TaskStatus};
use crate::error::TaskError;
use crate::repository::TaskRepository;
use std::sync::Arc;

/// Application service coordinating task-related business workflows.
#[derive(Clone)]
pub struct TaskManager {
    repository: Arc<dyn TaskRepository>,
}

impl TaskManager {
    /// Creates a new `TaskManager` with the supplied repository implementation.
    #[must_use]
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }

    /// Adds a new task with the specified description.
    ///
    /// # Errors
    /// Returns `TaskError::EmptyDescription` if description is empty, or `TaskError::Storage` on I/O failure.
    pub fn add_task(&self, description: impl Into<String>) -> Result<Task, TaskError> {
        let next_id = self.repository.next_id()?;
        let task = Task::new(next_id, description)?;
        self.repository.save(task)
    }

    /// Updates the description of an existing task.
    ///
    /// # Errors
    /// Returns `TaskError::TaskNotFound` if the task does not exist, `TaskError::EmptyDescription` if description is empty, or `TaskError::Storage` on I/O failure.
    pub fn update_task(&self, id: u64, description: impl Into<String>) -> Result<Task, TaskError> {
        let mut task = self
            .repository
            .find_by_id(id)?
            .ok_or(TaskError::TaskNotFound(id))?;

        task.update_description(description)?;
        self.repository.save(task)
    }

    /// Deletes a task by ID.
    ///
    /// # Errors
    /// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on I/O failure.
    pub fn delete_task(&self, id: u64) -> Result<(), TaskError> {
        let deleted = self.repository.delete(id)?;
        if deleted {
            Ok(())
        } else {
            Err(TaskError::TaskNotFound(id))
        }
    }

    /// Updates the status of a task to `InProgress`.
    ///
    /// # Errors
    /// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on I/O failure.
    pub fn mark_in_progress(&self, id: u64) -> Result<Task, TaskError> {
        let mut task = self
            .repository
            .find_by_id(id)?
            .ok_or(TaskError::TaskNotFound(id))?;

        task.set_status(TaskStatus::InProgress);
        self.repository.save(task)
    }

    /// Updates the status of a task to `Done`.
    ///
    /// # Errors
    /// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on I/O failure.
    pub fn mark_done(&self, id: u64) -> Result<Task, TaskError> {
        let mut task = self
            .repository
            .find_by_id(id)?
            .ok_or(TaskError::TaskNotFound(id))?;

        task.set_status(TaskStatus::Done);
        self.repository.save(task)
    }

    /// Retrieves a single task by ID.
    ///
    /// # Errors
    /// Returns `TaskError::TaskNotFound` if the task does not exist, or `TaskError::Storage` on I/O failure.
    pub fn get_task(&self, id: u64) -> Result<Task, TaskError> {
        self.repository
            .find_by_id(id)?
            .ok_or(TaskError::TaskNotFound(id))
    }

    /// Lists all tasks, optionally filtered by a status.
    ///
    /// # Errors
    /// Returns `TaskError::Storage` or `TaskError::Serialization` on I/O failure.
    pub fn list_tasks(&self, status_filter: Option<TaskStatus>) -> Result<Vec<Task>, TaskError> {
        let all_tasks = self.repository.find_all()?;
        match status_filter {
            Some(status) => Ok(all_tasks
                .into_iter()
                .filter(|t| t.status == status)
                .collect()),
            None => Ok(all_tasks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::JsonFileTaskRepository;
    use tempfile::tempdir;

    fn setup_manager() -> (TaskManager, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tasks.json");
        let repo = Arc::new(JsonFileTaskRepository::new(file_path));
        (TaskManager::new(repo), dir)
    }

    #[test]
    fn test_add_and_get_task() {
        let (manager, _dir) = setup_manager();
        let task = manager.add_task("Buy milk").unwrap();
        assert_eq!(task.id, 1);
        assert_eq!(task.description, "Buy milk");
        assert_eq!(task.status, TaskStatus::Todo);

        let fetched = manager.get_task(1).unwrap();
        assert_eq!(fetched, task);
    }

    #[test]
    fn test_update_task() {
        let (manager, _dir) = setup_manager();
        manager.add_task("Initial task").unwrap();

        let updated = manager.update_task(1, "Updated task").unwrap();
        assert_eq!(updated.description, "Updated task");
    }

    #[test]
    fn test_update_non_existent_task() {
        let (manager, _dir) = setup_manager();
        let err = manager.update_task(99, "Updated task").unwrap_err();
        assert!(matches!(err, TaskError::TaskNotFound(99)));
    }

    #[test]
    fn test_delete_task() {
        let (manager, _dir) = setup_manager();
        manager.add_task("Task to delete").unwrap();
        manager.delete_task(1).unwrap();

        let err = manager.get_task(1).unwrap_err();
        assert!(matches!(err, TaskError::TaskNotFound(1)));
    }

    #[test]
    fn test_mark_in_progress_and_done() {
        let (manager, _dir) = setup_manager();
        manager.add_task("Flow task").unwrap();

        let in_progress = manager.mark_in_progress(1).unwrap();
        assert_eq!(in_progress.status, TaskStatus::InProgress);

        let done = manager.mark_done(1).unwrap();
        assert_eq!(done.status, TaskStatus::Done);
    }

    #[test]
    fn test_list_tasks_with_filtering() {
        let (manager, _dir) = setup_manager();
        manager.add_task("Task 1").unwrap();
        manager.add_task("Task 2").unwrap();
        manager.add_task("Task 3").unwrap();

        manager.mark_in_progress(2).unwrap();
        manager.mark_done(3).unwrap();

        let all = manager.list_tasks(None).unwrap();
        assert_eq!(all.len(), 3);

        let todo = manager.list_tasks(Some(TaskStatus::Todo)).unwrap();
        assert_eq!(todo.len(), 1);
        assert_eq!(todo[0].id, 1);

        let in_prog = manager.list_tasks(Some(TaskStatus::InProgress)).unwrap();
        assert_eq!(in_prog.len(), 1);
        assert_eq!(in_prog[0].id, 2);

        let done = manager.list_tasks(Some(TaskStatus::Done)).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].id, 3);
    }
}
