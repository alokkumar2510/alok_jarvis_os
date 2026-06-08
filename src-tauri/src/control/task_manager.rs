use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use crate::control::priority_manager::Priority;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Paused,
    Queued,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "Running",
            TaskStatus::Paused => "Paused",
            TaskStatus::Queued => "Queued",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
        }
    }
}

pub struct RollbackAction {
    pub description: String,
    pub undo: Box<dyn FnOnce() -> Result<(), String> + Send + Sync>,
}

pub struct Task {
    pub id: String,
    pub goal: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub current_step: usize,
    pub total_steps: usize,
    pub start_time: Instant,
    pub remaining_time_seconds: Option<u64>,
    pub pause_tx: watch::Sender<bool>,
    pub pause_rx: watch::Receiver<bool>,
    pub affected_files: Vec<String>,
    pub rollback_log: Vec<RollbackAction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskSnapshot {
    pub id: String,
    pub goal: String,
    pub status: String,
    pub priority: String,
    pub current_step: usize,
    pub total_steps: usize,
    pub elapsed_seconds: u64,
    pub remaining_time_seconds: Option<u64>,
    pub affected_files: Vec<String>,
}

pub struct TaskManager {
    tasks: Mutex<HashMap<String, Task>>,
}

lazy_static::lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = TaskManager::new();
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
        }
    }

    /// Registers a new task. If an active task exists with the same ID, it is updated.
    pub fn register_task(&self, id: &str, goal: &str, priority: Priority, total_steps: usize) {
        let mut lock = self.tasks.lock().unwrap();
        
        let (tx, rx) = watch::channel(false); // Default: not paused
        
        let task = Task {
            id: id.to_string(),
            goal: goal.to_string(),
            status: TaskStatus::Running,
            priority,
            current_step: 0,
            total_steps,
            start_time: Instant::now(),
            remaining_time_seconds: None,
            pause_tx: tx,
            pause_rx: rx,
            affected_files: Vec::new(),
            rollback_log: Vec::new(),
        };
        
        lock.insert(id.to_string(), task);
        println!("TaskManager: Task '{}' registered successfully with priority {:?}", id, priority);
    }

    /// Retrieve the status of a task
    pub fn get_task_status(&self, id: &str) -> Option<TaskStatus> {
        let lock = self.tasks.lock().unwrap();
        lock.get(id).map(|t| t.status)
    }

    /// Set status of a task
    pub fn update_status(&self, id: &str, status: TaskStatus) {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            task.status = status;
            
            // If completed or failed, make sure we unpause
            if status == TaskStatus::Completed || status == TaskStatus::Failed || status == TaskStatus::Cancelled {
                let _ = task.pause_tx.send(false);
            }
            println!("TaskManager: Task '{}' status updated to {:?}", id, status);
        }
    }

    /// Check if task is paused
    pub fn is_paused(&self, id: &str) -> bool {
        let lock = self.tasks.lock().unwrap();
        lock.get(id).map(|t| t.status == TaskStatus::Paused).unwrap_or(false)
    }

    /// Check if task is cancelled
    pub fn is_cancelled(&self, id: &str) -> bool {
        let lock = self.tasks.lock().unwrap();
        lock.get(id).map(|t| t.status == TaskStatus::Cancelled).unwrap_or(false)
    }

    /// Check if any task is active (running or paused)
    pub fn get_active_task_id(&self) -> Option<String> {
        let lock = self.tasks.lock().unwrap();
        for (id, t) in lock.iter() {
            if t.status == TaskStatus::Running || t.status == TaskStatus::Paused {
                return Some(id.clone());
            }
        }
        None
    }

    /// Update progress and dynamically estimate remaining time
    pub fn update_progress(&self, id: &str, current_step: usize, total_steps: usize) {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            task.current_step = current_step;
            task.total_steps = total_steps;
            
            // Calculate dynamic remaining time estimate
            let elapsed = task.start_time.elapsed().as_secs();
            if current_step > 0 && total_steps > current_step {
                let time_per_step = elapsed as f64 / current_step as f64;
                let steps_left = (total_steps - current_step) as f64;
                task.remaining_time_seconds = Some((time_per_step * steps_left).round() as u64);
            }
        }
    }

    /// Pause task
    pub fn pause_task(&self, id: &str) -> Result<(), String> {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Paused;
                let _ = task.pause_tx.send(true);
                println!("TaskManager: Task '{}' paused.", id);
                Ok(())
            } else {
                Err(format!("Task is not running (current state: {:?})", task.status))
            }
        } else {
            Err("Task not found".to_string())
        }
    }

    /// Resume task
    pub fn resume_task(&self, id: &str) -> Result<(), String> {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            if task.status == TaskStatus::Paused {
                task.status = TaskStatus::Running;
                let _ = task.pause_tx.send(false);
                println!("TaskManager: Task '{}' resumed.", id);
                Ok(())
            } else {
                Err(format!("Task is not paused (current state: {:?})", task.status))
            }
        } else {
            Err("Task not found".to_string())
        }
    }

    /// Pause all running tasks
    pub fn pause_all_running(&self) {
        let mut lock = self.tasks.lock().unwrap();
        for (id, task) in lock.iter_mut() {
            if task.status == TaskStatus::Running {
                task.status = TaskStatus::Paused;
                let _ = task.pause_tx.send(true);
                println!("TaskManager: Automatically paused task '{}'", id);
            }
        }
    }

    /// Resume all paused tasks
    pub fn resume_all_paused(&self) {
        let mut lock = self.tasks.lock().unwrap();
        for (id, task) in lock.iter_mut() {
            if task.status == TaskStatus::Paused {
                task.status = TaskStatus::Running;
                let _ = task.pause_tx.send(false);
                println!("TaskManager: Automatically resumed task '{}'", id);
            }
        }
    }

    /// Get wait lock receiver for task pausing
    pub fn get_pause_rx(&self, id: &str) -> Option<watch::Receiver<bool>> {
        let lock = self.tasks.lock().unwrap();
        lock.get(id).map(|t| t.pause_rx.clone())
    }

    /// Yield execution if the task is paused
    pub async fn wait_if_paused(&self, id: &str) {
        let mut rx = {
            let lock = self.tasks.lock().unwrap();
            match lock.get(id) {
                Some(t) => t.pause_rx.clone(),
                None => return,
            }
        };

        while *rx.borrow() {
            let _ = rx.changed().await;
        }
    }

    /// Registers a rollback action for a task
    pub fn register_rollback_action<F>(&self, id: &str, description: &str, undo: F)
    where
        F: FnOnce() -> Result<(), String> + Send + Sync + 'static,
    {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            task.rollback_log.push(RollbackAction {
                description: description.to_string(),
                undo: Box::new(undo),
            });
            println!("TaskManager: Registered rollback action for task '{}': {}", id, description);
        }
    }

    /// Registers a file modified by a task
    pub fn register_affected_file(&self, id: &str, path: &str) {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            if !task.affected_files.contains(&path.to_string()) {
                task.affected_files.push(path.to_string());
            }
        }
    }

    /// Pop and return the rollback actions for a task (taking ownership to run them)
    pub fn take_rollback_actions(&self, id: &str) -> Vec<RollbackAction> {
        let mut lock = self.tasks.lock().unwrap();
        if let Some(task) = lock.get_mut(id) {
            std::mem::take(&mut task.rollback_log)
        } else {
            Vec::new()
        }
    }

    /// Return snapshots of all tasks (useful for serialization/UI status reporting)
    pub fn get_task_snapshots(&self) -> Vec<TaskSnapshot> {
        let lock = self.tasks.lock().unwrap();
        lock.values()
            .map(|t| TaskSnapshot {
                id: t.id.clone(),
                goal: t.goal.clone(),
                status: format!("{:?}", t.status),
                priority: format!("{:?}", t.priority),
                current_step: t.current_step,
                total_steps: t.total_steps,
                elapsed_seconds: t.start_time.elapsed().as_secs(),
                remaining_time_seconds: t.remaining_time_seconds,
                affected_files: t.affected_files.clone(),
            })
            .collect()
    }

    /// Retrieve summary of currently running or paused task
    pub fn get_running_task_summary(&self) -> String {
        let lock = self.tasks.lock().unwrap();
        let active_tasks: Vec<&Task> = lock
            .values()
            .filter(|t| t.status == TaskStatus::Running || t.status == TaskStatus::Paused)
            .collect();

        if active_tasks.is_empty() {
            "I am currently idle and ready to assist.".to_string()
        } else {
            let mut summary = Vec::new();
            for t in active_tasks {
                let state = if t.status == TaskStatus::Paused { "paused" } else { "running" };
                let step_info = if t.total_steps > 0 {
                    format!(" (step {} of {})", t.current_step, t.total_steps)
                } else {
                    String::new()
                };
                summary.push(format!("I am {} working on: '{}'{}", state, t.goal, step_info));
            }
            summary.join("\n")
        }
    }
}
