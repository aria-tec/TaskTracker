# ADR-004: Axum Web REST API Design

## Status
Accepted

## Date
2026-08-17

## Context
In modern Rust backend services, [Axum](https://docs.rs/axum) is the premier asynchronous web framework, known for its ergonomic routing, extractor-based handler signatures, and modular integration with the `tokio` and `tower` ecosystems.

We needed a RESTful API layer exposing the full task management capability while maintaining state consistency with the JSON file repository.

## Decision
1. **REST Resource Routing**:
   - `GET /health` -> Liveness check
   - `POST /api/tasks` -> Create task (HTTP 201 Created)
   - `GET /api/tasks` -> List tasks (supports query parameter `?status=todo|in-progress|done`)
   - `GET /api/tasks/:id` -> Retrieve task by ID
   - `PUT /api/tasks/:id` -> Update task description
   - `PATCH /api/tasks/:id/in-progress` -> Transition status to in-progress
   - `PATCH /api/tasks/:id/done` -> Transition status to done
   - `DELETE /api/tasks/:id` -> Delete task (HTTP 204 No Content)

2. **State Sharing**:
   We leverage Axum's `State(manager)` extractor with `TaskManager` backed by `Arc<dyn TaskRepository>`. This provides lock-protected access to file storage across async worker tasks.

3. **Middleware**:
   - `tower_http::cors::CorsLayer`: Enables cross-origin requests from frontend clients.
   - `tower_http::trace::TraceLayer`: Provides structured HTTP request tracing via `tracing`.

4. **Dual Running Modes**:
   - Standalone daemon: `cargo run --bin task-server` (respects `PORT` and `TASKS_FILE` env vars).
   - CLI subcommand: `task-cli serve [port]` to immediately run the web service alongside CLI operations.

## Alternatives Considered

### Actix-web
- **Pros**: Mature, fast.
- **Cons**: Axum integrates natively with Tokio and Tower, providing superior macro-free ergonomics and alignment with modern Rust ecosystems.
- **Rejected**: Axum was explicitly chosen and requested.

## Consequences
- Full REST API functionality with standard status codes and JSON payload validation.
- Seamless compatibility with web dashboards, frontend SPAs, and microservices.
