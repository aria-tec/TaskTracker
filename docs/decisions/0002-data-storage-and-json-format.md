# ADR-002: Data Storage and Atomic JSON Persistence

## Status
Accepted

## Date
2026-08-17

## Context
The Task Tracker specification requires persisting tasks in a JSON file (`tasks.json`) located in the current working directory. The file must be automatically initialized if it does not exist.

File-based JSON storage introduces two main reliability concerns:
1. **File corruption on unexpected termination**: If the process is terminated mid-write, partial JSON content will cause syntax errors on subsequent reads.
2. **Concurrent access**: Multiple operations running concurrently or across threads should avoid race conditions or dirty reads.

## Decision
1. **Schema Definition**:
   Each task in JSON conforms to the exact specification:
   ```json
   {
     "id": 1,
     "description": "Buy groceries and cook dinner",
     "status": "in-progress",
     "createdAt": "2026-08-17T00:00:00+00:00",
     "updatedAt": "2026-08-17T00:10:00+00:00"
   }
   ```
   CamelCase keys (`createdAt`, `updatedAt`) and kebab-case status values (`todo`, `in-progress`, `done`) are strictly serialized using Serde attributes.

2. **Atomic Write Strategy (Write-and-Rename)**:
   When modifying tasks, the repository serializes the updated list into a temporary hidden file in the same directory (`.tasks_<pid>_<timestamp>.tmp`) and then atomically replaces the destination `tasks.json` using `std::fs::rename`. This guarantees atomic file replacement on POSIX systems.

3. **In-Memory Concurrency Protection**:
   `JsonFileTaskRepository` protects file I/O operations using a `std::sync::Mutex<()>`, ensuring thread-safe access within the process.

## Alternatives Considered

### Direct File Overwrite (`OpenOptions::truncate`)
- **Pros**: Slightly simpler code.
- **Cons**: High risk of leaving a truncated or corrupted JSON file if interrupted by SIGINT / power failure.
- **Rejected**: Fails robustness requirements.

### Append-Only Transaction Log (WAL)
- **Pros**: High write throughput.
- **Cons**: Excessive complexity for a task tracker; requires log compaction and reconciliation on startup.
- **Rejected**: Atomic full-state snapshots via write-and-rename are optimal for this data size.

## Consequences
- Zero risk of leaving corrupted partial JSON files.
- `tasks.json` is always pretty-printed and human-readable.
- Path is configurable for isolated unit and integration testing.
