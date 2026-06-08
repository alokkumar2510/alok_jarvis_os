use tauri::{AppHandle, Manager, Emitter};
use crate::AppState;
use crate::control::task_manager::{TASK_MANAGER, TaskStatus};
use crate::control::task_cancellation::cancel_and_rollback_task;

/// Called immediately when the wake-word VAD is triggered
pub fn interrupt_on_wakeword(app: &AppHandle) {
    println!("InterruptEngine: Interrupting on wake-word...");

    // 1. Immediately stop current TTS/speech
    crate::voice::tts::stop();

    // 2. Stop speaking animation by updating voice loop to Listening
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut vs) = state.voice_state.lock() {
            *vs = crate::voice::VoiceState::Listening;
        }
    }

    // 3. Pause all currently running tasks (P2, P3, etc.)
    TASK_MANAGER.pause_all_running();

    // 4. Emit event to UI to transition from speaking to listening
    let _ = app.emit("voice-state-change", serde_json::json!({
        "state": "listening",
        "transcript": "",
        "message": "Listening..."
    }));
}

/// Redirects the active workflow by discarding the old goal (running rollback if needed)
/// and starting a new goal.
pub async fn redirect_workflow(app: &AppHandle, new_goal: &str) -> Result<(), String> {
    println!("InterruptEngine: Redirecting workflow to: '{}'", new_goal);

    // 1. Find the paused or running task to cancel
    let active_task_id = TASK_MANAGER.get_active_task_id();
    
    if let Some(ref task_id) = active_task_id {
        println!("InterruptEngine: Discarding and rolling back previous task '{}'", task_id);
        
        let _ = app.emit("dialogue-event", serde_json::json!({
            "sender": "Alok",
            "message": format!("Discarding previous task: '{}'. Rolling back changes.", task_id)
        }));

        // Cancel and rollback the previous task
        if let Err(e) = cancel_and_rollback_task(task_id) {
            eprintln!("InterruptEngine error rolling back task '{}': {}", task_id, e);
        }
    }

    // 2. Start the new goal
    let state = app.state::<AppState>();
    
    let db_clone = state.db.clone();
    let app_clone = app.clone();
    let goal_str = new_goal.to_string();

    // Spawn the new swarm execution/planner thread
    tauri::async_runtime::spawn(async move {
        let _ = crate::agents::AgentFramework::execute_agent_goal(&goal_str, db_clone, app_clone).await;
    });

    let _ = app.emit("dialogue-event", serde_json::json!({
        "sender": "Alok",
        "message": format!("Starting new task: '{}'", new_goal)
    }));

    Ok(())
}
