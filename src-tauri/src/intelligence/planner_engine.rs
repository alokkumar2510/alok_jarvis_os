use crate::planner::{PlannerEngine as BasePlanner, Plan};
use crate::database::Database;
use std::sync::Arc;
use tauri::AppHandle;

pub struct PlannerEngine {
    inner: Arc<BasePlanner>,
}

impl PlannerEngine {
    pub fn new(inner: Arc<BasePlanner>) -> Self {
        Self { inner }
    }

    /// Plans a goal locally using rules or Groq fallback.
    pub fn generate_plan(&self, goal: &str, db: &Database) -> Plan {
        self.inner.generate_plan(goal, db)
    }

    /// Executes the generated plan asynchronously.
    pub fn execute_plan(&self, plan: Plan, app: AppHandle) {
        self.inner.execute_plan(plan, app);
    }
}
