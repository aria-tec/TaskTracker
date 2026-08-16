use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_add_task_success() {
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("task-cli").unwrap();
    cmd.current_dir(&temp_dir)
        .arg("add")
        .arg("Buy groceries")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task added successfully (ID: 1)"));

    let json_content = fs::read_to_string(temp_dir.path().join("tasks.json")).unwrap();
    assert!(json_content.contains("\"description\": \"Buy groceries\""));
    assert!(json_content.contains("\"status\": \"todo\""));
    assert!(json_content.contains("\"id\": 1"));
    assert!(json_content.contains("\"createdAt\":"));
    assert!(json_content.contains("\"updatedAt\":"));
}

#[test]
fn test_cli_update_task_success() {
    let temp_dir = tempdir().unwrap();

    // 1. Add task
    let mut cmd_add = Command::cargo_bin("task-cli").unwrap();
    cmd_add
        .current_dir(&temp_dir)
        .arg("add")
        .arg("Buy groceries")
        .assert()
        .success();

    // 2. Update task
    let mut cmd_update = Command::cargo_bin("task-cli").unwrap();
    cmd_update
        .current_dir(&temp_dir)
        .arg("update")
        .arg("1")
        .arg("Buy groceries and cook dinner")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Task updated successfully (ID: 1)",
        ));

    let json_content = fs::read_to_string(temp_dir.path().join("tasks.json")).unwrap();
    assert!(json_content.contains("Buy groceries and cook dinner"));
}

#[test]
fn test_cli_mark_in_progress_and_done() {
    let temp_dir = tempdir().unwrap();

    let mut cmd_add = Command::cargo_bin("task-cli").unwrap();
    cmd_add
        .current_dir(&temp_dir)
        .arg("add")
        .arg("Learn Rust")
        .assert()
        .success();

    // Mark in progress
    let mut cmd_progress = Command::cargo_bin("task-cli").unwrap();
    cmd_progress
        .current_dir(&temp_dir)
        .arg("mark-in-progress")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Task marked as in progress (ID: 1)",
        ));

    let json_after_progress = fs::read_to_string(temp_dir.path().join("tasks.json")).unwrap();
    assert!(json_after_progress.contains("\"status\": \"in-progress\""));

    // Mark done
    let mut cmd_done = Command::cargo_bin("task-cli").unwrap();
    cmd_done
        .current_dir(&temp_dir)
        .arg("mark-done")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task marked as done (ID: 1)"));

    let json_after_done = fs::read_to_string(temp_dir.path().join("tasks.json")).unwrap();
    assert!(json_after_done.contains("\"status\": \"done\""));
}

#[test]
fn test_cli_delete_task() {
    let temp_dir = tempdir().unwrap();

    let mut cmd_add = Command::cargo_bin("task-cli").unwrap();
    cmd_add
        .current_dir(&temp_dir)
        .arg("add")
        .arg("Task to be deleted")
        .assert()
        .success();

    let mut cmd_del = Command::cargo_bin("task-cli").unwrap();
    cmd_del
        .current_dir(&temp_dir)
        .arg("delete")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Task deleted successfully (ID: 1)",
        ));

    let mut cmd_list = Command::cargo_bin("task-cli").unwrap();
    cmd_list
        .current_dir(&temp_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks found."));
}

#[test]
fn test_cli_list_filtering() {
    let temp_dir = tempdir().unwrap();

    // Add 3 tasks
    for desc in &["Task 1 Todo", "Task 2 InProg", "Task 3 Done"] {
        let mut cmd = Command::cargo_bin("task-cli").unwrap();
        cmd.current_dir(&temp_dir)
            .arg("add")
            .arg(desc)
            .assert()
            .success();
    }

    // Set statuses
    let mut cmd_p = Command::cargo_bin("task-cli").unwrap();
    cmd_p
        .current_dir(&temp_dir)
        .arg("mark-in-progress")
        .arg("2")
        .assert()
        .success();

    let mut cmd_d = Command::cargo_bin("task-cli").unwrap();
    cmd_d
        .current_dir(&temp_dir)
        .arg("mark-done")
        .arg("3")
        .assert()
        .success();

    // List all
    let mut cmd_list_all = Command::cargo_bin("task-cli").unwrap();
    cmd_list_all
        .current_dir(&temp_dir)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1 Todo"))
        .stdout(predicate::str::contains("Task 2 InProg"))
        .stdout(predicate::str::contains("Task 3 Done"));

    // List todo
    let mut cmd_list_todo = Command::cargo_bin("task-cli").unwrap();
    cmd_list_todo
        .current_dir(&temp_dir)
        .arg("list")
        .arg("todo")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 1 Todo"))
        .stdout(predicate::str::contains("Task 2 InProg").not());

    // List in-progress
    let mut cmd_list_inprog = Command::cargo_bin("task-cli").unwrap();
    cmd_list_inprog
        .current_dir(&temp_dir)
        .arg("list")
        .arg("in-progress")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 2 InProg"))
        .stdout(predicate::str::contains("Task 1 Todo").not());

    // List done
    let mut cmd_list_done = Command::cargo_bin("task-cli").unwrap();
    cmd_list_done
        .current_dir(&temp_dir)
        .arg("list")
        .arg("done")
        .assert()
        .success()
        .stdout(predicate::str::contains("Task 3 Done"))
        .stdout(predicate::str::contains("Task 1 Todo").not());
}

#[test]
fn test_cli_error_handling() {
    let temp_dir = tempdir().unwrap();

    // Missing description on add
    let mut cmd_add_empty = Command::cargo_bin("task-cli").unwrap();
    cmd_add_empty
        .current_dir(&temp_dir)
        .arg("add")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing task description"));

    // Non-existent task update
    let mut cmd_up_nonexistent = Command::cargo_bin("task-cli").unwrap();
    cmd_up_nonexistent
        .current_dir(&temp_dir)
        .arg("update")
        .arg("999")
        .arg("New title")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Task not found with ID: 999"));

    // Invalid ID format
    let mut cmd_invalid_id = Command::cargo_bin("task-cli").unwrap();
    cmd_invalid_id
        .current_dir(&temp_dir)
        .arg("delete")
        .arg("not-a-number")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid task ID"));

    // Unknown command
    let mut cmd_unknown = Command::cargo_bin("task-cli").unwrap();
    cmd_unknown
        .current_dir(&temp_dir)
        .arg("unknown-cmd")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown command"));
}
