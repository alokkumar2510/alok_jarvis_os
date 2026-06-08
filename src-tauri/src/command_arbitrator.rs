use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use crate::action_bus::{ActionRequest, ActionResponse};
use crate::AppState;
use tauri::AppHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

pub struct ActiveCommandEntry {
    pub request: ActionRequest,
    pub priority: CommandPriority,
    pub cancel_tx: oneshot::Sender<()>,
}

pub struct QueuedCommand {
    pub request: ActionRequest,
    pub priority: CommandPriority,
    pub start_tx: oneshot::Sender<oneshot::Receiver<()>>,
}

pub struct CommandArbitrator {
    active_commands: Arc<Mutex<HashMap<String, ActiveCommandEntry>>>,
    command_queues: Arc<Mutex<HashMap<String, VecDeque<QueuedCommand>>>>,
}

impl CommandArbitrator {
    pub fn new() -> Self {
        Self {
            active_commands: Arc::new(Mutex::new(HashMap::new())),
            command_queues: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Determines the priority of an ActionRequest.
    pub fn get_priority(request: &ActionRequest) -> CommandPriority {
        let action_id = request.action_id.as_str();

        // 1. Critical priority: safety, termination, and voice interruptions
        if action_id == "stop_speech" || action_id == "resume_speech" || action_id == "sleep_system" || action_id == "lock_system" {
            return CommandPriority::Critical;
        }

        // 2. High priority: application launching and closing commands
        if action_id == "launch_app" || action_id == "close_app" {
            return CommandPriority::High;
        }

        // 3. Medium priority: volume adjustments, browser control, planner tasks
        if action_id == "volume_up" || action_id == "volume_down" || action_id == "mute_volume"
           || request.action_type == crate::action_bus::ActionType::BrowserControl
           || request.action_type == crate::action_bus::ActionType::PlannerExecution
        {
            return CommandPriority::Medium;
        }

        // 4. Low priority: default
        CommandPriority::Low
    }

    /// Computes a conflict key for an ActionRequest.
    /// Commands that yield the same conflict key are considered mutually exclusive.
    pub fn get_conflict_key(request: &ActionRequest) -> Option<String> {
        let action_id = request.action_id.as_str();
        
        // 1. App control and Browser commands targeting the same resource
        let app_name = request.args.get("app_name")
            .or_else(|| request.args.get("app"))
            .map(|s| s.to_lowercase());

        if request.action_type == crate::action_bus::ActionType::BrowserControl 
           || action_id == "launch_app" 
           || action_id == "close_app" 
           || action_id == "focus_app" 
           || action_id == "minimize_app" 
           || action_id == "maximize_app" 
        {
            let name = app_name.unwrap_or_else(|| "browser".to_string());
            // Map common browsers to a unified key
            let unified_name = if name.contains("chrome") || name.contains("browser") || name.contains("edge") || name.contains("firefox") {
                "browser".to_string()
            } else {
                name
            };
            return Some(format!("app:{}", unified_name));
        }

        // 2. Volume controls
        if action_id == "volume_up" || action_id == "volume_down" || action_id == "mute_volume" {
            return Some("device:volume".to_string());
        }

        // 3. System power states
        if action_id == "lock_system" || action_id == "sleep_system" {
            return Some("system:power".to_string());
        }

        // 4. Planner and Agent executions
        if request.action_type == crate::action_bus::ActionType::PlannerExecution {
            return Some("planner:exec".to_string());
        }

        None
    }

    /// Try to merge two conflicting commands to optimize execution.
    pub fn try_merge(active: &ActionRequest, incoming: &ActionRequest) -> Option<ActionRequest> {
        if active.action_id == incoming.action_id {
            // Volume commands can be merged by keeping the newest one
            if active.action_id == "volume_up" || active.action_id == "volume_down" {
                return Some(incoming.clone());
            }
        }
        None
    }

    /// Executing command with conflict detection, preemption, priority handling, queueing, and merging.
    pub async fn arbitrate_and_execute(
        &self,
        request: ActionRequest,
        state: &AppState,
        app: &AppHandle,
    ) -> ActionResponse {
        let key_opt = Self::get_conflict_key(&request);
        
        if let Some(ref key) = key_opt {
            let active = self.active_commands.lock().unwrap();
            if active.contains_key(key) {
                println!(
                    "CommandArbitrator: Conflict detected on key '{}' for incoming command '{}'",
                    key, request.action_id
                );
                crate::log_dev_event(
                    app,
                    "Arbitrator",
                    "warn",
                    &format!(
                        "Preempting/Queueing action '{}' due to conflict key '{}'",
                        request.action_id, key
                    )
                );
            }
        }

        let action_bus = state.action_bus.clone();
        let app_clone = app.clone();
        let request_clone = request.clone();
        self.arbitrate_and_execute_with(request, move || async move {
            action_bus.dispatch(request_clone, &app_clone).await
        }).await
    }

    /// Core arbitration execution logic, isolated from Tauri/AppState types for testing.
    pub async fn arbitrate_and_execute_with<F, Fut>(
        &self,
        request: ActionRequest,
        f: F,
    ) -> ActionResponse
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ActionResponse>,
    {
        let key_opt = Self::get_conflict_key(&request);
        let priority = Self::get_priority(&request);
        let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();

        // We wrap mutex locks in an isolated scope block to ensure guards are dropped
        // before any await point, satisfying Send/Sync bounds in Tauri async commands.
        let start_rx_opt = if let Some(ref key) = key_opt {
            let mut active_lock = self.active_commands.lock().unwrap();
            let mut queues_lock = self.command_queues.lock().unwrap();

            let mut execute_now = false;
            
            if let Some(active_entry) = active_lock.get(key) {
                // Try to merge commands first
                if let Some(_merged) = Self::try_merge(&active_entry.request, &request) {
                    println!(
                        "CommandArbitrator: Merged incoming command '{}' with active '{}'",
                        request.action_id, active_entry.request.action_id
                    );
                }

                if priority >= active_entry.priority {
                    execute_now = true;
                }
            } else {
                execute_now = true;
            }

            if execute_now {
                // Remove active entry to gain ownership of the oneshot Sender
                if let Some(old_entry) = active_lock.remove(key) {
                    println!(
                        "CommandArbitrator: Preempting running command '{:?}' (priority {:?}) with newer command '{:?}' (priority {:?})",
                        old_entry.request.action_id, old_entry.priority, request.action_id, priority
                    );
                    let _ = old_entry.cancel_tx.send(());
                }

                // Clear queue for this key, as a newer high-priority command cancels all previous pending commands
                if let Some(q) = queues_lock.get_mut(key) {
                    q.clear();
                }

                active_lock.insert(key.clone(), ActiveCommandEntry {
                    request: request.clone(),
                    priority,
                    cancel_tx,
                });
                None
            } else {
                // Lower priority command: queue it
                println!(
                    "CommandArbitrator: Queueing command '{}' (priority {:?}) behind active command",
                    request.action_id, priority
                );
                let (start_tx, start_rx) = oneshot::channel::<oneshot::Receiver<()>>();
                let queued = QueuedCommand {
                    request: request.clone(),
                    priority,
                    start_tx,
                };
                queues_lock.entry(key.clone()).or_insert_with(VecDeque::new).push_back(queued);
                Some(start_rx)
            }
        } else {
            None
        };

        if let Some(start_rx) = start_rx_opt {
            tokio::select! {
                res = start_rx => {
                    match res {
                        Ok(rx) => {
                            cancel_rx = rx;
                        }
                        Err(_) => {
                            return ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some("Queued command was cancelled before execution".to_string()),
                            };
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                    return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some("Queued command timed out waiting for execution".to_string()),
                    };
                }
            }
        }

        let request_clone = request.clone();
        let key_clone = key_opt.clone();

        let response = tokio::select! {
            res = f() => res,
            _ = &mut cancel_rx => {
                println!("CommandArbitrator: Action '{}' was cancelled/preempted.", request_clone.action_id);
                ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Action '{}' was cancelled because a newer instruction was received.",
                        request_clone.action_id
                    )),
                }
            }
        };

