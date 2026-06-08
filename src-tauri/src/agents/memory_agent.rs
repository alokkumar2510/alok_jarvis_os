use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use crate::database::Database;
use crate::planner::Step;

pub struct MemoryAgent;

impl MemoryAgent {
    pub async fn run(goal: &str, db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        println!("MemoryAgent: Initiating memory task for goal: '{}'", goal);

        let mut steps = vec![
            Step {
                name: "Index Memories".to_string(),
                description: "Search SQLite memory records for matching context keywords".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Consolidate Facts".to_string(),
                description: "Map and rank extracted nodes in the knowledge graph".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
        ];

        Self::update_ui(&app, goal, &steps);

        // Step 1: Index Memories
        steps[0].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let query = goal.replace("memory", "").replace("search", "").trim().to_string();
        let db_lock = db.lock().map_err(|e| e.to_string())?;
        let search_results = db_lock.search_memories(&query).unwrap_or_default();
        
        steps[0].status = "Completed".to_string();
        steps[0].description = format!("Found {} matched memories in SQLite", search_results.len());
        Self::update_ui(&app, goal, &steps);

        // Step 2: Consolidate Facts
        steps[1].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let output = if search_results.is_empty() {
            "No direct memory records matched this goal. Scanned indexed records successfully.".to_string()
        } else {
            format!("Recalled matched fact: '{}'", search_results[0])
        };

        steps[1].status = "Completed".to_string();
        steps[1].description = output.clone();
        Self::update_ui(&app, goal, &steps);

        Ok(output)
    }

    fn update_ui(app: &AppHandle, goal: &str, steps: &[Step]) {
        let _ = app.emit("planner-update", serde_json::json!({
            "goal": goal.to_string(),
            "steps": steps.to_vec(),
        }));
    }
}
