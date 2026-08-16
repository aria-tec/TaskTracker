use std::env;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use task_tracker::api::create_router;
use task_tracker::domain::{Task, TaskStatus};
use task_tracker::error::TaskError;
use task_tracker::repository::JsonFileTaskRepository;
use task_tracker::service::TaskManager;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let repo = Arc::new(JsonFileTaskRepository::default_path());
    let manager = TaskManager::new(repo);

    if let Err(err) = run_cli(&args, &manager).await {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

/// Executes the CLI command based on positional arguments.
async fn run_cli(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "add" => handle_add(args, manager),
        "update" => handle_update(args, manager),
        "delete" => handle_delete(args, manager),
        "mark-in-progress" | "mark_in_progress" => handle_mark_in_progress(args, manager),
        "mark-done" | "mark_done" => handle_mark_done(args, manager),
        "list" => handle_list(args, manager),
        "serve" => handle_serve(args, manager).await,
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        unknown => Err(TaskError::InvalidCommand(unknown.to_string())),
    }
}

fn handle_add(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 3 {
        return Err(TaskError::InvalidArgument(
            "Missing task description. Usage: task-cli add \"<description>\"".to_string(),
        ));
    }
    let task = manager.add_task(&args[2])?;
    println!("Task added successfully (ID: {})", task.id);
    Ok(())
}

fn handle_update(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 3 {
        return Err(TaskError::InvalidArgument(
            "Missing task ID. Usage: task-cli update <id> \"<description>\"".to_string(),
        ));
    }
    if args.len() < 4 {
        return Err(TaskError::InvalidArgument(
            "Missing task description. Usage: task-cli update <id> \"<description>\"".to_string(),
        ));
    }
    let id = parse_id(&args[2])?;
    let task = manager.update_task(id, &args[3])?;
    println!("Task updated successfully (ID: {})", task.id);
    Ok(())
}

fn handle_delete(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 3 {
        return Err(TaskError::InvalidArgument(
            "Missing task ID. Usage: task-cli delete <id>".to_string(),
        ));
    }
    let id = parse_id(&args[2])?;
    manager.delete_task(id)?;
    println!("Task deleted successfully (ID: {id})");
    Ok(())
}

fn handle_mark_in_progress(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 3 {
        return Err(TaskError::InvalidArgument(
            "Missing task ID. Usage: task-cli mark-in-progress <id>".to_string(),
        ));
    }
    let id = parse_id(&args[2])?;
    let task = manager.mark_in_progress(id)?;
    println!("Task marked as in progress (ID: {})", task.id);
    Ok(())
}

fn handle_mark_done(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    if args.len() < 3 {
        return Err(TaskError::InvalidArgument(
            "Missing task ID. Usage: task-cli mark-done <id>".to_string(),
        ));
    }
    let id = parse_id(&args[2])?;
    let task = manager.mark_done(id)?;
    println!("Task marked as done (ID: {})", task.id);
    Ok(())
}

fn handle_list(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    let filter = if args.len() >= 3 {
        Some(TaskStatus::from_str(&args[2])?)
    } else {
        None
    };

    let tasks = manager.list_tasks(filter)?;
    print_tasks(&tasks);
    Ok(())
}

async fn handle_serve(args: &[String], manager: &TaskManager) -> Result<(), TaskError> {
    let port = if args.len() >= 3 {
        args[2]
            .parse::<u16>()
            .map_err(|_| TaskError::InvalidArgument(format!("Invalid port number '{}'", args[2])))?
    } else {
        3000
    };

    let app = create_router(manager.clone());
    let addr = format!("0.0.0.0:{port}");
    println!("Starting Task Tracker Axum API server on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| TaskError::Storage(format!("Failed to bind to address {addr}: {e}")))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| TaskError::Storage(format!("Server error: {e}")))
}

/// Parses a string into a numeric task ID.
fn parse_id(s: &str) -> Result<u64, TaskError> {
    s.parse::<u64>().map_err(|_| {
        TaskError::InvalidArgument(format!(
            "Invalid task ID '{s}'. ID must be a positive integer."
        ))
    })
}

/// Formats and displays tasks in a readable tabular representation.
fn print_tasks(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    println!(
        "{:<6} {:<14} {:<40} {:<22} {:<22}",
        "ID", "STATUS", "DESCRIPTION", "CREATED AT", "UPDATED AT"
    );
    println!("{}", "-".repeat(110));

    for task in tasks {
        let status_display = match task.status {
            TaskStatus::Todo => "[TODO]",
            TaskStatus::InProgress => "[IN PROGRESS]",
            TaskStatus::Done => "[DONE]",
        };

        let desc_display = if task.description.len() > 37 {
            format!("{}...", &task.description[..34])
        } else {
            task.description.clone()
        };

        println!(
            "{:<6} {:<14} {:<40} {:<22} {:<22}",
            task.id, status_display, desc_display, task.created_at, task.updated_at
        );
    }
}

/// Prints application usage instructions.
fn print_usage() {
    println!(
        r#"Task Tracker CLI - Manage your tasks directly from the command line

USAGE:
    task-cli <COMMAND> [ARGUMENTS]

COMMANDS:
    add "<description>"            Add a new task
    update <id> "<description>"    Update an existing task's description
    delete <id>                   Delete a task by ID
    mark-in-progress <id>         Mark a task as in progress
    mark-done <id>                Mark a task as done
    list                          List all tasks
    list done                     List all completed tasks
    list todo                     List all pending tasks
    list in-progress              List all in-progress tasks
    serve [port]                  Start the Axum REST API server (default port: 3000)
    help                          Display this help message

EXAMPLES:
    task-cli add "Buy groceries"
    task-cli update 1 "Buy groceries and cook dinner"
    task-cli mark-in-progress 1
    task-cli mark-done 1
    task-cli list
    task-cli list done
    task-cli delete 1
"#
    );
}