        // Post-execution: pop from queue and trigger next command if applicable
        if let Some(ref key) = key_clone {
            let mut active_lock = self.active_commands.lock().unwrap();
            let mut queues_lock = self.command_queues.lock().unwrap();

            // Clear active registry if it was indeed this command that just finished
            if let Some(entry) = active_lock.get(key) {
                if entry.request.action_id == request_clone.action_id {
                    active_lock.remove(key);
                }
            }

            // Trigger next queued command if any
            if let Some(q) = queues_lock.get_mut(key) {
                if let Some(next_command) = q.pop_front() {
                    let (new_cancel_tx, new_cancel_rx) = oneshot::channel::<()>();

                    active_lock.insert(key.clone(), ActiveCommandEntry {
                        request: next_command.request.clone(),
                        priority: next_command.priority,
                        cancel_tx: new_cancel_tx,
                    });

                    let _ = next_command.start_tx.send(new_cancel_rx);
                }
            }
        }

        response
    }

    /// Purge all active and queued commands.
    pub fn clear_all(&self) {
        let mut active = self.active_commands.lock().unwrap();
        let mut queues = self.command_queues.lock().unwrap();
        
        // Cancel all active commands
        for (_, entry) in active.drain() {
            let _ = entry.cancel_tx.send(());
        }
        
        // Clear all queued commands
        queues.clear();
        println!("CommandArbitrator: Cleared all active and queued commands.");
    }
}

lazy_static::lazy_static! {
    pub static ref ARBITRATOR: CommandArbitrator = CommandArbitrator::new();
}
