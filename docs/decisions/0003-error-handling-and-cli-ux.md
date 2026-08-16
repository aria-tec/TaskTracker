# ADR-003: Error Handling Strategy and CLI User Experience

## Status
Accepted

## Date
2026-08-17

## Context
A command-line tool must provide clear, actionable feedback to users rather than cryptic stack traces, panics, or raw I/O errors. Furthermore, domain errors should translate intuitively to standard HTTP status codes when served via Axum.

## Decision
1. **Domain Error Enumeration (`TaskError`)**:
   We define a centralized error enum in `src/error.rs` capturing all error categories:
   - `TaskNotFound(u64)`
   - `EmptyDescription`
   - `InvalidStatus(String)`
   - `InvalidCommand(String)`
   - `InvalidArgument(String)`
   - `Storage(String)`
   - `Serialization(String)`

2. **No Panics in Business Logic**:
   All public functions return `Result<T, TaskError>`. No `.unwrap()` or `.expect()` calls are permitted in production paths.

3. **CLI Feedback & Exit Codes**:
   - Success operations output concise confirmation messages to `stdout` with exit code `0` (e.g. `Task added successfully (ID: 1)`).
   - Error conditions output human-friendly messages prefixed with `Error: ...` to `stderr` and exit with code `1`.
   - `task-cli help` or executing with no arguments displays a clear, formatted usage reference.

4. **HTTP Status Code Mapping**:
   `TaskError` implements `axum::response::IntoResponse`, mapping:
   - `TaskNotFound` -> `404 Not Found`
   - `EmptyDescription`, `InvalidStatus`, `InvalidArgument` -> `400 Bad Request`
   - `Storage`, `Serialization` -> `500 Internal Server Error`

## Alternatives Considered

### Using `anyhow::Error` Everywhere
- **Pros**: Quick to set up.
- **Cons**: Erases typed domain distinctions, making it hard for API handlers to distinguish between a 404 (Not Found) and a 400 (Bad Request).
- **Rejected**: Strongly typed errors via `TaskError` preserve domain clarity and enable clean HTTP mapping.

## Consequences
- CLI users receive clear guidance when inputs are malformed.
- API consumers receive standard JSON error responses with consistent HTTP status codes.
