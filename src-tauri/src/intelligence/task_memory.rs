use crate::database::Database;
use crate::intelligence::{solution_database, execution_history};

/// Recalls how a build issue or task was solved by querying solutions and command logs
pub fn recall_build_fix(db: &Database, query: &str) -> Result<String, String> {
    let query_clean = query.trim().trim_end_matches('?');

    // 1. Check direct error signature matches in solution database first
    if let Some(sol) = solution_database::find_solution(db, query_clean)? {
        return Ok(sol);
    }

    // 2. Scan recent execution history to locate failed commands that were later resolved
    let history = execution_history::get_history(db, 50)?;
    let query_lower = query_clean.to_lowercase();

    let mut relevant_fail = None;
    let mut relevant_success = None;

    for exec in &history {
        let cmd_lower = exec.command.to_lowercase();
        let task_lower = exec.task_name.to_lowercase();
        let out_lower = exec.output.to_lowercase();

        if cmd_lower.contains(&query_lower) || task_lower.contains(&query_lower) || out_lower.contains(&query_lower) {
            if !exec.success {
                if relevant_fail.is_none() {
                    relevant_fail = Some(exec.clone());
                }
            } else {
                if relevant_success.is_none() {
                    relevant_success = Some(exec.clone());
                }
            }
        }
    }

    if let Some(fail) = relevant_fail {
        if let Some(success) = relevant_success {
            return Ok(format!(
                "I found an execution sequence:\n\
                 - Encountered Failure: '{}'\n  Error details: {}\n\
                 - Subsequently Succeeded with command: '{}'\n  Output: {}",
                fail.command,
                fail.error_msg.as_deref().unwrap_or(&fail.output),
                success.command,
                success.output
            ));
        } else {
            return Ok(format!(
                "I found a record of a failed build command:\n\
                 - Command: '{}'\n  Error details: {}",
                fail.command,
                fail.error_msg.as_deref().unwrap_or(&fail.output)
            ));
        }
    }

    Ok("I couldn't find a recorded solution or matching command execution history in my logs.".to_string())
}
