use std::sync::{Arc, Mutex};
use std::process::Command;
use tauri::{AppHandle, Emitter};
use crate::database::Database;
use crate::planner::Step;

pub struct SystemAgent;

impl SystemAgent {
    pub async fn run(goal: &str, _db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        println!("SystemAgent: Starting environment check/setup goal: '{}'", goal);

        let mut steps = vec![
            Step {
                name: "Check Pre-requisites".to_string(),
                description: "Verify if required compiler/runtime is installed".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Verify Environment Version".to_string(),
                description: "Run version check commands to verify PATH availability".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
        ];

        Self::update_ui(&app, goal, &steps);

        // Step 1: Check Pre-requisites
        steps[0].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let goal_lower = goal.to_lowercase();
        let (check_cmd, arg) = if goal_lower.contains("python") {
            ("python", "--version")
        } else if goal_lower.contains("rust") {
            ("rustc", "--version")
        } else if goal_lower.contains("node") {
            ("node", "--version")
        } else if goal_lower.contains("flutter") {
            ("flutter", "--version")
        } else {
            ("git", "--version") // default fallback check
        };

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &format!("{} {}", check_cmd, arg)])
            .output();

        let mut success = false;
        let msg = match output {
            Ok(out) if out.status.success() => {
                success = true;
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
            Ok(out) => {
                String::from_utf8_lossy(&out.stderr).trim().to_string()
            }
            Err(e) => {
                e.to_string()
            }
        };

        if success {
            steps[0].status = "Completed".to_string();
            steps[0].description = format!("Found: {}", msg);
        } else {
            // Failure recovery: notify user about missing dependency
            steps[0].status = "Failed".to_string();
            steps[0].description = format!("Dependency not found or error occurred: {}", msg);
        }
        Self::update_ui(&app, goal, &steps);

        // Step 2: Verify Environment Version
        steps[1].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        if success {
            steps[1].status = "Completed".to_string();
            Self::update_ui(&app, goal, &steps);
            Ok(format!("Environment setup check completed: {}.", msg))
        } else {
            steps[1].status = "Failed".to_string();
            Self::update_ui(&app, goal, &steps);
            Err(format!("Pre-requisite missing: {}", msg))
        }
    }

    fn update_ui(app: &AppHandle, goal: &str, steps: &[Step]) {
        let _ = app.emit("planner-update", serde_json::json!({
            "goal": goal.to_string(),
            "steps": steps.to_vec(),
        }));
    }
}
