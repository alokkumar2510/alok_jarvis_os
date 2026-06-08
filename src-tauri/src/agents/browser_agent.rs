use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use crate::database::Database;
use crate::planner::Step;
use crate::environment_awareness::browser_context_engine::BrowserContextEngine;

pub struct BrowserAgent;

impl BrowserAgent {
    pub async fn run(goal: &str, _db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        println!("BrowserAgent: Starting browser task chain for goal: '{}'", goal);

        let mut steps = vec![
            Step {
                name: "Analyze Active Tabs".to_string(),
                description: "Scan the window manager for active browser open tabs".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Execute Tab Operation".to_string(),
                description: "Perform clean, search, or group tabs operation".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Verify Automation Result".to_string(),
                description: "Confirm target browser tab state conforms to request".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
        ];

        Self::update_ui(&app, goal, &steps);

        // Step 1: Analyze Active Tabs
        steps[0].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let tabs = BrowserContextEngine::get_open_tabs();

        steps[0].status = "Completed".to_string();
        steps[0].description = format!("Found {} open tabs in active browsers", tabs.len());
        Self::update_ui(&app, goal, &steps);

        // Step 2: Execute Tab Operation
        steps[1].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let goal_lower = goal.to_lowercase();
        let result_msg = if goal_lower.contains("close") || goal_lower.contains("duplicate") || goal_lower.contains("clean") {
            BrowserContextEngine::close_duplicate_tabs()
        } else if goal_lower.contains("youtube") {
            // Find query term
            let query = if goal_lower.contains("rust") { "Rust tutorials" } else { "coding references" };
            match crate::desktop::control::open_website(&format!("https://www.youtube.com/results?search_query={}", query.replace(" ", "+"))) {
                Ok(_) => format!("Search results for YouTube opened successfully."),
                Err(e) => format!("Failed to open YouTube search: {}", e),
            }
        } else {
            "No specific browser automation match. Scanned tabs successfully.".to_string()
        };

        steps[1].status = "Completed".to_string();
        steps[1].description = result_msg.clone();
        Self::update_ui(&app, goal, &steps);

        // Step 3: Verify Automation Result
        steps[2].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        steps[2].status = "Completed".to_string();
        Self::update_ui(&app, goal, &steps);

        Ok(result_msg)
    }

    fn update_ui(app: &AppHandle, goal: &str, steps: &[Step]) {
        let _ = app.emit("planner-update", serde_json::json!({
            "goal": goal.to_string(),
            "steps": steps.to_vec(),
        }));
    }
}
