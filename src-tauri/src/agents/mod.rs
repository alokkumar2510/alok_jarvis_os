use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use crate::database::Database;

pub mod research_agent;
pub mod automation_agent;
pub mod productivity_agent;
pub mod browser_agent;
pub mod system_agent;
pub mod file_agent;
pub mod memory_agent;
pub mod developer_agent;
pub mod agent_manager;

pub struct AgentFramework;

impl AgentFramework {
    /// Delegates the goal to the AgentManager coordinator for parallel execution in the swarm.
    pub async fn execute_agent_goal(goal: &str, db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        agent_manager::AgentManager::run_swarm(goal, db, app).await
    }
}
