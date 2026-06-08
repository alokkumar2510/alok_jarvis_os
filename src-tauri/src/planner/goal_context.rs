use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoalContext {
    pub goal: String,
    pub progress_pct: f64,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
}

impl GoalContext {
    pub fn from_plan(plan: &crate::planner::Plan) -> Self {
        let mut completed = Vec::new();
        let mut failed = Vec::new();
        let mut pending = Vec::new();

        for step in &plan.steps {
            match step.status.as_str() {
                "Completed" => completed.push(step.name.clone()),
                "Failed" => failed.push(step.name.clone()),
                "Running" => pending.push(format!("(Running) {}", step.name)),
                _ => pending.push(step.name.clone()),
            }
        }

        let total = plan.steps.len() as f64;
        let progress_pct = if total > 0.0 {
            (completed.len() as f64 / total) * 100.0
        } else {
            0.0
        };

        Self {
            goal: plan.goal.clone(),
            progress_pct,
            completed_steps: completed,
            failed_steps: failed,
            pending_steps: pending,
        }
    }
}
