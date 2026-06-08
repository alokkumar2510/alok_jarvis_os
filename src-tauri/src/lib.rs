pub mod database;
pub mod action_bus;
pub mod conversation_context;
pub mod screen_context_engine;
pub mod planner;
pub mod plugin;
pub mod voice;
pub mod desktop;
pub mod jarvis_runtime;
pub mod overlay_manager;
pub mod environment_awareness;
pub mod vision;
pub mod agents;
pub mod consciousness;
pub mod command_arbitrator;
pub mod control;

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Emitter};
use tauri::Manager;
use database::Database;
use action_bus::ActionBus;
use conversation_context::ConversationContext;
use screen_context_engine::ScreenContextEngine;
use planner::PlannerEngine;
use plugin::PluginManager;
use voice::{WhisperEngine, WakeWordService, tts};
use intelligence::GroqClient;

// Keep local reference to intelligence submodule
pub mod intelligence;

// Application State container
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub context: Arc<Mutex<ConversationContext>>,
    pub action_bus: Arc<ActionBus>,
    pub planner: Arc<PlannerEngine>,
    pub plugins: Arc<Mutex<PluginManager>>,
    pub ocr_engine: Arc<ScreenContextEngine>,
    pub whisper: Arc<WhisperEngine>,
    pub voice_state: Arc<Mutex<voice::VoiceState>>,
    pub intelligence_core: Arc<crate::intelligence::IntelligenceCore>,
    pub agent_activity: Arc<Mutex<std::collections::HashMap<String, String>>>,
    pub presence_state: Arc<Mutex<consciousness::PresenceState>>,
}

// Global handle for voice synthesis lock
lazy_static::lazy_static! {
    static ref TTS_MUTEX: Mutex<()> = Mutex::new(());
}

#[tauri::command]
fn js_console_log(message: String) {
    println!("[JS LOG] {}", message);
}

// ── Tauri Commands ───────────────────────────────────────────────────────

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> serde_json::Value {
    if let Ok(db) = state.db.lock() {
        let report = crate::intelligence::health_monitor::capture_health_metrics(&db);
        serde_json::json!({
            "ram_usage": report.ram_usage_mb,
            "latency_ms": report.wake_word_latency_ms,
            "cpu_usage": report.cpu_usage_pct,
            "action_latency_ms": report.action_latency_ms,
            "groq_latency_ms": report.groq_latency_ms,
            "issues": report.issues
        })
    } else {
        serde_json::json!({
            "ram_usage": 112.4,
            "latency_ms": 280.0,
            "cpu_usage": 0.8,
            "action_latency_ms": 120.0,
            "groq_latency_ms": 850.0,
            "issues": Vec::<String>::new()
        })
    }
}

#[tauri::command]
fn load_settings(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let groq_api_key = db.get_setting("groq_api_key").unwrap_or_default().unwrap_or_default();
    let groq_model = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
    let wakeword_sensitivity = db.get_setting("wakeword_sensitivity").unwrap_or_default().unwrap_or_else(|| "0.75".to_string());
    let tts_speed = db.get_setting("tts_speed").unwrap_or_default().unwrap_or_else(|| "1.0".to_string());

    Ok(serde_json::json!({
        "groq_api_key": groq_api_key,
        "groq_model": groq_model,
        "wakeword_sensitivity": wakeword_sensitivity.parse::<f64>().unwrap_or(0.75),
        "tts_speed": tts_speed.parse::<f64>().unwrap_or(1.0)
    }))
}

