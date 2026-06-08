use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use crate::database::Database;

pub mod goal_manager;
pub mod task_planner;
pub mod task_executor;
pub mod verification_engine;
pub mod goal_execution_engine;
pub mod goal_context;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Step {
    pub name: String,
    pub description: String,
    pub status: String, // "Pending", "Running", "Completed", "Failed"
    pub exec_cmd: Option<String>,
    pub verify_cmd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plan {
    pub goal: String,
    pub steps: Vec<Step>,
}

pub struct PlannerEngine {
    pub goal_manager: goal_manager::GoalManager,
}

impl PlannerEngine {
    pub fn new() -> Self {
        Self {
            goal_manager: goal_manager::GoalManager::new(),
        }
    }

    /// Generate a step-by-step plan for a goal using TaskPlanner.
    pub fn generate_plan(&self, goal: &str, db: &Database) -> Plan {
        task_planner::TaskPlanner::plan_goal(db, goal)
    }

    /// Execute the generated plan asynchronously.
    pub fn execute_plan(&self, plan: Plan, app: AppHandle) {
        self.goal_manager.set_goal(plan.clone());
        let active_goal_ref = self.goal_manager.get_active_goal_ref();

        let goal_name = plan.goal.clone();
        let task_id = format!("plan_{}", goal_name.replace(" ", "_"));
        
        crate::control::task_manager::TASK_MANAGER.register_task(
            &task_id,
            &goal_name,
            crate::control::priority_manager::Priority::P2,
            plan.steps.len(),
        );

        tauri::async_runtime::spawn(async move {
            let mut current_plan = plan;
            
            // Emit initial plan to frontend
            let _ = app.emit("planner-update", current_plan.clone());

            let mut final_status = crate::control::task_manager::TaskStatus::Completed;

            for i in 0..current_plan.steps.len() {
                // Check if the plan was cancelled
                {
                    let active = active_goal_ref.lock().unwrap();
                    if active.is_none() || crate::control::task_manager::TASK_MANAGER.is_cancelled(&task_id) {
                        println!("PlannerEngine: Plan execution cancelled.");
                        final_status = crate::control::task_manager::TaskStatus::Cancelled;
                        break;
                    }
                }

                // If paused, wait until resumed
                crate::control::task_manager::TASK_MANAGER.wait_if_paused(&task_id).await;

                // Check again post-pause
                {
                    let active = active_goal_ref.lock().unwrap();
                    if active.is_none() || crate::control::task_manager::TASK_MANAGER.is_cancelled(&task_id) {
                        println!("PlannerEngine: Plan execution cancelled post-pause.");
                        final_status = crate::control::task_manager::TaskStatus::Cancelled;
                        break;
                    }
                }

                // Update progress in TaskManager
                crate::control::task_manager::TASK_MANAGER.update_progress(&task_id, i + 1, current_plan.steps.len());

                // Mark step as Running
                current_plan.steps[i].status = "Running".to_string();
                let _ = app.emit("planner-update", current_plan.clone());
                {
                    if let Ok(mut active) = active_goal_ref.lock() {
                        *active = Some(current_plan.clone());
                    }
                }

                // 1. Run Execution command if any
                let mut exec_success = true;
                if let Some(ref cmd) = current_plan.steps[i].exec_cmd {
                    println!("PlannerEngine: Executing step {}: {}", i + 1, cmd);
                    let start_time = std::time::Instant::now();
                    let run_result = task_executor::TaskExecutor::execute_cmd(cmd);
                    let elapsed_ms = start_time.elapsed().as_millis() as f64;
                    
                    let (output, error_msg) = match run_result {
                        Ok(out) => {
                            println!("PlannerEngine: Step {} execution succeeded: {}", i + 1, out);
                            (out, None)
                        }
                        Err(e) => {
                            eprintln!("PlannerEngine: Step {} execution failed: {}", i + 1, e);
                            exec_success = false;
                            (e.to_string(), Some(e.to_string()))
                        }
                    };
                    
                    // Log command outcome and performance latency
                    use tauri::Manager;
                    if let Some(app_state) = app.try_state::<crate::AppState>() {
                        if let Ok(db_lock) = app_state.db.lock() {
                            let _ = crate::intelligence::execution_history::log_command(
                                &db_lock,
                                &current_plan.steps[i].name,
                                cmd,
                                &output,
                                exec_success,
                                error_msg.as_deref(),
                                elapsed_ms,
                                0,
                            );
                            crate::intelligence::performance_tracker::log_latency(&db_lock, "action_execution", elapsed_ms);
                        }
                    }
                }

                // Check again for cancellation after execution
                {
                    let active = active_goal_ref.lock().unwrap();
                    if active.is_none() || crate::control::task_manager::TASK_MANAGER.is_cancelled(&task_id) {
                        println!("PlannerEngine: Plan execution cancelled during step execution.");
                        final_status = crate::control::task_manager::TaskStatus::Cancelled;
                        break;
                    }
                }

                // 2. Run Verification command if execution succeeded
                let mut verify_success = false;
                if exec_success {
                    let verify_cmd = current_plan.steps[i].verify_cmd.as_deref();
                    verify_success = verification_engine::VerificationEngine::verify_step(verify_cmd);
                }

                // Check again for cancellation after verification
                {
                    let active = active_goal_ref.lock().unwrap();
                    if active.is_none() || crate::control::task_manager::TASK_MANAGER.is_cancelled(&task_id) {
                        println!("PlannerEngine: Plan execution cancelled during step verification.");
                        final_status = crate::control::task_manager::TaskStatus::Cancelled;
                        break;
                    }
                }

                if exec_success && verify_success {
                    current_plan.steps[i].status = "Completed".to_string();
                } else {
                    current_plan.steps[i].status = "Failed".to_string();
                    let _ = app.emit("planner-update", current_plan.clone());
                    {
                        if let Ok(mut active) = active_goal_ref.lock() {
                            *active = Some(current_plan.clone());
                        }
                    }
                    final_status = crate::control::task_manager::TaskStatus::Failed;
                    break;
                }

                let _ = app.emit("planner-update", current_plan.clone());
                {
                    if let Ok(mut active) = active_goal_ref.lock() {
                        *active = Some(current_plan.clone());
                    }
                }
            }

            crate::control::task_manager::TASK_MANAGER.update_status(&task_id, final_status);
            println!("PlannerEngine: Finished plan execution.");
        });
    }

    pub fn cancel_plan(&self) {
        self.goal_manager.cancel_goal();
    }

    pub fn get_active_plan(&self) -> Option<Plan> {
        self.goal_manager.get_active_goal()
    }
}
