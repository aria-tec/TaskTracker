# ADR-001: Modular Architecture for Task Tracker Core, CLI, and REST API

## Status
Accepted

## Date
2026-08-17

## Context
The Task Tracker project requires building a CLI application that stores tasks in a JSON file and performs standard lifecycle operations (adding, updating, deleting, status changes, and listing). In addition, the roadmap backend stack requires supporting modern asynchronous HTTP web services (specifically using the **Axum** framework).

We needed an architecture that:
1. Keeps business logic cleanly isolated from presentation layers (CLI and Web API).
2. Adheres to Uncle Bob's Clean Code principles (Single Responsibility Principle, Dependency Inversion, Intention-Revealing interfaces).
3. Allows both the CLI (`task-cli`) and the Web API (`task-server`) to share the same domain and storage logic without code duplication.

## Decision
We adopted a layered domain-driven architecture structured as a Rust library crate (`task_tracker`) and two binary entry points:

1. **`domain` Layer**: Contains pure domain models (`Task`, `TaskStatus`) and business invariants (e.g. non-empty description validation, timestamp updates).
2. **`repository` Layer**: Defines the `TaskRepository` trait and the `JsonFileTaskRepository` implementation, isolating file system operations from business rules.
3. **`service` Layer**: Houses `TaskManager`, orchestrating use cases (`add_task`, `update_task`, `delete_task`, `mark_in_progress`, `mark_done`, `list_tasks`).
4. **`api` Layer**: Encapsulates Axum routing, DTO request/response representations, and HTTP handler functions.
5. **Binaries**:
   - `task-cli` (`src/bin/task_cli.rs`): Fast command-line interface accepting positional arguments.
   - `task-server` (`src/bin/task_server.rs`): Dedicated Axum HTTP REST server.

## Alternatives Considered

### Single Monolithic CLI File
- **Pros**: Easy to write initially in a single `main.rs`.
- **Cons**: High coupling between CLI argument parsing, file reading/writing, and business logic. Difficult to test or reuse for an Axum web server.
- **Rejected**: Violates SRP and Clean Code guidelines.

### Separate Micro-crates in a Workspace
- **Pros**: Strict boundary enforcement.
- **Cons**: Over-engineering for a task tracker; increases build and maintenance overhead.
- **Rejected**: A modular single-crate structure with clear module boundaries (`domain`, `repository`, `service`, `api`) is the most maintainable and ergonomic approach.

## Consequences
- Clean separation of concerns makes unit and integration testing fast and isolated.
- Future storage backends (e.g., SQLite, PostgreSQL) can be added simply by implementing `TaskRepository`.
- Both CLI and HTTP API share identical business rules and validation logic.
