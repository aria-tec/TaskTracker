use axum::{
    body::{to_bytes, Body},
    http::{self, Request, StatusCode},
};
use std::sync::Arc;
use task_tracker::api::create_router;
use task_tracker::domain::{Task, TaskStatus};
use task_tracker::repository::JsonFileTaskRepository;
use task_tracker::service::TaskManager;
use tempfile::tempdir;
use tower::ServiceExt;

fn create_test_app() -> (axum::Router, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("tasks.json");
    let repo = Arc::new(JsonFileTaskRepository::new(file_path));
    let manager = TaskManager::new(repo);
    let app = create_router(manager);
    (app, dir)
}

#[tokio::test]
async fn test_health_check() {
    let (app, _dir) = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"healthy");
}

#[tokio::test]
async fn test_create_and_get_task() {
    let (app, _dir) = create_test_app();

    // 1. Create task
    let create_payload = serde_json::to_vec(&serde_json::json!({
        "description": "Write integration tests"
    }))
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/tasks")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let created_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(created_task.id, 1);
    assert_eq!(created_task.description, "Write integration tests");
    assert_eq!(created_task.status, TaskStatus::Todo);

    // 2. Get task by ID
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let fetched_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched_task, created_task);
}

#[tokio::test]
async fn test_update_and_status_transitions() {
    let (app, _dir) = create_test_app();

    // Create task
    let payload = serde_json::to_vec(&serde_json::json!({
        "description": "Initial task"
    }))
    .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/tasks")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Update description
    let update_payload = serde_json::to_vec(&serde_json::json!({
        "description": "Updated task description"
    }))
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PUT)
                .uri("/api/tasks/1")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let updated_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_task.description, "Updated task description");

    // Mark in progress
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/api/tasks/1/in-progress")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let prog_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(prog_task.status, TaskStatus::InProgress);

    // Mark done
    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/api/tasks/1/done")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let done_task: Task = serde_json::from_slice(&body).unwrap();
    assert_eq!(done_task.status, TaskStatus::Done);
}

#[tokio::test]
async fn test_list_and_filter_tasks() {
    let (app, _dir) = create_test_app();

    // Create 3 tasks
    for i in 1..=3 {
        let payload = serde_json::to_vec(&serde_json::json!({
            "description": format!("Task {i}")
        }))
        .unwrap();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/api/tasks")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/api/tasks/2/in-progress")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::PATCH)
                .uri("/api/tasks/3/done")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // List all
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let all: Vec<Task> = serde_json::from_slice(&body).unwrap();
    assert_eq!(all.len(), 3);

    // List todo
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tasks?status=todo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let todos: Vec<Task> = serde_json::from_slice(&body).unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, 1);

    // List in-progress
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tasks?status=in-progress")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let in_progs: Vec<Task> = serde_json::from_slice(&body).unwrap();
    assert_eq!(in_progs.len(), 1);
    assert_eq!(in_progs[0].id, 2);

    // List done
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tasks?status=done")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let dones: Vec<Task> = serde_json::from_slice(&body).unwrap();
    assert_eq!(dones.len(), 1);
    assert_eq!(dones[0].id, 3);
}

#[tokio::test]
async fn test_delete_task_endpoint() {
    let (app, _dir) = create_test_app();

    let payload = serde_json::to_vec(&serde_json::json!({
        "description": "To be deleted"
    }))
    .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/tasks")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri("/api/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify 404 when getting deleted task
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_error_responses() {
    let (app, _dir) = create_test_app();

    // 1. Get non-existent
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/tasks/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 2. Create task with empty description
    let payload = serde_json::to_vec(&serde_json::json!({
        "description": "   "
    }))
    .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/api/tasks")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Filter with invalid status
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tasks?status=invalid_status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
