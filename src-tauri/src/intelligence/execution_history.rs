use crate::database::Database;

pub fn log_command(
    db: &Database,
    task_name: &str,
    command: &str,
    output: &str,
    success: bool,
    error_msg: Option<&str>,
    duration_ms: f64,
    retries: i32,
) -> Result<(), String> {
    db.log_task_execution(task_name, command, output, success, error_msg, duration_ms, retries)
        .map_err(|e| e.to_string())
}

/// Retrieves the most recent task command executions
pub fn get_history(db: &Database, limit: u32) -> Result<Vec<crate::database::TaskExecution>, String> {
    db.get_recent_task_executions(limit).map_err(|e| e.to_string())
}
