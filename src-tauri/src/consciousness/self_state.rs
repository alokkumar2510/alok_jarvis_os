use serde::{Serialize, Deserialize};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfState {
    pub cpu_load: f64,
    pub ram_usage_mb: f64,
    pub active_goal: String,
    pub pending_tasks: Vec<String>,
    pub health_score: u8,
}

pub fn get_self_state(state: &AppState) -> SelfState {
    let mut cpu_load = 12.4;
    let mut ram_usage_mb = 112.4;
    
    if let Ok(db) = state.db.lock() {
        let report = crate::intelligence::health_monitor::capture_health_metrics(&db);
        cpu_load = report.cpu_usage_pct;
        ram_usage_mb = report.ram_usage_mb;
    }

    let active_goal = state.planner.get_active_plan()
        .map(|p| p.goal)
        .unwrap_or_else(|| "Idle".to_string());

    let pending_tasks = state.planner.get_active_plan()
        .map(|p| p.steps.iter()
            .filter(|s| s.status == "Pending" || s.status == "Running")
            .map(|s| s.name.clone())
            .collect())
        .unwrap_or_default();

    let health_score = if let Ok(db) = state.db.lock() {
        crate::intelligence::self_diagnostics::run_diagnostics(&db).health_score
    } else {
        98
    };

    SelfState {
        cpu_load,
        ram_usage_mb,
        active_goal,
        pending_tasks,
        health_score,
    }
}
