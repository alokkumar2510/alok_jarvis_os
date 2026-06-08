use crate::planner::Plan;
use std::sync::{Arc, Mutex};

pub struct GoalManager {
    active_goal: Arc<Mutex<Option<Plan>>>,
}

impl GoalManager {
    pub fn new() -> Self {
        Self {
            active_goal: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_goal(&self, plan: Plan) {
        if let Ok(mut active) = self.active_goal.lock() {
            *active = Some(plan);
        }
    }

    pub fn get_active_goal(&self) -> Option<Plan> {
        self.active_goal.lock().ok().and_then(|g| g.clone())
    }

    pub fn get_active_goal_ref(&self) -> Arc<Mutex<Option<Plan>>> {
        self.active_goal.clone()
    }

    pub fn cancel_goal(&self) {
        if let Ok(mut active) = self.active_goal.lock() {
            *active = None;
        }
    }

    pub fn update_step_status(&self, step_idx: usize, status: &str) -> Option<Plan> {
        if let Ok(mut active) = self.active_goal.lock() {
            if let Some(ref mut plan) = *active {
                if step_idx < plan.steps.len() {
                    plan.steps[step_idx].status = status.to_string();
                    return Some(plan.clone());
                }
            }
        }
        None
    }
}
