use crate::database::{Database, SolutionEntry};

/// Adds an error signature pattern and its corresponding resolution details to the database
pub fn save_solution(
    db: &Database,
    error_pattern: &str,
    solution_details: &str,
    build_command: &str,
) -> Result<(), String> {
    db.add_solution(error_pattern, solution_details, build_command)
        .map_err(|e| e.to_string())
}

/// Retrieves all solutions stored in the database
pub fn get_solutions(db: &Database) -> Result<Vec<SolutionEntry>, String> {
    db.get_solutions().map_err(|e| e.to_string())
}

/// Scans the provided error output against stored error patterns to find a matching fix
pub fn find_solution(db: &Database, error_output: &str) -> Result<Option<String>, String> {
    let solutions = get_solutions(db)?;
    let error_lower = error_output.to_lowercase();

    for entry in solutions {
        if error_lower.contains(&entry.error_pattern.to_lowercase()) {
            return Ok(Some(format!(
                "I recall how you solved this before:\n- Command: {}\n- Fix Details: {}",
                entry.build_command, entry.solution_details
            )));
        }
    }
    Ok(None)
}
