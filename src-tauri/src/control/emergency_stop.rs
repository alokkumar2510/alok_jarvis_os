use tauri::{AppHandle, Manager, Emitter};
use crate::AppState;
use crate::control::task_manager::{TASK_MANAGER, TaskStatus};
use crate::control::task_cancellation::cancel_and_rollback_task;
use crate::command_arbitrator::ARBITRATOR;

/// List of emergency stop keywords
pub const EMERGENCY_KEYWORDS: &[&str] = &[
    "stop everything",
    "emergency stop",
    "abort",
    "shut it down",
];

/// Checks if a text input matches any emergency stop commands
pub fn is_emergency_command(text: &str) -> bool {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();
    EMERGENCY_KEYWORDS.iter().any(|&k| trimmed == k || trimmed.contains(k))
}

/// Execute emergency stop procedures immediately
pub fn trigger_emergency_stop(app: &AppHandle) -> Result<(), String> {
    println!("EmergencyStop: Triggered Emergency Stop!");
    
    // 1. Stop Speech Synthesis (TTS) queue immediately
    crate::voice::tts::stop();
    println!("EmergencyStop: Purged Speech synthesis queue.");

    // 2. Cancel active Planner plans
    if let Some(state) = app.try_state::<AppState>() {
        state.planner.cancel_plan();
        println!("EmergencyStop: Cancelled planner engine.");

        // 3. Set Voice State back to Idle
        if let Ok(mut vs) = state.voice_state.lock() {
            *vs = crate::voice::VoiceState::Idle;
        }
    }

    // 4. Cancel and Rollback ALL active tasks in the TaskManager
    let active_task_ids: Vec<String> = {
        let snapshots = TASK_MANAGER.get_task_snapshots();
        snapshots.into_iter()
            .filter(|t| t.status == "Running" || t.status == "Paused")
            .map(|t| t.id)
            .collect()
    };

    for id in active_task_ids {
        println!("EmergencyStop: Cancelling and rolling back active task '{}'", id);
        if let Err(e) = cancel_and_rollback_task(&id) {
            eprintln!("EmergencyStop error rolling back task '{}': {}", id, e);
        }
    }

    // 5. Clear all active/queued commands in the Command Arbitrator
    ARBITRATOR.clear_all();

    // 6. Notify the user interface of state change to idle
    let _ = app.emit("voice-state-change", serde_json::json!({
        "state": "idle",
        "transcript": "",
        "message": "Emergency Stop Triggered. Swarms and TTS halted."
    }));

    let _ = app.emit("dialogue-event", serde_json::json!({
        "sender": "Alok",
        "message": "Acknowledged. Emergency stop triggered. Halted all active tasks."
    }));

    // Speak a brief confirmation
    let _ = crate::voice::tts::speak("Emergency stop acknowledged. All operations halted.");

    Ok(())
}
