# Task Tracker CLI & Axum REST API in Rust

A robust, production-grade task tracking application written in Rust, featuring both a fast Command-Line Interface (`task-cli`) and an asynchronous REST API service built with [Axum](https://docs.rs/axum) (`task-server`). All tasks are stored persistently in a human-readable JSON file (`tasks.json`) with atomic write guarantees.

> **Roadmap.sh Project Reference**: https://roadmap.sh/projects/task-tracker

---

## Features

- **Full Lifecycle Management**: Add, update, delete, and transition task statuses (`todo`, `in-progress`, `done`).
- **Positional Argument CLI**: Fast, ergonomic command-line interface following POSIX standards.
- **RESTful Axum HTTP API**: Full-featured web service with CORS, tracing, and query filtering.
- **Atomic File Storage**: Safe JSON persistence using atomic write-and-rename to prevent file corruption.
- **Auto-Initialization**: Creates `tasks.json` automatically on first run.
- **Clean Architecture & ADRs**: Designed following Uncle Bob's Clean Code principles, with Architecture Decision Records in `docs/decisions/`.

---

## Quick Start

### 1. Prerequisites
Ensure you have Rust (edition 2021 or newer) and Cargo installed:
```bash
rustc --version
cargo --version
```

### 2. Build the Binaries
```bash
cargo build --release
```
The compiled binaries will be available at:
- `target/release/task-cli`
- `target/release/task-server`

---

## Command Line Interface (`task-cli`)

### CLI Commands Reference

| Command | Description | Example |
|---|---|---|
| `add "<description>"` | Adds a new task | `task-cli add "Buy groceries"` |
| `update <id> "<description>"` | Updates an existing task's description | `task-cli update 1 "Buy groceries and cook dinner"` |
| `delete <id>` | Deletes a task by ID | `task-cli delete 1` |
| `mark-in-progress <id>` | Marks task status as `in-progress` | `task-cli mark-in-progress 1` |
| `mark-done <id>` | Marks task status as `done` | `task-cli mark-done 1` |
| `list` | Lists all tasks in a formatted table | `task-cli list` |
| `list todo` | Lists pending tasks (`todo`) | `task-cli list todo` |
| `list in-progress` | Lists active tasks (`in-progress`) | `task-cli list in-progress` |
| `list done` | Lists completed tasks (`done`) | `task-cli list done` |
| `serve [port]` | Starts the Axum REST API server (default: `3000`) | `task-cli serve 8080` |
| `help` | Prints CLI usage manual | `task-cli help` |

### CLI Usage Examples

```bash
# 1. Add a task
cargo run --bin task-cli -- add "Buy groceries"
# Output: Task added successfully (ID: 1)

# 2. Update task description
cargo run --bin task-cli -- update 1 "Buy groceries and cook dinner"
# Output: Task updated successfully (ID: 1)

# 3. Mark task as in progress
cargo run --bin task-cli -- mark-in-progress 1
# Output: Task marked as in progress (ID: 1)

# 4. List all tasks
cargo run --bin task-cli -- list

# 5. Filter tasks by status
cargo run --bin task-cli -- list in-progress
cargo run --bin task-cli -- list done
cargo run --bin task-cli -- list todo

# 6. Mark task as done
cargo run --bin task-cli -- mark-done 1
# Output: Task marked as done (ID: 1)

# 7. Delete task
cargo run --bin task-cli -- delete 1
# Output: Task deleted successfully (ID: 1)
```

---

## Axum REST API (`task-server`)

You can launch the web API server either using the dedicated binary:
```bash
cargo run --bin task-server
```
or via the CLI subcommand:
```bash
cargo run --bin task-cli -- serve 3000
```

### Environment Variables
- `PORT`: HTTP port to bind (default: `3000`).
- `TASKS_FILE`: Custom path to the JSON storage file (default: `tasks.json`).
- `RUST_LOG`: Tracing log filter (e.g. `task_tracker=debug,tower_http=info`).

### REST API Endpoints

| Method | Endpoint | Description | Status Code |
|---|---|---|---|
| `GET` | `/health` | Server health check | `200 OK` |
| `POST` | `/api/tasks` | Create a new task | `201 Created` |
| `GET` | `/api/tasks` | List all tasks (optional `?status=...`) | `200 OK` |
| `GET` | `/api/tasks/:id` | Get task by ID | `200 OK` / `404 Not Found` |
| `PUT` | `/api/tasks/:id` | Update task description | `200 OK` / `404 Not Found` |
| `PATCH` | `/api/tasks/:id/in-progress` | Mark task as in-progress | `200 OK` / `404 Not Found` |
| `PATCH` | `/api/tasks/:id/done` | Mark task as done | `200 OK` / `404 Not Found` |
| `DELETE` | `/api/tasks/:id` | Delete task | `204 No Content` / `404 Not Found` |

### API Examples with `curl`

#### 1. Create a Task
```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"description": "Learn Rust and Axum"}'
```
Response (`201 Created`):
```json
{
  "id": 1,
  "description": "Learn Rust and Axum",
  "status": "todo",
  "createdAt": "2026-08-17T00:34:44Z",
  "updatedAt": "2026-08-17T00:34:44Z"
}
```

#### 2. List Tasks (with optional filter)
```bash
# List all tasks
curl http://localhost:3000/api/tasks

# Filter by status
curl http://localhost:3000/api/tasks?status=in-progress
```

#### 3. Mark Task In Progress / Done
```bash
# In Progress
curl -X PATCH http://localhost:3000/api/tasks/1/in-progress

# Done
curl -X PATCH http://localhost:3000/api/tasks/1/done
```

#### 4. Update Task Description
```bash
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"description": "Build high-performance microservices in Rust"}'
```

#### 5. Delete Task
```bash
curl -X DELETE http://localhost:3000/api/tasks/1
```

---

## JSON Storage Schema

Tasks are stored in `tasks.json` conforming to the roadmap.sh specification:
```json
[
  {
    "id": 1,
    "description": "Buy groceries and cook dinner",
    "status": "in-progress",
    "createdAt": "2026-08-17T00:00:00Z",
    "updatedAt": "2026-08-17T00:10:00Z"
  }
]
```

### Property Specifications
- `id`: Unique unsigned integer.
- `description`: Non-empty text describing the task.
- `status`: One of `"todo"`, `"in-progress"`, `"done"`.
- `createdAt`: ISO 8601 UTC timestamp of creation.
- `updatedAt`: ISO 8601 UTC timestamp of the most recent update.

---

## Project Architecture

```
roadmap-backend-rust/
├── Cargo.toml
├── docs/
│   └── decisions/
│       ├── 0001-task-tracker-architecture.md
│       ├── 0002-data-storage-and-json-format.md
│       ├── 0003-error-handling-and-cli-ux.md
│       └── 0004-axum-web-api-design.md
├── src/
│   ├── lib.rs              # Library root exporting domain, repository, service, api
│   ├── domain/             # Core entities, value objects & invariants
│   │   ├── mod.rs
│   │   ├── task.rs         # Task struct & validation
│   │   └── status.rs       # TaskStatus enum
│   ├── repository/         # Storage layer abstraction
│   │   ├── mod.rs
│   │   ├── traits.rs       # TaskRepository trait
│   │   └── json_file.rs    # Atomic JSON file repository
│   ├── service/            # Business application service
│   │   ├── mod.rs
│   │   └── task_manager.rs # TaskManager use-case coordinator
│   ├── api/                # Axum HTTP handlers, routes & DTOs
│   │   ├── mod.rs
│   │   ├── dto.rs
│   │   ├── handlers.rs
│   │   └── routes.rs
│   ├── error.rs            # Centralized typed error definitions
│   └── bin/
│       ├── task_cli.rs     # task-cli CLI binary
│       └── task_server.rs  # task-server Axum HTTP binary
└── tests/
    ├── cli_integration_test.rs # CLI end-to-end integration tests
    └── api_integration_test.rs # Axum REST API integration tests
```

### Architecture Decision Records (ADRs)
- [ADR-001: Modular Architecture for Task Tracker Core, CLI, and REST API](file:///Users/arias/Documents/antigravity/roadmap-backend-rust/docs/decisions/0001-task-tracker-architecture.md)
- [ADR-002: Data Storage and Atomic JSON Persistence](file:///Users/arias/Documents/antigravity/roadmap-backend-rust/docs/decisions/0002-data-storage-and-json-format.md)
- [ADR-003: Error Handling Strategy and CLI User Experience](file:///Users/arias/Documents/antigravity/roadmap-backend-rust/docs/decisions/0003-error-handling-and-cli-ux.md)
- [ADR-004: Axum Web REST API Design](file:///Users/arias/Documents/antigravity/roadmap-backend-rust/docs/decisions/0004-axum-web-api-design.md)

---

## Testing & Quality Assurance

Run the comprehensive test suite (unit + integration tests):
```bash
cargo test
```

Run linter checks with zero warnings:
```bash
cargo clippy --all-targets -- -D warnings
```

---

## License
MIT License
