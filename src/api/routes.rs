use super::handlers;
use crate::service::TaskManager;
use axum::{
    routing::{get, patch, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Builds and configures the Axum application router.
pub fn create_router(manager: TaskManager) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route(
            "/api/tasks",
            post(handlers::create_task).get(handlers::list_tasks),
        )
        .route(
            "/api/tasks/{id}",
            get(handlers::get_task)
                .put(handlers::update_task)
                .delete(handlers::delete_task),
        )
        .route(
            "/api/tasks/{id}/in-progress",
            patch(handlers::mark_in_progress),
        )
        .route("/api/tasks/{id}/done", patch(handlers::mark_done))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(manager)
}
