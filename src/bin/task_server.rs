use std::env;
use std::sync::Arc;
use task_tracker::api::create_router;
use task_tracker::repository::JsonFileTaskRepository;
use task_tracker::service::TaskManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "task_tracker=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let storage_file = env::var("TASKS_FILE").unwrap_or_else(|_| "tasks.json".to_string());
    let repo = Arc::new(JsonFileTaskRepository::new(storage_file));
    let manager = TaskManager::new(repo);

    let app = create_router(manager);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Starting Task Tracker Axum REST API on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    tracing::info!("Received shutdown signal, terminating server gracefully...");
}