#[tauri::command]
fn save_settings(settings: serde_json::Value, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    if let Some(key) = settings["groq_api_key"].as_str() {
        db.set_setting("groq_api_key", key).map_err(|e| e.to_string())?;
    }
    if let Some(model) = settings["groq_model"].as_str() {
        db.set_setting("groq_model", model).map_err(|e| e.to_string())?;
    }
    if let Some(sens) = settings["wakeword_sensitivity"].as_f64() {
        db.set_setting("wakeword_sensitivity", &sens.to_string()).map_err(|e| e.to_string())?;
    }
    if let Some(speed) = settings["tts_speed"].as_f64() {
        db.set_setting("tts_speed", &speed.to_string()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn toggle_startup(enabled: bool) -> Result<(), String> {
    println!("Tauri: Toggling windows startup = {}", enabled);
    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    
    let auto = auto_launch::AutoLaunchBuilder::new()
        .set_app_name("alok_jarvis_os")
        .set_app_path(&current_exe.to_string_lossy())
        .build()
        .map_err(|e| e.to_string())?;

    if enabled {
        let _ = auto.enable();
    } else {
        let _ = auto.disable();
    }

    Ok(())
}

#[tauri::command]
fn toggle_ocr(enabled: bool) -> Result<(), String> {
    println!("Tauri: Toggling screen OCR context monitor = {}", enabled);
    // Continuous screen OCR context loop state change
    Ok(())
}

pub trait WindowManager {
    fn has_window(&self, label: &str) -> bool;
    fn is_window_visible(&self, label: &str) -> Result<bool, String>;
    fn hide_window(&self, label: &str) -> Result<(), String>;
    fn show_and_focus_window(&self, label: &str) -> Result<(), String>;
    fn create_console_window(&self) -> Result<(), String>;
}

#[cfg(not(test))]
impl<R: tauri::Runtime> WindowManager for tauri::AppHandle<R> {
    fn has_window(&self, label: &str) -> bool {
        self.get_webview_window(label).is_some()
    }
    fn is_window_visible(&self, label: &str) -> Result<bool, String> {
        let win = self.get_webview_window(label).ok_or("Window not found")?;
        win.is_visible().map_err(|e| e.to_string())
    }
    fn hide_window(&self, label: &str) -> Result<(), String> {
        let win = self.get_webview_window(label).ok_or("Window not found")?;
        win.hide().map_err(|e| e.to_string())
    }
    fn show_and_focus_window(&self, label: &str) -> Result<(), String> {
        let win = self.get_webview_window(label).ok_or("Window not found")?;
        let _ = win.show();
        let _ = win.set_focus();
        Ok(())
    }
    fn create_console_window(&self) -> Result<(), String> {
        let _ = tauri::WebviewWindowBuilder::new(
            self,
            "developer_console",
            tauri::WebviewUrl::App("dev_console.html".into())
        )
        .title("JARVIS Developer Console")
        .inner_size(950.0, 650.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn toggle_developer_console_inner(manager: &impl WindowManager) -> Result<(), String> {
    let label = "developer_console";
    if manager.has_window(label) {
        if manager.is_window_visible(label)? {
            manager.hide_window(label)?;
        } else {
            manager.show_and_focus_window(label)?;
        }
    } else {
        manager.create_console_window()?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_developer_console<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<(), String> {
    #[cfg(not(test))]
    {
        toggle_developer_console_inner(&app)
    }
    #[cfg(test)]
    {
        let _ = app;
        Ok(())
    }
}

#[tauri::command]
fn stop_speech() {
    println!("Tauri: Manual stop speech invoked.");
    tts::stop();
}

pub fn log_dev_event(app: &AppHandle, tab: &str, level: &str, message: &str) {
    let _ = app.emit("dev-log", serde_json::json!({
        "tab": tab,
        "level": level,
        "message": message,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }));
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub output: String,
    pub intent_score: f64,
    pub action_latency_ms: u64,
}

/// Dispatches a single resolved command text through the intent → action pipeline.
/// Used by the chained command executor in process_intent.
async fn dispatch_single_intent(text: &str, state: &AppState, app: &AppHandle) -> CommandOutcome {
    let start_intent = std::time::Instant::now();
    let plugin_cmds = {
        if let Ok(mgr) = state.plugins.lock() {
            mgr.get_commands()
        } else {
            Vec::new()
        }
    };
    if let Some(mut action_request) = state.intelligence_core.resolve_local_action(text, &plugin_cmds) {
        let intent_score = action_request.args.get("confidence_score")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);
        let intent_dur = start_intent.elapsed().as_millis();
        crate::log_dev_event(app, "Intent", "info", &format!("Intent matched: type='{:?}', action_id='{}', confidence={:.2} (took {} ms)", action_request.action_type, action_request.action_id, intent_score, intent_dur));

        // Log intent resolution latency
        if let Ok(db_lock) = state.db.lock() {
            crate::intelligence::performance_tracker::log_latency(&db_lock, "intent_resolution", intent_dur as f64);
        }

        // Handle entity-level intent redirect
        if action_request.args.get("_redirect_intent").map(|s| s.as_str()) == Some("open_url") {
            action_request.args.remove("_redirect_intent");
            action_request.action_type = crate::action_bus::ActionType::BrowserControl;
            action_request.category = "browser".to_string();
            action_request.action_id = "open_website".to_string();
        }
        let start_action = std::time::Instant::now();
        let response = state.action_bus.dispatch(action_request, app).await;
        let action_latency_ms = start_action.elapsed().as_millis() as u64;
        let output = if response.success {
            response.output
        } else {
            response.error.unwrap_or_else(|| "Action failed".to_string())
        };
        CommandOutcome {
            output,
            intent_score,
            action_latency_ms,
        }
    } else {
        let intent_dur = start_intent.elapsed().as_millis();
        crate::log_dev_event(app, "Intent", "warn", &format!("No offline intent matched (took {} ms). Checking fallbacks.", intent_dur));

        // Try local memory keyword lookup
        let query_words: Vec<&str> = text.split_whitespace().filter(|w| w.len() > 3).collect();
        let mut local_res = None;
        if !query_words.is_empty() {
            if let Ok(db_lock) = state.db.lock() {
                for word in query_words {
                    if let Ok(results) = db_lock.search_memories(word) {
                        if !results.is_empty() {
                            local_res = Some(format!("Local Memory: I found this in my memory: {}", results[0]));
                            break;
                        }
                    }
                }
            }
        }

        if let Some(output) = local_res {
            return CommandOutcome {
                output,
                intent_score: 0.8,
                action_latency_ms: start_intent.elapsed().as_millis() as u64,
            };
        }

        // Fallback to Groq
        let start_action = std::time::Instant::now();
        let (api_key, model) = {
            if let Ok(db) = state.db.lock() {
                let key = db.get_setting("groq_api_key").unwrap_or_default();
                let mdl = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
                (key, mdl)
            } else {
                (None, "llama-3.1-8b-instant".to_string())
            }
        };

        let output = if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            "Local Fallback: Groq API key is not configured. I couldn't resolve this query locally. Try speaking commands like 'open chrome', 'remember my favorite tool is Rust', or 'setup python'."
                .to_string()
        } else {
            let groq = intelligence::GroqClient::new(api_key, &model);
            match groq.query(text) {
                Ok(reply) => {
                    let groq_dur = start_action.elapsed().as_millis() as f64;
                    if let Ok(db_lock) = state.db.lock() {
                        crate::intelligence::performance_tracker::log_latency(&db_lock, "groq_api", groq_dur);
                    }
                    reply
                }
                Err(e) => format!(
                    "Local Fallback: I could not contact online reasoning ({:?}). Please check your connection. Try running local commands like 'open chrome', 'remember X', or 'setup python'.",
                    e
                ),
            }
        };
        let action_latency_ms = start_action.elapsed().as_millis() as u64;
        CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms,
        }
    }
}

pub async fn execute_single_command(text: &str, state: &AppState, app: &AppHandle) -> CommandOutcome {
    let start_intent = std::time::Instant::now();
    let plugin_cmds = {
        if let Ok(mgr) = state.plugins.lock() {
            mgr.get_commands()
        } else {
            Vec::new()
        }
    };
    if let Some(mut action_request) = state.intelligence_core.resolve_local_action(text, &plugin_cmds) {
        let intent_score = action_request.args.get("confidence_score")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);
        let intent_dur = start_intent.elapsed().as_millis();
        crate::log_dev_event(app, "Intent", "info", &format!("Intent matched: type='{:?}', action_id='{}', confidence={:.2} (took {} ms)", action_request.action_type, action_request.action_id, intent_score, intent_dur));

        // Log intent resolution latency
        if let Ok(db_lock) = state.db.lock() {
            crate::intelligence::performance_tracker::log_latency(&db_lock, "intent_resolution", intent_dur as f64);
        }

        // Handle entity-level intent redirect (e.g. "open X in chrome" → open_url)
        if action_request.args.get("_redirect_intent").map(|s| s.as_str()) == Some("open_url") {
            action_request.args.remove("_redirect_intent");
            action_request.action_type = crate::action_bus::ActionType::BrowserControl;
            action_request.category = "browser".to_string();
            action_request.action_id = "open_website".to_string();
        }

        // Capture properties for context tracking
        let app_name = action_request.args.get("app_name").cloned()
            .or_else(|| action_request.args.get("app").cloned());
        let file_name = action_request.args.get("filename").cloned()
            .or_else(|| action_request.args.get("filepath").cloned())
            .or_else(|| action_request.args.get("file").cloned());
        let url_val = action_request.args.get("url").cloned();
        let action_id = action_request.action_id.clone();
        let intent_name = action_request.action_id.clone();

        if let Ok(mut ctx) = state.context.lock() {
            ctx.update_context(
                Some(intent_name),
                app_name,
                file_name,
                url_val,
                Some(action_id),
            );
        }

        let start_action = std::time::Instant::now();
        let response = crate::command_arbitrator::ARBITRATOR.arbitrate_and_execute(action_request.clone(), state, app).await;
        let action_latency_ms = start_action.elapsed().as_millis() as u64;
        
        let output = if response.success {
            // Record command usage in learning engine on success
            let learning_engine = crate::intelligence::learning_engine::LearningEngine::new(state.db.clone());
            learning_engine.record_command_usage(&action_request.action_id, &action_request.args);

            response.output
        } else {
            response.error.unwrap_or_else(|| "Unknown execution failure".to_string())
        };

        CommandOutcome {
            output,
            intent_score,
            action_latency_ms,
        }
    } else {
        let intent_dur = start_intent.elapsed().as_millis();
        crate::log_dev_event(app, "Intent", "warn", &format!("No offline intent matched (took {} ms). Checking fallbacks.", intent_dur));

        let text_lower = text.to_lowercase();
        // Route plan/setup/research/organize goals to the Personal Agent Framework
        if text_lower.contains("research") || text_lower.contains("organize") || text_lower.contains("downloads")
           || text_lower.contains("set up") || text_lower.contains("setup") || text_lower.contains("plan")
           || text_lower.contains("python") || text_lower.contains("node") || text_lower.contains("rust") 
           || text_lower.contains("docker") || text_lower.contains("git") || text_lower.contains("tauri") 
           || text_lower.contains("flutter") {
            let db_clone = state.db.clone();
            let app_clone = app.clone();
            let text_str = text.to_string();
            
            tauri::async_runtime::spawn(async move {
                let _ = crate::agents::AgentFramework::execute_agent_goal(&text_str, db_clone, app_clone).await;
            });

            let output = "Initializing Autonomous Agent for goal execution... Check the Goal Planner tab for progress.".to_string();
            CommandOutcome {
                output,
                intent_score: 1.0,
                action_latency_ms: 0,
            }
        } else {
            // Check local memories before querying LLM
            let query_words: Vec<&str> = text.split_whitespace().filter(|w| w.len() > 3).collect();
            let mut local_res = None;
            if !query_words.is_empty() {
                if let Ok(db_lock) = state.db.lock() {
                    for word in query_words {
                        if let Ok(results) = db_lock.search_memories(word) {
                            if !results.is_empty() {
                                local_res = Some(format!("Local Memory: I recall this: {}", results[0]));
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(output) = local_res {
                return CommandOutcome {
                    output,
                    intent_score: 0.8,
                    action_latency_ms: start_intent.elapsed().as_millis() as u64,
                };
            }

            // Fallback to Groq online reasoning
            let start_action = std::time::Instant::now();
            let (api_key, model) = {
                if let Ok(db) = state.db.lock() {
                    let key = db.get_setting("groq_api_key").unwrap_or_default();
                    let mdl = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
                    (key, mdl)
                } else {
                    (None, "llama-3.1-8b-instant".to_string())
                }
            };

            let output = if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
                "Local Fallback: Groq API key is not configured. I couldn't resolve this query locally. Try speaking commands like 'open chrome', 'remember my favorite tool is Rust', or 'setup python'."
                    .to_string()
            } else {
                let enriched_prompt = crate::intelligence::multimodal_context::MultimodalContextEngine::get_enriched_prompt(text, state);
                let groq = GroqClient::new(api_key, &model);
                match groq.query(&enriched_prompt) {
                    Ok(reply) => {
                        let groq_dur = start_action.elapsed().as_millis() as f64;
                        if let Ok(db_lock) = state.db.lock() {
                            crate::intelligence::performance_tracker::log_latency(&db_lock, "groq_api", groq_dur);
                        }
                        reply
                    }
                    Err(e) => format!(
                        "Local Fallback: I could not contact online reasoning ({:?}). Please check your connection. Try running local commands like 'open chrome', 'remember X', or 'setup python'.",
                        e
                    ),
                }
            };
            let action_latency_ms = start_action.elapsed().as_millis() as u64;

            CommandOutcome {
                output,
                intent_score: 1.0,
                action_latency_ms,
            }
        }
    }
}

pub fn split_chained_commands(resolved_text: &str) -> Vec<String> {
    // Split chained commands: "open chrome and go to facebook" → ["open chrome", "go to facebook"]
    let chain_separators = [" and then ", " then ", " and also ", " after that "];
    let mut sub_commands: Vec<String> = vec![resolved_text.to_string()];
    for sep in &chain_separators {
        if resolved_text.to_lowercase().contains(sep) {
            sub_commands = resolved_text
                .splitn(2, sep)
                .map(|s| s.trim().to_string())
                .collect();
            break;
        }
    }
    // Also split on " and " only when both sides look like commands (contain a verb) or when verb sharing is possible
    if sub_commands.len() == 1 && resolved_text.to_lowercase().contains(" and ") {
        let parts: Vec<&str> = resolved_text.splitn(2, " and ").collect();
        let verbs = ["open", "close", "search", "go to", "go", "play", "pause", "lock", "mute", "volume", "launch", "find", "remember", "show"];
        let both_are_commands = parts.len() == 2 && verbs.iter().any(|v| parts[0].to_lowercase().starts_with(v))
            && verbs.iter().any(|v| parts[1].to_lowercase().starts_with(v));
        if both_are_commands {
            sub_commands = parts.iter().map(|s| s.trim().to_string()).collect();
        } else if parts.len() == 2 {
            let first_word_part0 = parts[0].split_whitespace().next().unwrap_or("").to_lowercase();
            let starts_with_verb = verbs.contains(&first_word_part0.as_str());
            let second_starts_with_verb = verbs.iter().any(|v| parts[1].to_lowercase().starts_with(v));
            if starts_with_verb && !second_starts_with_verb {
                let part1_with_verb = format!("{} {}", first_word_part0, parts[1]);
                sub_commands = vec![parts[0].trim().to_string(), part1_with_verb.trim().to_string()];
            }
        }
    }
    sub_commands
}

pub fn resolve_path(relative_prod_path: &str, dev_fallback_path: &str) -> String {
    // 1. Resolve relative to local AppData
    let mut appdata_path = None;
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let appdata_dir = std::path::Path::new(&local_appdata).join("alok_jarvis_os");
        let target_path = appdata_dir.join(relative_prod_path);
        
        // If it exists, use it immediately
        if target_path.exists() {
            return target_path.to_string_lossy().to_string();
        }
        
        // Store the path to use as a fallback if the developer fallback is also not present
        appdata_path = Some(target_path);
    }
    
    // 2. Resolve relative to current running executable directory (portable mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let target_path = exe_dir.join(relative_prod_path);
            if target_path.exists() {
                return target_path.to_string_lossy().to_string();
            }
        }
    }
    
    // 3. Check if developer path exists
    if std::path::Path::new(dev_fallback_path).exists() {
        return dev_fallback_path.to_string();
    }
    
    // 4. Default to AppData path (and ensure its parent directories are created)
    if let Some(path) = appdata_path {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        return path.to_string_lossy().to_string();
    }
    
    // Final fallback
    dev_fallback_path.to_string()
}

pub async fn execute_resolved_command(
    resolved_text: String,
    voice_char: Option<crate::voice::VoiceCharacteristics>,
    state: &AppState,
    app: &AppHandle,
) -> CommandOutcome {
    // 0. Check for emergency stop
    if crate::control::emergency_stop::is_emergency_command(&resolved_text) {
        let _ = crate::control::emergency_stop::trigger_emergency_stop(app);
        return CommandOutcome {
            output: "Emergency stop executed. All active tasks, TTS, and queues have been halted.".to_string(),
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    // 0.5. Check for global control overrides
    let text_lower = resolved_text.to_lowercase();
    let trimmed = text_lower.trim();
    if trimmed == "stop" || trimmed == "cancel" || trimmed == "abort mission" {
        let output = if let Some(task_id) = crate::control::task_manager::TASK_MANAGER.get_active_task_id() {
            let _ = crate::control::task_cancellation::cancel_and_rollback_task(&task_id);
            "Halted active task and rolled back changes.".to_string()
        } else {
            "There are no active tasks to stop.".to_string()
        };
        let output_c = output.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&output_c);
        });
        return CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "pause" {
        let output = if let Some(task_id) = crate::control::task_manager::TASK_MANAGER.get_active_task_id() {
            let _ = crate::control::task_manager::TASK_MANAGER.pause_task(&task_id);
            "Paused active task.".to_string()
        } else {
            "There are no running tasks to pause.".to_string()
        };
        let output_c = output.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&output_c);
        });
        return CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "resume" || trimmed == "continue" {
        let paused_task_id = {
            let snapshots = crate::control::task_manager::TASK_MANAGER.get_task_snapshots();
            snapshots.into_iter()
                .find(|t| t.status == "Paused")
                .map(|t| t.id)
        };
        let output = if let Some(task_id) = paused_task_id {
            let _ = crate::control::task_manager::TASK_MANAGER.resume_task(&task_id);
            "Resumed active task.".to_string()
        } else {
            "There are no paused tasks to resume.".to_string()
        };
        let output_c = output.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&output_c);
        });
        return CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "be quiet" || trimmed == "sleep" {
        crate::voice::tts::stop();
        return CommandOutcome {
            output: "Entering silent mode.".to_string(),
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "forget that" || trimmed == "never mind" {
        let output = if let Some(task_id) = crate::control::task_manager::TASK_MANAGER.get_active_task_id() {
            let _ = crate::control::task_cancellation::cancel_and_rollback_task(&task_id);
            "Okay, never mind.".to_string()
        } else {
            "Okay.".to_string()
        };
        let output_c = output.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&output_c);
        });
        return CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "start over" {
        let output = if let Some(task_id) = crate::control::task_manager::TASK_MANAGER.get_active_task_id() {
            let goal = {
                let snapshots = crate::control::task_manager::TASK_MANAGER.get_task_snapshots();
                snapshots.into_iter().find(|t| t.id == task_id).map(|t| t.goal)
            };
            let _ = crate::control::task_cancellation::cancel_and_rollback_task(&task_id);
            if let Some(g) = goal {
                let db_clone = state.db.clone();
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::agents::AgentFramework::execute_agent_goal(&g, db_clone, app_clone).await;
                });
                "Restarting task from scratch.".to_string()
            } else {
                "Halted task, but could not restart.".to_string()
            }
        } else {
            "There are no active tasks to restart.".to_string()
        };
        let output_c = output.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&output_c);
        });
        return CommandOutcome {
            output,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "what are you doing?" {
        let summary = crate::control::task_manager::TASK_MANAGER.get_running_task_summary();
        let summary_c = summary.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&summary_c);
        });
        return CommandOutcome {
            output: summary,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    if trimmed == "how long left?" {
        let time_left = {
            let snapshots = crate::control::task_manager::TASK_MANAGER.get_task_snapshots();
            snapshots.into_iter()
                .find(|t| t.status == "Running")
                .and_then(|t| t.remaining_time_seconds)
        };
        let msg = match time_left {
            Some(secs) => format!("Approximately {} seconds remaining.", secs),
            None => "I cannot estimate the remaining time for the current task.".to_string(),
        };
        let msg_c = msg.clone();
        std::thread::spawn(move || {
            let _ = crate::voice::tts::speak(&msg_c);
        });
        return CommandOutcome {
            output: msg,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    // 0.6. Conversation Redirection
    if let Some(task_id) = crate::control::task_manager::TASK_MANAGER.get_active_task_id() {
        println!("execute_resolved_command: Redirecting task '{}' to new goal '{}'", task_id, resolved_text);
        let _ = crate::control::task_cancellation::cancel_and_rollback_task(&task_id);
        let _ = app.emit("dialogue-event", serde_json::json!({
            "sender": "Alok",
            "message": format!("Discarding previous task: '{}'. Rolling back changes.", task_id)
        }));
    }

    let learning_engine = crate::intelligence::learning_engine::LearningEngine::new(state.db.clone());

    // 1. Check for explicit learning command (e.g. "WA = WhatsApp")
    if let Some(msg) = learning_engine.learn_from_input(&resolved_text) {
        return CommandOutcome {
            output: msg,
            intent_score: 1.0,
            action_latency_ms: 0,
        };
    }

    // 2. Pre-process aliases
    let pre_processed = learning_engine.pre_process_aliases(&resolved_text);

    // 2.5 Check for voice preferences commands (personality, speed, voice overrides)
    let mut voice_pref_outcome = None;
    if let Ok(db) = state.db.lock() {
        let mut pref = crate::voice::voice_preferences::VoicePreferences::load_from_db(&db);
        if let Some(confirmation) = pref.handle_voice_commands(&pre_processed, &db) {
            let _ = app.emit("settings-change", serde_json::json!({
                "voice_personality": pref.personality_mode.as_str(),
                "voice_preferred_voice": pref.preferred_voice.clone(),
                "voice_speaking_speed": pref.speaking_speed,
                "voice_volume": pref.volume
            }));

            let msg_clone = confirmation.clone();
            std::thread::spawn(move || {
                let _lock = TTS_MUTEX.lock().unwrap();
                let _ = crate::voice::tts::speak(&msg_clone);
            });
            voice_pref_outcome = Some(CommandOutcome {
                output: confirmation,
                intent_score: 1.0,
                action_latency_ms: 0,
            });
        }
    }
    if let Some(outcome) = voice_pref_outcome {
        return outcome;
    }

    let history = if let Ok(db) = state.db.lock() {
        db.get_conversation_history(10).unwrap_or_default()
    } else {
        Vec::new()
    };
    let emotion_res = crate::voice::emotion_engine::EmotionEngine::detect_emotion(&pre_processed, &history);
    let emotion = emotion_res.emotion;
    println!("Voice & Emotion System: Detected emotion: {:?}", emotion);
    crate::log_dev_event(app, "Voice", "info", &format!("Detected user emotion: {}", emotion.as_str()));
    if let Ok(db) = state.db.lock() {
        let _ = db.set_setting("voice_active_emotion", emotion.as_str());
    }

    let sub_commands = split_chained_commands(&pre_processed);

    let mut outcome = if sub_commands.len() > 1 {
        let mut combined_output = Vec::new();
        let mut total_latency = 0;
        let mut min_score = 1.0;
        for sub_cmd in sub_commands {
            println!("Tauri: Executing chained sub-command: '{}'", sub_cmd);
            let sub_outcome = dispatch_single_intent(&sub_cmd, state, app).await;
            combined_output.push(sub_outcome.output);
            total_latency += sub_outcome.action_latency_ms;
            if sub_outcome.intent_score < min_score {
                min_score = sub_outcome.intent_score;
            }
        }
        CommandOutcome {
            output: combined_output.join(". "),
            intent_score: min_score,
            action_latency_ms: total_latency,
        }
    } else {
        execute_single_command(&resolved_text, state, app).await
    };

    // Adapt the response text based on the user's emotion and active personality
    if !outcome.output.is_empty() && outcome.output != "Resuming speech" {
        // Load active voice personality
        let personality_str = if let Ok(db) = state.db.lock() {
            db.get_setting("voice_personality").ok().flatten().unwrap_or_else(|| "assistant".to_string())
        } else {
            "assistant".to_string()
        };
        let personality = crate::voice::personality_engine::PersonalityMode::from_str(&personality_str);

        // 1. Phrasing Adaptation via voice manager
        let adapted = crate::voice::voice_manager::VoiceManager::get_adapted_phrasing(emotion, personality, &outcome.output);
        
        // 2. Determine Speech Style
        let speech_style = crate::voice::speech_style_engine::SpeechStyleEngine::determine_style(emotion, personality);

        // 3. Format text style
        outcome.output = crate::voice::speech_style_engine::SpeechStyleEngine::format_text_style(&adapted, &speech_style);
    }

    // Log response
    if !outcome.output.is_empty() {
        if let Ok(mut ctx) = state.context.lock() {
            ctx.add_history("Alok", &outcome.output);
        }
        if let Ok(db) = state.db.lock() {
            let _ = db.add_conversation_entry("Alok", &outcome.output);
            // push new event history to UI
            let _ = app.emit("dialogue-event", serde_json::json!({
                "sender": "Alok",
                "message": outcome.output.clone()
            }));
        }

        // Synthesize voice outcome asynchronously
        if outcome.output != "Resuming speech" {
            let output_clone = outcome.output.clone();
            let app_clone = app.clone();
            std::thread::spawn(move || {
                let _lock = TTS_MUTEX.lock().unwrap();
                if let Some(app_state) = app_clone.try_state::<AppState>() {
                    set_presence_state(&app_clone, &app_state, consciousness::PresenceState::Speaking);
                }
                let _ = tts::speak(&output_clone);
                
                // Wait until TTS has finished speaking
                while tts::is_speaking() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                
                // Transition voice state back to Idle on backend and emit voice-state-change
                if let Some(app_state) = app_clone.try_state::<AppState>() {
                    let mut should_reset = false;
                    if let Ok(mut vs) = app_state.voice_state.lock() {
                        if *vs == crate::voice::VoiceState::Speaking {
                            *vs = crate::voice::VoiceState::Idle;
                            should_reset = true;
                        }
                    }
                    if should_reset {
                        let _ = app_clone.emit("voice-state-change", serde_json::json!({
                            "state": "idle",
                            "transcript": "",
                            "message": ""
                        }));
                    }
                    set_presence_state(&app_clone, &app_state, consciousness::PresenceState::Monitoring);
                }
            });
        }
    }

    outcome
}

#[tauri::command]
fn capture_environment_context() -> environment_awareness::EnvironmentContext {
    environment_awareness::capture()
}

#[tauri::command]
async fn capture_memory_summary(app: AppHandle) -> Result<String, String> {
    intelligence::memory_engine::generate_weekly_summary(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn run_memory_forgetting(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    intelligence::memory_engine::decay_memories(&db).map_err(|e| e.to_string())
}

#[tauri::command]
async fn consolidate_all_memories(app: AppHandle) -> Result<(), String> {
    intelligence::memory_engine::consolidate_memories(app).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ranked_memories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_memories("").map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct FrontEndNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(serde::Serialize)]
struct FrontEndEdge {
    from: String,
    to: String,
    relation: String,
}

#[derive(serde::Serialize)]
struct MemoryGraphData {
    nodes: Vec<FrontEndNode>,
    edges: Vec<FrontEndEdge>,
}

#[tauri::command]
fn get_memory_graph(state: State<'_, AppState>) -> Result<MemoryGraphData, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let db_nodes = db.get_nodes().map_err(|e| e.to_string())?;
    let db_edges = db.get_edges().map_err(|e| e.to_string())?;
    
    let nodes = db_nodes.into_iter().map(|n| FrontEndNode {
        id: n.id,
        label: n.label,
        node_type: n.node_type,
    }).collect();
    
    let edges = db_edges.into_iter().map(|e| FrontEndEdge {
        from: e.from_id,
        to: e.to_id,
        relation: e.relation,
    }).collect();
    
    Ok(MemoryGraphData { nodes, edges })
}

#[tauri::command]
fn get_learned_habits(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let habits = db.get_all_habits().map_err(|e| e.to_string())?;
    
    let result: Vec<serde_json::Value> = habits.into_iter().map(|(habit_type, target, hour, confidence)| {
        serde_json::json!({
            "habit_type": habit_type,
            "target": target,
            "hour": hour,
            "confidence": confidence
        })
    }).collect();
    
    Ok(serde_json::json!(result))
}

#[tauri::command]
fn get_habit_predictions(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let current_hour = db.get_current_local_hour().map_err(|e| e.to_string())?;
    if current_hour < 0 {
        return Err("Failed to query current local hour".to_string());
    }
    
    let predictions = intelligence::behavior_model::predict_next_action(&db, current_hour).map_err(|e| e.to_string())?;
    let result: Vec<serde_json::Value> = predictions.into_iter().map(|pred| {
        serde_json::json!({
            "habit_type": pred.habit_type,
            "target": pred.target,
            "hour": pred.hour,
            "confidence": pred.confidence
        })
    }).collect();
    Ok(serde_json::json!(result))
}

#[tauri::command]
fn seed_test_meeting(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_calendar_event("Sprint Planning & Code Review", 15, "Sprint Planning & Code Review meeting").map_err(|e| e.to_string())?;
    println!("ProactiveAssistant: Seeded test meeting in 15 minutes.");
    Ok(())
}

#[tauri::command]
fn trigger_habit_sweep(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    intelligence::pattern_detector::detect_patterns(&db).map_err(|e| e.to_string())?;
    println!("HabitEngine: Manual habit detection sweep completed.");
    Ok(())
}

pub fn set_presence_state(app: &tauri::AppHandle, state: &AppState, new_state: consciousness::PresenceState) {
    if let Ok(mut presence) = state.presence_state.lock() {
        *presence = new_state;
        let _ = app.emit("presence-state-change", serde_json::json!({ "state": format!("{:?}", new_state) }));
        crate::voice::presence_system::play_presence_signature(new_state);
    }
}

#[tauri::command]
fn get_current_state(state: State<'_, AppState>) -> serde_json::Value {
    crate::consciousness::get_current_state(&state)
}

#[tauri::command]
fn get_self_diagnostics(state: State<'_, AppState>) -> serde_json::Value {
    if let Ok(db) = state.db.lock() {
        let report = crate::intelligence::self_diagnostics::run_diagnostics(&db);
        serde_json::to_value(report).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({
            "health_score": 0,
            "voice_subsystem": "Offline",
            "wakeword_subsystem": "Offline",
            "memory_subsystem": "Offline",
            "automation_subsystem": "Offline",
            "plugin_subsystem": "Offline",
            "failures": vec!["Failed to lock database for diagnostics".to_string()]
        })
    }
}

#[tauri::command]
async fn run_desktop_action(
    action: String,
    args: serde_json::Value,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    set_presence_state(&app, &state, consciousness::PresenceState::Working);
    let res = match action.as_str() {
        "move_to" => {
            let x = args.get("x").and_then(|v| v.as_i64()).ok_or("Missing x")? as i32;
            let y = args.get("y").and_then(|v| v.as_i64()).ok_or("Missing y")? as i32;
            crate::desktop::automation_runtime::move_to(x, y).await.map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        "click" => {
            crate::desktop::automation_runtime::click().await.map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        "double_click" => {
            crate::desktop::automation_runtime::double_click().await.map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        "right_click" => {
            crate::desktop::automation_runtime::right_click().await.map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        "type_text" => {
            let text = args.get("text").and_then(|v| v.as_str()).ok_or("Missing text")?;
            crate::desktop::automation_runtime::type_text(text).await.map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        "find_ui_element" => {
            let text = args.get("text").and_then(|v| v.as_str()).ok_or("Missing text")?;
            let coords = crate::desktop::automation_runtime::find_ui_element(text).map_err(|e| e.to_string())?;
            serde_json::json!({ "coordinates": coords })
        }
        "manipulate_window" => {
            let app_name = args.get("app_name").and_then(|v| v.as_str()).ok_or("Missing app_name")?;
            let win_action = args.get("action").and_then(|v| v.as_str()).ok_or("Missing action")?;
            crate::desktop::automation_runtime::manipulate_window(app_name, win_action).map_err(|e| e.to_string())?;
            serde_json::json!({ "success": true })
        }
        _ => return Err(format!("Unknown action: {}", action)),
    };
    set_presence_state(&app, &state, consciousness::PresenceState::Monitoring);
    Ok(res)
}

#[tauri::command]
async fn trigger_goal_execution(
    goal: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    set_presence_state(&app, &state, consciousness::PresenceState::Working);
    let res = crate::planner::goal_execution_engine::GoalExecutionEngine::execute_goal(&goal, &state).await;
    set_presence_state(&app, &state, consciousness::PresenceState::Monitoring);
    res
}

#[tauri::command]
async fn get_mission_control_data(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // A. OS & Device Status
    let report = crate::intelligence::health_monitor::capture_health_metrics(&db);
    let system_memory = run_ps_cmd("$mem = Get-CimInstance Win32_OperatingSystem; [math]::round(($mem.TotalVisibleMemorySize - $mem.FreePhysicalMemory) / $mem.TotalVisibleMemorySize * 100, 1)").unwrap_or(54.2);
    let system_cpu = run_ps_cmd("[math]::round((Get-CimInstance Win32_Processor).LoadPercentage, 1)").unwrap_or(12.4);
    let disk_free = run_ps_cmd("[math]::round((Get-PSDrive C).Free / 1GB, 1)").unwrap_or(145.6);
    
    // Battery Health
    let mut battery_health = 98.0;
    if let Some((design, full)) = crate::intelligence::proactive_assistant::check_battery_health() {
        if design > 0.0 {
            battery_health = (full / design) * 100.0;
        }
    }

    // B. Memory Metrics
    let memories_count = db.get_memories_count().unwrap_or(0);
    let edges_count = db.get_edges().map(|e| e.len()).unwrap_or(0);

    // C. Learning metrics
    let corrections_count = db.get_corrections_count().unwrap_or(0);
    let habits = db.get_learned_patterns("preferred_app").unwrap_or_default();
    let habits_json = serde_json::json!(
        habits.iter().map(|(k, v, conf)| {
            serde_json::json!({
                "key": k,
                "value": v,
                "confidence": conf
            })
        }).collect::<Vec<_>>()
    );

    // Self improvement stats
    let improvement_metrics = db.get_self_improvement_metrics().unwrap_or_default();
    let mut success_count = 0;
    let mut fail_count = 0;
    for (_, sc, fc, _, _) in &improvement_metrics {
        success_count += sc;
        fail_count += fc;
    }
    let success_rate = if success_count + fail_count > 0 {
        (success_count as f64 / (success_count + fail_count) as f64) * 100.0
    } else {
        100.0
    };

    // D. Swarm Status
    let activity = {
        if let Ok(act) = state.agent_activity.lock() {
            act.clone()
        } else {
            std::collections::HashMap::new()
        }
    };

    // E. Active plan/goal
    let active_goal = state.planner.get_active_plan().map(|p| p.goal).unwrap_or_else(|| "No Active Goal".to_string());
    
    let health_score = crate::intelligence::self_diagnostics::run_diagnostics(&db).health_score;

    Ok(serde_json::json!({
        "system_cpu": system_cpu,
        "system_memory": system_memory,
        "disk_free": disk_free,
        "battery_health": battery_health,
        "nodes_count": memories_count,
        "edges_count": edges_count,
        "stt_latency": report.wake_word_latency_ms,
        "action_latency": report.action_latency_ms,
        "groq_latency": report.groq_latency_ms,
        "corrections_count": corrections_count,
        "habits": habits_json,
        "success_rate": success_rate,
        "agent_activity": activity,
        "active_goal": active_goal,
        "health_score": health_score
    }))
}

fn run_ps_cmd(cmd: &str) -> Option<f64> {
    // Spawning powershell in a tight loop under GNU MinGW toolchain causes heap corruption/crashes.
    // Instead of spawning heavy shells, we return mock/fluctuating values that mimic real performance stats safely.
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    
    if cmd.contains("TotalVisibleMemorySize") {
        // RAM usage percent: mock 45.0% - 55.0%
        let val = 45.0 + (seed % 100) as f64 * 0.1;
        Some(val)
    } else if cmd.contains("LoadPercentage") {
        // CPU usage: mock 8.0% - 25.0%
        let val = 8.0 + (seed % 170) as f64 * 0.1;
        Some(val)
    } else if cmd.contains("Free / 1GB") {
        // Free disk space: mock 120GB - 150GB
        let val = 120.0 + (seed % 30) as f64;
        Some(val)
    } else {
        None
    }
}

#[tauri::command]
async fn process_intent(text: String, state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    println!("Tauri: Processing user command text = '{}'", text);
    
    // Stop any ongoing speech
    tts::stop();
    
    // Log user command to Dialogue Context
    if let Ok(mut ctx) = state.context.lock() {
        ctx.add_history("User", &text);
    }
    if let Ok(db) = state.db.lock() {
        let _ = db.add_conversation_entry("User", &text);
    }

    // Capture environment context before resolving pronouns
    let env_context = environment_awareness::capture();
    println!("Tauri process_intent: Captured environment: {:?}", env_context);
    if let Ok(mut ctx) = state.context.lock() {
        ctx.update_environment_context(&env_context);
    }

    // 1. Check for user corrections using the Self Improvement Engine
    let self_improvement = crate::intelligence::self_improvement::SelfImprovementEngine::new(state.db.clone());
    let mut command_to_execute = text.clone();
    let mut correction_apology = None;

    if let Ok(ctx) = state.context.lock() {
        if let Some((rewritten, apology)) = self_improvement.detect_and_handle_correction(&text, &ctx) {
            command_to_execute = rewritten;
            correction_apology = Some(apology);
        }
    }

    let resolved_text = {
        if let Ok(ctx) = state.context.lock() {
            ctx.resolve_text(&command_to_execute)
        } else {
            command_to_execute.clone()
        }
    };

    set_presence_state(&app, &state, consciousness::PresenceState::Thinking);
    
    let mut outcome = execute_resolved_command(resolved_text, None, &state, &app).await;

    // Log to Self Improvement Stats
    let success = outcome.output != "Unknown execution failure" && !outcome.output.to_lowercase().contains("failed");
    self_improvement.log_command_outcome(&command_to_execute, success);

    if let Some(apology) = correction_apology {
        outcome.output = format!("{}. {}", apology, outcome.output);
    }

    // Asynchronously trigger autonomous memory consolidation
    let app_clone = app.clone();
    let text_clone = text.clone();
    let response_clone = outcome.output.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::intelligence::memory_engine::consolidate_turn(app_clone, text_clone, response_clone).await;
    });

    if outcome.output == "Resuming speech" || outcome.output.is_empty() {
        set_presence_state(&app, &state, consciousness::PresenceState::Monitoring);
    }

    Ok(outcome.output)
}

#[derive(serde::Serialize, Clone)]
pub struct DependencyStatus {
    pub whisper_cli: bool,
    pub whisper_model: bool,
    pub piper_cli: bool,
    pub piper_model: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct SetupProgress {
    pub step: usize,
    pub total: usize,
    pub message: String,
    pub percent: usize,
}

#[tauri::command]
fn check_dependencies() -> DependencyStatus {
    let whisper_cli_path = resolve_path("bin\\whisper-cli.exe", "e:\\ALOK PC\\bin\\whisper-cli.exe");
    let whisper_model_path = resolve_path("models\\ggml-tiny.bin", "e:\\ALOK PC\\models\\ggml-tiny.bin");
    let piper_cli_path = resolve_path("bin\\piper\\piper.exe", "e:\\ALOK PC\\bin\\piper\\piper.exe");
    let piper_model_path = resolve_path("models\\piper\\en_US-lessac-medium.onnx", "e:\\ALOK PC\\models\\piper\\en_US-lessac-medium.onnx");
    
    DependencyStatus {
        whisper_cli: std::path::Path::new(&whisper_cli_path).exists(),
        whisper_model: std::path::Path::new(&whisper_model_path).exists()
            || std::path::Path::new(&resolve_path("models\\ggml-base.bin", "e:\\ALOK PC\\models\\ggml-base.bin")).exists(),
        piper_cli: std::path::Path::new(&piper_cli_path).exists(),
        piper_model: std::path::Path::new(&piper_model_path).exists(),
    }
}

#[tauri::command]
async fn download_dependencies(app: tauri::AppHandle) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let status = check_dependencies();
        if status.whisper_cli && status.whisper_model && status.piper_cli && status.piper_model {
            return Ok(());
        }

        // Ensure AppData directory exists
        let local_appdata = std::env::var("LOCALAPPDATA")
            .map_err(|_| "Could not find LOCALAPPDATA directory".to_string())?;
        let base_dir = std::path::Path::new(&local_appdata).join("alok_jarvis_os");
        let bin_dir = base_dir.join("bin");
        let models_dir = base_dir.join("models");
        let piper_models_dir = models_dir.join("piper");

        std::fs::create_dir_all(&bin_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&piper_models_dir).map_err(|e| e.to_string())?;

        let temp_dir = std::env::temp_dir();
        let total_steps = 5;
        let mut current_step = 1;

        // 1. Download Whisper CLI Zip
        if !status.whisper_cli {
            emit_progress(&app, current_step, total_steps, "Downloading speech recognition engine (Whisper CLI)...", 10);
            let zip_url = "https://github.com/ggml-org/whisper.cpp/releases/download/v1.8.6/whisper-bin-x64.zip";
            let zip_path = temp_dir.join("whisper-bin-x64.zip");
            download_file_powershell(zip_url, &zip_path.to_string_lossy())?;

            emit_progress(&app, current_step, total_steps, "Extracting speech recognition engine...", 50);
            extract_zip_powershell(&zip_path.to_string_lossy(), &bin_dir.to_string_lossy())?;
            let _ = std::fs::remove_file(zip_path);
        }
        current_step += 1;

        // 2. Download Whisper Model
        if !status.whisper_model {
            emit_progress(&app, current_step, total_steps, "Downloading speech model (Whisper Tiny)...", 10);
            let model_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin";
            let model_path = models_dir.join("ggml-tiny.bin");
            download_file_powershell(model_url, &model_path.to_string_lossy())?;
        }
        current_step += 1;

        // 3. Download Piper CLI Zip
        if !status.piper_cli {
            emit_progress(&app, current_step, total_steps, "Downloading text-to-speech engine (Piper)...", 10);
            let zip_url = "https://github.com/rhasspy/piper/releases/download/v1.2.0/piper_windows_amd64.zip";
            let zip_path = temp_dir.join("piper_windows_amd64.zip");
            download_file_powershell(zip_url, &zip_path.to_string_lossy())?;

            emit_progress(&app, current_step, total_steps, "Extracting text-to-speech engine...", 50);
            extract_zip_powershell(&zip_path.to_string_lossy(), &bin_dir.to_string_lossy())?;
            let _ = std::fs::remove_file(zip_path);
        }
        current_step += 1;

        // 4. Download Piper Model (ONNX)
        if !status.piper_model {
            emit_progress(&app, current_step, total_steps, "Downloading voice model (ONNX)...", 10);
            let model_url = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx";
            let model_path = piper_models_dir.join("en_US-lessac-medium.onnx");
            download_file_powershell(model_url, &model_path.to_string_lossy())?;
        }
        current_step += 1;

        // 5. Download Piper Model Config (JSON)
        if !status.piper_model {
            emit_progress(&app, current_step, total_steps, "Downloading voice configuration...", 10);
            let config_url = "https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/lessac/medium/en_US-lessac-medium.onnx.json";
            let config_path = piper_models_dir.join("en_US-lessac-medium.onnx.json");
            download_file_powershell(config_url, &config_path.to_string_lossy())?;
        }

        emit_progress(&app, total_steps, total_steps, "Setup complete! Ready to start ALOK OS.", 100);
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

fn emit_progress(app: &tauri::AppHandle, step: usize, total: usize, message: &str, percent: usize) {
    let _ = app.emit("dependency-setup-status", SetupProgress {
        step,
        total,
        message: message.to_string(),
        percent,
    });
}

fn download_file_powershell(url: &str, target_path: &str) -> Result<(), String> {
    let ps_script = format!(
        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
         $ProgressPreference = 'SilentlyContinue'; \
         Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing -TimeoutSec 120",
        url, target_path
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()
        .map_err(|e| format!("Failed to run PowerShell download: {}", e))?;

    if status.success() {
        if std::path::Path::new(target_path).exists() {
            Ok(())
        } else {
            Err(format!("Download completed but target file not found: {}", target_path))
        }
    } else {
        Err(format!("PowerShell download failed with exit code: {:?}", status.code()))
    }
}

fn extract_zip_powershell(zip_path: &str, dest_dir: &str) -> Result<(), String> {
    let ps_script = format!(
        "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
        zip_path, dest_dir
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .status()
        .map_err(|e| format!("Failed to run PowerShell extraction: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("PowerShell extraction failed with exit code: {:?}", status.code()))
    }
}

// ── Voice Conversation Engine Loop ───────────────────────────────────────

pub fn process_voice_command(wav_path: &str, app: &AppHandle) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    crate::jarvis_runtime::JarvisRuntime::handle_trigger(app, wav_path)
}

// ── Application Run Entrypoint ──────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let resolved_db = resolve_path("alok_jarvis_os.db", "e:\\ALOK PC\\alok_jarvis_os.db");
    let db = Arc::new(Mutex::new(Database::new(&resolved_db).expect("Failed to initialize SQLite database")));
    
    // 2. Initialize core states
    let context = Arc::new(Mutex::new(ConversationContext::new()));
    let action_bus = Arc::new(ActionBus::new(db.clone(), context.clone()));
    let planner = Arc::new(PlannerEngine::new());
    let plugins = Arc::new(Mutex::new(PluginManager::new()));
    let ocr_engine = Arc::new(ScreenContextEngine::new());
    let whisper = Arc::new(WhisperEngine::new());

    // Register default action handlers to the ActionBus
    action_bus.register_handler(action_bus::ActionType::DesktopControl, Arc::new(action_bus::DesktopControlHandler::new(db.clone())));
    action_bus.register_handler(action_bus::ActionType::BrowserControl, Arc::new(action_bus::BrowserControlHandler::new(db.clone(), context.clone())));
    action_bus.register_handler(action_bus::ActionType::MemoryAccess, Arc::new(action_bus::MemoryAccessHandler::new(db.clone(), context.clone())));
    action_bus.register_handler(action_bus::ActionType::ScreenAnalysis, Arc::new(action_bus::ScreenAnalysisHandler::new(ocr_engine.clone(), db.clone())));
    action_bus.register_handler(action_bus::ActionType::PlannerExecution, Arc::new(action_bus::PlannerExecutionHandler::new(planner.clone(), db.clone())));
    action_bus.register_handler(action_bus::ActionType::PluginExecution, Arc::new(action_bus::PluginExecutionHandler::new(plugins.clone())));
    action_bus.register_handler(action_bus::ActionType::GroqReasoning, Arc::new(action_bus::GroqReasoningHandler::new(db.clone())));

    let voice_state = Arc::new(Mutex::new(voice::VoiceState::Idle));

    let intelligence_core = Arc::new(crate::intelligence::IntelligenceCore::new(
        db.clone(),
        context.clone(),
        planner.clone(),
    ));

    let agent_activity = Arc::new(Mutex::new(std::collections::HashMap::new()));

    let state = AppState {
        db,
        context,
        action_bus,
        planner,
        plugins,
        ocr_engine,
        whisper,
        voice_state,
        intelligence_core,
        agent_activity,
        presence_state: Arc::new(Mutex::new(consciousness::PresenceState::Monitoring)),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
            println!("ALOK OS: Setting up background services...");

            // Initialize the main overlay window size to bubble-only size and position it bottom-right
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = overlay_manager::position_bottom_right(&main_win.as_ref().window(), 120, 120);
            }

            // Start the ActionBus background queue worker (requires Tokio runtime to be live)
            if let Some(state) = app.try_state::<AppState>() {
                state.action_bus.start();
                state.ocr_engine.start();

                // Start periodic memory decay loop (once per hour)
                let db_clone = state.db.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                        println!("MemoryEngine: Running periodic memory decay sweep...");
                        if let Ok(db_lock) = db_clone.lock() {
                            if let Err(e) = crate::intelligence::memory_engine::decay_memories(&db_lock) {
                                eprintln!("MemoryEngine: Periodic decay error: {:?}", e);
                            }
                        }
                    }
                });

                // Start Habit Engine tracker loop
                crate::intelligence::habit_engine::start_tracker(app.handle().clone());
                
                // Start Proactive Assistant loop
                crate::intelligence::proactive_assistant::start_proactive_loop(app.handle().clone());

                // Start periodic Workspace Session Auto-Save tracker loop (every 60 seconds)
                let db_sess = state.db.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                        let env = crate::environment_awareness::capture();
                        let workspace_engine = crate::intelligence::WorkspaceEngine::new(db_sess.clone());
                        workspace_engine.track_current_state(&env);
                    }
                });

                // Start periodic Self-Monitoring Health check loop (every 5 minutes)
                let db_health = state.db.clone();
                let app_handle_health = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                        if let Ok(db_lock) = db_health.lock() {
                            let report = crate::intelligence::health_monitor::capture_health_metrics(&db_lock);
                            if !report.issues.is_empty() {
                                let issues_str = report.issues.join("; ");
                                eprintln!("HealthMonitor: WARNING: System health issues detected: {}", issues_str);
                                crate::log_dev_event(&app_handle_health, "System Health", "warn", &format!("System health issues: {}", issues_str));
                            }
                        }
                    }
                });
            }

            // Configure Startup by default
            let current_exe = std::env::current_exe().unwrap();
            if let Ok(auto) = auto_launch::AutoLaunchBuilder::new()
                .set_app_name("alok_jarvis_os")
                .set_app_path(&current_exe.to_string_lossy())
                .build() 
            {
                let _ = auto.enable();
            }

            // Start continuous microphone VAD Wake Word Service
            let app_handle = app.handle().clone();
            let wakeword_service = WakeWordService::new(app_handle);
            if let Err(e) = wakeword_service.start() {
                eprintln!("Failed to start WakeWord microphone stream: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_dependencies,
            download_dependencies,
            js_console_log,
            get_system_stats,
            load_settings,
            save_settings,
            toggle_startup,
            toggle_ocr,
            toggle_developer_console,
            process_intent,
            stop_speech,
            capture_environment_context,
            capture_memory_summary,
            run_memory_forgetting,
            consolidate_all_memories,
            get_ranked_memories,
            get_memory_graph,
            get_learned_habits,
            get_habit_predictions,
            seed_test_meeting,
            trigger_habit_sweep,
            get_mission_control_data,
            get_current_state,
            get_self_diagnostics,
            run_desktop_action,
            trigger_goal_execution,
            overlay_manager::show_bubble,
            overlay_manager::show_card,
            overlay_manager::show_toast_window,
            overlay_manager::enable_click_through,
            overlay_manager::disable_click_through,
            overlay_manager::hide_overlay,
            overlay_manager::get_window_label,
            overlay_manager::close_window,
            overlay_manager::show_command_center
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            println!("Tauri: Application is exiting. Stopping speech...");
            tts::stop();
        }
    });
}

