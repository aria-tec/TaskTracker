use super::traits::TaskRepository;
use crate::domain::Task;
use crate::error::TaskError;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// File-based JSON implementation of `TaskRepository`.
/// Uses atomic file writes (write-and-rename) to prevent corruption.
pub struct JsonFileTaskRepository {
    file_path: PathBuf,
    lock: Mutex<()>,
}

impl JsonFileTaskRepository {
    /// Creates a repository associated with the given file path.
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            lock: Mutex::new(()),
        }
    }

    /// Creates a repository pointing to the default `./tasks.json`.
    #[must_use]
    pub fn default_path() -> Self {
        Self::new("tasks.json")
    }

    /// Reads all tasks directly from the JSON file.
    /// Creates the file with `[]` if it doesn't exist.
    fn read_tasks(&self) -> Result<Vec<Task>, TaskError> {
        if !self.file_path.exists() {
            self.ensure_file_exists()?;
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.file_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        let tasks: Vec<Task> = serde_json::from_str(trimmed)?;
        Ok(tasks)
    }

    /// Atomically writes the entire list of tasks to the JSON file.
    fn write_tasks(&self, tasks: &[Task]) -> Result<(), TaskError> {
        let parent_dir = self.file_path.parent().unwrap_or_else(|| Path::new("."));

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }

        let temp_file_path = parent_dir.join(format!(
            ".tasks_{}_{}.tmp",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        let json_bytes = serde_json::to_vec_pretty(tasks)?;

        {
            let mut temp_file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_file_path)?;

            temp_file.write_all(&json_bytes)?;
            temp_file.flush()?;
        }

        fs::rename(&temp_file_path, &self.file_path)?;
        Ok(())
    }

    /// Ensures that the tasks file exists, initializing it with `[]` if absent.
    fn ensure_file_exists(&self) -> Result<(), TaskError> {
        if let Some(parent) = self.file_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.file_path)?;

        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            file.write_all(b"[]\n")?;
            file.flush()?;
        }
        Ok(())
    }
}

impl TaskRepository for JsonFileTaskRepository {
    fn find_all(&self) -> Result<Vec<Task>, TaskError> {
        let _guard = self.lock.lock().unwrap();
        self.read_tasks()
    }

    fn find_by_id(&self, id: u64) -> Result<Option<Task>, TaskError> {
        let _guard = self.lock.lock().unwrap();
        let tasks = self.read_tasks()?;
        Ok(tasks.into_iter().find(|t| t.id == id))
    }

    fn save(&self, task: Task) -> Result<Task, TaskError> {
        let _guard = self.lock.lock().unwrap();
        let mut tasks = self.read_tasks()?;

        if let Some(index) = tasks.iter().position(|t| t.id == task.id) {
            tasks[index] = task.clone();
        } else {
            tasks.push(task.clone());
        }

        self.write_tasks(&tasks)?;
        Ok(task)
    }

    fn delete(&self, id: u64) -> Result<bool, TaskError> {
        let _guard = self.lock.lock().unwrap();
        let mut tasks = self.read_tasks()?;
        let initial_len = tasks.len();
        tasks.retain(|t| t.id != id);

        if tasks.len() < initial_len {
            self.write_tasks(&tasks)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn next_id(&self) -> Result<u64, TaskError> {
        let _guard = self.lock.lock().unwrap();
        let tasks = self.read_tasks()?;
        let max_id = tasks.iter().map(|t| t.id).max().unwrap_or(0);
        Ok(max_id + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TaskStatus;
    use tempfile::tempdir;

    #[test]
    fn test_empty_repository_initialization() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tasks.json");
        let repo = JsonFileTaskRepository::new(&file_path);

        let tasks = repo.find_all().unwrap();
        assert!(tasks.is_empty());
        assert!(file_path.exists());
    }

    #[test]
    fn test_save_and_find_tasks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tasks.json");
        let repo = JsonFileTaskRepository::new(&file_path);

        let id = repo.next_id().unwrap();
        assert_eq!(id, 1);

        let task1 = Task::new(id, "Task 1").unwrap();
        repo.save(task1.clone()).unwrap();

        let loaded = repo.find_by_id(1).unwrap();
        assert_eq!(loaded, Some(task1));

        let id2 = repo.next_id().unwrap();
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_update_existing_task() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tasks.json");
        let repo = JsonFileTaskRepository::new(&file_path);

        let mut task = Task::new(1, "Initial description").unwrap();
        repo.save(task.clone()).unwrap();

        task.update_description("Updated description").unwrap();
        task.set_status(TaskStatus::InProgress);
        repo.save(task.clone()).unwrap();

        let loaded = repo.find_by_id(1).unwrap().unwrap();
        assert_eq!(loaded.description, "Updated description");
        assert_eq!(loaded.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_delete_task() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("tasks.json");
        let repo = JsonFileTaskRepository::new(&file_path);

        let task = Task::new(1, "Task to delete").unwrap();
        repo.save(task).unwrap();

        assert!(repo.delete(1).unwrap());
        assert!(!repo.delete(1).unwrap());
        assert_eq!(repo.find_all().unwrap().len(), 0);
    }
}
