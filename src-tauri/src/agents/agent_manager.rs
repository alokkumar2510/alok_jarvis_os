use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};
use crate::database::Database;
use crate::AppState;

pub struct AgentManager;

impl AgentManager {
    /// Decomposes a swarm goal, executes specialized agents in parallel where possible,
    /// updates the AppState agent activity status map, and aggregates results.
    pub async fn run_swarm(goal: &str, db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        let goal_lower = goal.to_lowercase();
        println!("AgentManager: Swarm coordinator received goal: '{}'", goal);

        // Get access to the shared agent activity state
        let activity_map = if let Some(state) = app.try_state::<AppState>() {
            Some(state.agent_activity.clone())
        } else {
            None
        };

        let set_agent_status = |agent: &str, status: &str| {
            if let Some(ref map) = activity_map {
                if let Ok(mut m) = map.lock() {
                    m.insert(agent.to_string(), status.to_string());
                }
            }
        };

        // Determine which agents are needed based on the goal keywords
        let run_browser = goal_lower.contains("browser") || goal_lower.contains("tabs") || goal_lower.contains("youtube");
        let run_file = goal_lower.contains("file") || goal_lower.contains("folder") || goal_lower.contains("downloads") || goal_lower.contains("organize");
        let run_memory = goal_lower.contains("memory") || goal_lower.contains("recall") || goal_lower.contains("fact");
        let run_developer = goal_lower.contains("code") || goal_lower.contains("error") || goal_lower.contains("compiler") || goal_lower.contains("logs");
        let run_research = goal_lower.contains("research") || goal_lower.contains("find out") || goal_lower.contains("compare") || goal_lower.contains("arxiv");
        let run_automation = goal_lower.contains("automation") || goal_lower.contains("launch") || goal_lower.contains("system") || goal_lower.contains("volume");

        // Set default idle for all agents if map is empty
        if let Some(ref map) = activity_map {
            let mut m = map.lock().unwrap();
            if m.is_empty() {
                m.insert("BrowserAgent".to_string(), "Idle".to_string());
                m.insert("FileAgent".to_string(), "Idle".to_string());
                m.insert("MemoryAgent".to_string(), "Idle".to_string());
                m.insert("DeveloperAgent".to_string(), "Idle".to_string());
                m.insert("ResearchAgent".to_string(), "Idle".to_string());
                m.insert("AutomationAgent".to_string(), "Idle".to_string());
            }
        }

        // We will execute the active agents in parallel using tokio::spawn and collect handles
        let mut handles = Vec::new();
        let db_ref = db.clone();
        let app_ref = app.clone();

        // 1. Spawning BrowserAgent
        if run_browser {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("BrowserAgent", "Running: Executing browser actions");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::browser_agent::BrowserAgent::run(&goal_c, db_c, app_c).await;
                ("BrowserAgent", res)
            }));
        }

        // 2. Spawning FileAgent
        if run_file {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("FileAgent", "Running: Processing files");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::file_agent::FileAgent::run(&goal_c, db_c, app_c).await;
                ("FileAgent", res)
            }));
        }

        // 3. Spawning MemoryAgent
        if run_memory {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("MemoryAgent", "Running: Retrieving facts");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::memory_agent::MemoryAgent::run(&goal_c, db_c, app_c).await;
                ("MemoryAgent", res)
            }));
        }

        // 4. Spawning DeveloperAgent
        if run_developer {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("DeveloperAgent", "Running: Diagnostics");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::developer_agent::DeveloperAgent::run(&goal_c, db_c, app_c).await;
                ("DeveloperAgent", res)
            }));
        }

        // 5. Spawning ResearchAgent
        if run_research {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("ResearchAgent", "Running: Scraping web sources");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::research_agent::ResearchAgent::run(&goal_c, db_c, app_c).await;
                ("ResearchAgent", res)
            }));
        }

        // 6. Spawning AutomationAgent
        if run_automation {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("AutomationAgent", "Running: Controlling system");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::automation_agent::AutomationAgent::run(&goal_c, db_c, app_c).await;
                ("AutomationAgent", res)
            }));
        }

        // Fallback: If no keyword matches, delegate to SystemAgent as default
        if handles.is_empty() {
            let db_c = db_ref.clone();
            let app_c = app_ref.clone();
            let goal_c = goal.to_string();
            set_agent_status("AutomationAgent", "Running: Running system diagnostics");
            handles.push(tokio::spawn(async move {
                let res = crate::agents::system_agent::SystemAgent::run(&goal_c, db_c, app_c).await;
                ("AutomationAgent", res)
            }));
        }

        // Wait for all spawned agents to complete in parallel
        let mut results = Vec::new();
        for h in handles {
            if let Ok((agent_name, outcome)) = h.await {
                set_agent_status(agent_name, "Idle");
                match outcome {
                    Ok(out) => results.push(format!("{}: {}", agent_name, out)),
                    Err(err) => results.push(format!("{}: Failed ({})", agent_name, err)),
                }
            }
        }

        Ok(results.join("\n"))
    }
}
