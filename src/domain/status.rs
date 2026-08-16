use crate::error::TaskError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Represents the lifecycle status of a Task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    /// The task is pending and has not been started.
    Todo,
    /// The task is currently actively being worked on.
    InProgress,
    /// The task has been successfully completed.
    Done,
}

impl TaskStatus {
    /// Returns the canonical string slice representation of the status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::InProgress => "in-progress",
            Self::Done => "done",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TaskStatus {
    type Err = TaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "todo" => Ok(Self::Todo),
            "in-progress" | "inprogress" | "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            other => Err(TaskError::InvalidStatus(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_string() {
        assert_eq!(TaskStatus::Todo.to_string(), "todo");
        assert_eq!(TaskStatus::InProgress.to_string(), "in-progress");
        assert_eq!(TaskStatus::Done.to_string(), "done");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!("todo".parse::<TaskStatus>().unwrap(), TaskStatus::Todo);
        assert_eq!(
            "in-progress".parse::<TaskStatus>().unwrap(),
            TaskStatus::InProgress
        );
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert!("invalid".parse::<TaskStatus>().is_err());
    }

    #[test]
    fn test_status_serde() {
        let serialized = serde_json::to_string(&TaskStatus::InProgress).unwrap();
        assert_eq!(serialized, "\"in-progress\"");
        let deserialized: TaskStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, TaskStatus::InProgress);
    }
}
