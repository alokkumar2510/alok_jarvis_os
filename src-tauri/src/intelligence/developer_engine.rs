use crate::database::Database;
use crate::intelligence::{code_analyzer, log_analyzer, solution_database};
use std::sync::{Arc, Mutex};

pub struct DeveloperEngine {
    db: Arc<Mutex<Database>>,
}

impl DeveloperEngine {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Analyzes raw build logs or tracebacks and provides actionable suggestions
    pub fn analyze_error(&self, error_log: &str) -> String {
        let db_lock = self.db.lock().unwrap();

        // 1. Search for matching solution in solution_database first
        if let Ok(Some(sol)) = solution_database::find_solution(&db_lock, error_log) {
            return format!("Known Solution Found:\n{}", sol);
        }

        // 2. Fallback to offline parsing via log_analyzer
        if let Some(parsed) = log_analyzer::parse_log(error_log) {
            let mut explanation = format!(
                "Developer Assistant: I detected a **{}** error (type: `{}`):\n\
                 - Description: {}\n",
                parsed.language, parsed.error_type, parsed.description
            );

            if let Some(loc) = parsed.file_location {
                explanation.push_str(&format!(" - File Location: {}\n", loc));
            }

            let suggestion = match parsed.language.as_str() {
                "Rust" => {
                    if parsed.error_type.contains("E0382") {
                        "This is a Rust borrow checker error. The value was moved and then used again. You should borrow it (e.g. `&value`), clone it, or adjust the scopes."
                    } else if parsed.error_type.contains("E0432") {
                        "An import was not found. Check if the crate is declared in Cargo.toml or if the module path is correct."
                    } else {
                        "Review Rust compiler diagnostics. Run `cargo check` to examine error warnings."
                    }
                }
                "Python" => {
                    if parsed.error_type.contains("ModuleNotFoundError") {
                        "A required Python module is missing. Run `pip install <module_name>` inside your active virtual environment."
                    } else if parsed.error_type.contains("KeyError") {
                        "Dictionary key lookup failed. Ensure keys are populated or use `.get('key')` for optional values."
                    } else {
                        "Examine the Python stack traceback and check local variable states at the failing line."
                    }
                }
                "JavaScript/TypeScript" => {
                    if parsed.error_type.contains("TypeError") {
                        "You attempted to invoke a method or property on a null/undefined value. Check if variable variables are initialized."
                    } else {
                        "Inspect stack traces. Verify if npm modules are fully installed (`npm install`)."
                    }
                }
                _ => "Review the logs carefully to find the root cause."
            };

            explanation.push_str(&format!("\nSuggested Fix:\n{}", suggestion));
            return explanation;
        }

        "Developer Assistant: I couldn't parse the diagnostic signature from this log. Review compile/run parameters, or let me know if you want to search the codebase for symbols.".to_string()
    }

    /// Searches codebase files for symbol occurrences
    pub fn search_symbols(&self, root_path: &str, query: &str) -> String {
        let results = code_analyzer::search_codebase(root_path, query);
        if results.is_empty() {
            format!("No occurrences of '{}' found in codebase '{}'.", query, root_path)
        } else {
            format!(
                "Found {} occurrence(s) of '{}' in project structure:\n{}",
                results.len(),
                query,
                results.join("\n")
            )
        }
    }
}
