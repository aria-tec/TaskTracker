#![allow(clippy::multiple_crate_versions)]

pub mod api;
pub mod domain;
pub mod error;
pub mod repository;
pub mod service;

pub use domain::{Task, TaskStatus};
pub use error::TaskError;
pub use repository::{JsonFileTaskRepository, TaskRepository};
pub use service::TaskManager;
