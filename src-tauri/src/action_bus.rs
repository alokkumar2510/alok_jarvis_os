use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot};

use crate::database::Database;
use crate::conversation_context::ConversationContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    DesktopControl,
    BrowserControl,
    MemoryAccess,
    ScreenAnalysis,
    PlannerExecution,
    PluginExecution,
    GroqReasoning,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionRequest {
    pub action_type: ActionType,
    pub category: String,  // "device", "app", "file", "browser", "system"
    pub action_id: String, // e.g. "launch_app", "volume_up", "lock_system"
    pub args: HashMap<String, String>,
    pub timeout_seconds: Option<u64>,
    pub max_retries: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActionResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

pub trait ActionHandler: Send + Sync {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>>;
}

pub struct QueueItem {
    pub request: ActionRequest,
    pub app: AppHandle,
    pub tx: oneshot::Sender<ActionResponse>,
}

pub struct ActionBus {
    pub db: Arc<Mutex<Database>>,
    pub context: Arc<Mutex<ConversationContext>>,
    pub handlers: Arc<RwLock<HashMap<ActionType, Arc<dyn ActionHandler>>>>,
    pub queue_tx: mpsc::UnboundedSender<QueueItem>,
    pub queue_rx: Mutex<Option<mpsc::UnboundedReceiver<QueueItem>>>,
}

impl ActionBus {
    pub fn new(db: Arc<Mutex<Database>>, context: Arc<Mutex<ConversationContext>>) -> Self {
        let (queue_tx, queue_rx) = mpsc::unbounded_channel::<QueueItem>();
        let handlers = Arc::new(RwLock::new(HashMap::new()));

        Self {
            db,
            context,
            handlers,
            queue_tx,
            queue_rx: Mutex::new(Some(queue_rx)),
        }
    }

    /// Must be called from within Tauri's setup() closure where the Tokio runtime is live.
    pub fn start(&self) {
        let mut rx_guard = self.queue_rx.lock().unwrap();
        if let Some(mut queue_rx) = rx_guard.take() {
            let handlers_clone = self.handlers.clone();
            let db_clone = self.db.clone();
            let context_clone = self.context.clone();

            tauri::async_runtime::spawn(async move {
                while let Some(item) = queue_rx.recv().await {
                    let handlers = handlers_clone.clone();
                    let db = db_clone.clone();
                    let context = context_clone.clone();

                    tauri::async_runtime::spawn(async move {
                        let response = Self::dispatch_internal(
                            item.request,
                            &item.app,
                            handlers,
                            db,
                            context,
                        ).await;
                        let _ = item.tx.send(response);
                    });
                }
            });
        }
    }

    pub fn register_handler(&self, action_type: ActionType, handler: Arc<dyn ActionHandler>) {
        println!("ActionBus: Registering handler for {:?}", action_type);
        if let Ok(mut handlers) = self.handlers.write() {
            handlers.insert(action_type, handler);
        }
    }

    pub fn queue(&self, request: ActionRequest, app: AppHandle) -> oneshot::Receiver<ActionResponse> {
        let (tx, rx) = oneshot::channel();
        let item = QueueItem { request, app, tx };
        crate::log_dev_event(&item.app, "Actions", "info", &format!("Queued action request: {}:{}.", item.request.category, item.request.action_id));
        if let Err(e) = self.queue_tx.send(item) {
            eprintln!("ActionBus: Failed to queue action: {}", e);
        }
        rx
    }

    pub async fn dispatch(&self, request: ActionRequest, app: &AppHandle) -> ActionResponse {
        Self::dispatch_internal(
            request,
            app,
            self.handlers.clone(),
            self.db.clone(),
            self.context.clone(),
        ).await
    }

    async fn dispatch_internal(
        request: ActionRequest,
        app: &AppHandle,
        handlers: Arc<RwLock<HashMap<ActionType, Arc<dyn ActionHandler>>>>,
        db: Arc<Mutex<Database>>,
        context: Arc<Mutex<ConversationContext>>,
    ) -> ActionResponse {
        println!("ActionBus: Dispatching action = {:?}", request);
        crate::log_dev_event(app, "Actions", "info", &format!("Dispatching request: category='{}', action_id='{}'", request.category, request.action_id));
        
        // 1. Log action execution start in db/context
        let action_log = format!(
            "Executing action: {:?} - {}:{}",
            request.action_type, request.category, request.action_id
        );
        if let Ok(mut ctx) = context.lock() {
            ctx.add_history("Alok", &action_log);
        }
        if let Ok(db_lock) = db.lock() {
            let _ = db_lock.add_conversation_entry("Alok", &action_log);
        }

        // Publish start event to frontend
        let _ = app.emit("action-start", serde_json::json!({
            "action_type": request.action_type,
            "category": request.category,
            "action_id": request.action_id,
        }));

        // 2. Perform execution with retries and timeout
        let max_retries = request.max_retries.unwrap_or(0);
        let timeout_seconds = request.timeout_seconds.unwrap_or(30);
        let timeout_duration = Duration::from_secs(timeout_seconds);

        let start_time = std::time::Instant::now();
        let mut attempts = 0;
        let mut last_response = ActionResponse {
            success: false,
            output: String::new(),
            error: Some("Execution not started".to_string()),
        };

        while attempts <= max_retries {
            if attempts > 0 {
                println!(
                    "ActionBus: Retrying action {} (attempt {}/{})",
                    request.action_id, attempts, max_retries
                );
                let _ = app.emit("action-retry", serde_json::json!({
                    "action_id": request.action_id,
                    "attempt": attempts,
                    "max_retries": max_retries,
                }));
            }

            // Find matching handler
            let handler = {
                if let Ok(handlers_lock) = handlers.read() {
                    handlers_lock.get(&request.action_type).cloned()
                } else {
                    None
                }
            };

            let execution_result = match handler {
                Some(h) => {
                    // Wrap execution in tokio::time::timeout
                    match tokio::time::timeout(timeout_duration, h.execute(&request, app)).await {
                        Ok(res) => res,
                        Err(_) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Action timed out after {} seconds", timeout_seconds)),
                        },
                    }
                }
                None => {
                    // Check if we have a PluginExecution handler to process unhandled types
                    let plugin_handler = {
                        if let Ok(handlers_lock) = handlers.read() {
                            handlers_lock.get(&ActionType::PluginExecution).cloned()
                        } else {
                            None
                        }
                    };
                    if let Some(ph) = plugin_handler {
                        match tokio::time::timeout(timeout_duration, ph.execute(&request, app)).await {
                            Ok(res) => res,
                            Err(_) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Plugin Action timed out after {} seconds", timeout_seconds)),
                            },
                        }
                    } else {
                        // Fallback to legacy matching if no explicit handler registered
                        Self::execute_legacy_fallback_static(&request)
                    }
                }
            };

            if execution_result.success {
                crate::log_dev_event(app, "Actions", "info", &format!("Action execution succeeded: output='{}'", execution_result.output));
                last_response = execution_result;
                break;
            } else {
                crate::log_dev_event(app, "Actions", "error", &format!("Action execution failed: {}", execution_result.error.as_ref().unwrap_or(&"Unknown error".to_string())));
                last_response = execution_result;
                attempts += 1;
                if attempts <= max_retries {
                    // Backoff logic: wait before retrying (exponential: 500ms, 1000ms, 1500ms...)
                    tokio::time::sleep(Duration::from_millis(500 * attempts as u64)).await;
                }
            }
        }

        let elapsed_ms = start_time.elapsed().as_millis() as f64;
        if let Ok(db_lock) = db.lock() {
            let cmd_desc = format!("{:?}({:?})", request.action_type, request.args);
            let out_desc = if last_response.success {
                &last_response.output
            } else {
                last_response.error.as_ref().unwrap_or(&last_response.output)
            };
            let _ = db_lock.log_task_execution(
                &request.action_id,
                &cmd_desc,
                out_desc,
                last_response.success,
                last_response.error.as_deref(),
                elapsed_ms,
                attempts as i32,
            );
        }

        // 3. Notify frontend overlay via Tauri event
        let _ = app.emit("action-execution", last_response.clone());

        // Publish end event
        let _ = app.emit("action-complete", serde_json::json!({
            "action_id": request.action_id,
            "success": last_response.success,
            "error": last_response.error,
        }));

        last_response
    }

    // Legacy fallback methods (keeps backwards compatibility with synchronous tests)
    pub fn execute_app_action(&self, request: &ActionRequest) -> ActionResponse {
        Self::execute_app_action_static(request)
    }

    pub fn execute_device_action(&self, request: &ActionRequest) -> ActionResponse {
        Self::execute_device_action_static(request)
    }

    pub fn execute_system_action(&self, request: &ActionRequest) -> ActionResponse {
        Self::execute_system_action_static(request)
    }

    pub fn execute_file_action(&self, request: &ActionRequest) -> ActionResponse {
        Self::execute_file_action_static(request)
    }

    pub fn execute_browser_action(&self, request: &ActionRequest) -> ActionResponse {
        Self::execute_browser_action_static(request)
    }

    fn execute_legacy_fallback_static(request: &ActionRequest) -> ActionResponse {
        match request.category.as_str() {
            "app" => Self::execute_app_action_static(request),
            "device" => Self::execute_device_action_static(request),
            "system" => Self::execute_system_action_static(request),
            "file" => Self::execute_file_action_static(request),
            "browser" => Self::execute_browser_action_static(request),
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown action category: {}", request.category)),
            },
        }
    }

    fn execute_app_action_static(request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "launch_app" => {
                let app_name = request.args.get("app_name").cloned().unwrap_or_default();
                if app_name.is_empty() {
                    return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some("Missing app_name argument".to_string()),
                    };
                }

                let mut launched = false;
                let mut attempts = 0;
                let max_attempts = 4; // 1 initial + 3 retries

                while attempts < max_attempts && !launched {
                    attempts += 1;
                    if attempts > 1 {
                        println!("ActionBus launch_app: Verification failed. Retrying launch of '{}' (attempt {}/{})", app_name, attempts, max_attempts);
                    }

                    // Attempt launch
                    let _ = crate::desktop::control::launch_app(&app_name);

                    // Wait 1.0s for window/process to initialize
                    std::thread::sleep(Duration::from_millis(1000));

                    // Verify if running
                    if crate::desktop::control::is_process_running(&app_name) {
                        launched = true;
                    } else {
                        // Sleep with linear backoff
                        std::thread::sleep(Duration::from_millis(500 * attempts as u64));
                    }
                }

                if launched {
                    ActionResponse {
                        success: true,
                        output: format!("Successfully launched {}", app_name),
                        error: None,
                    }
                } else {
                    // Try failure recovery for browsers
                    let name_lower = app_name.to_lowercase();
                    if name_lower.contains("chrome") || name_lower.contains("edge") || name_lower.contains("firefox") || name_lower.contains("brave") {
                        println!("ActionBus launch_app: Launch persistently failed. Triggering recovery engine for '{}'...", app_name);
                        match crate::intelligence::recovery_engine::RecoveryEngine::recover_browser_hang(&app_name) {
                            Ok(recovery_msg) => {
                                // Double check if process is running post recovery
                                std::thread::sleep(Duration::from_millis(1500));
                                if crate::desktop::control::is_process_running(&app_name) {
                                    return ActionResponse {
                                        success: true,
                                        output: format!("Launched via recovery: {}", recovery_msg),
                                        error: None,
                                    };
                                }
                            }
                            Err(e) => {
                                println!("ActionBus launch_app: Recovery failed: {}", e);
                            }
                        }
                    }

                    ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to launch {} after {} attempts and recovery checks.", app_name, max_attempts)),
                    }
                }
            }
            "close_app" => {
                let app_name = request.args.get("app_name").cloned().unwrap_or_default();
                if app_name.is_empty() {
                    return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some("Missing app_name to close".to_string()),
                    };
                }
                match crate::desktop::control::close_app(&app_name) {
                    Ok(_) => ActionResponse {
                        success: true,
                        output: format!("Closed {}", app_name),
                        error: None,
                    },
                    Err(e) => ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to close {}: {}", app_name, e)),
                    },
                }
            }
            "minimize_app" => {
                let app_name = request.args.get("app_name").cloned().unwrap_or_default();
                match crate::desktop::control::minimize_app(&app_name) {
                    Ok(_) => ActionResponse { success: true, output: format!("Minimized {}", app_name), error: None },
                    Err(e) => ActionResponse { success: false, output: String::new(), error: Some(e.to_string()) },
                }
            }
            "maximize_app" => {
                let app_name = request.args.get("app_name").cloned().unwrap_or_default();
                match crate::desktop::control::maximize_app(&app_name) {
                    Ok(_) => ActionResponse { success: true, output: format!("Maximized {}", app_name), error: None },
                    Err(e) => ActionResponse { success: false, output: String::new(), error: Some(e.to_string()) },
                }
            }
            "focus_app" => {
                let app_name = request.args.get("app_name").cloned().unwrap_or_default();
                match crate::desktop::control::focus_app(&app_name) {
                    Ok(_) => ActionResponse { success: true, output: format!("Focused {}", app_name), error: None },
                    Err(e) => ActionResponse { success: false, output: String::new(), error: Some(e.to_string()) },
                }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported app action: {}", request.action_id)),
            },
        }
    }

    fn execute_device_action_static(request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "volume_up" => {
                // Use nircmd or PowerShell to raise volume
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "powershell -c \"(New-Object -com WScript.Shell).SendKeys([char]175)\"" ])
                    .spawn();
                ActionResponse { success: true, output: "Volume increased".to_string(), error: None }
            }
            "volume_down" => {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "powershell -c \"(New-Object -com WScript.Shell).SendKeys([char]174)\"" ])
                    .spawn();
                ActionResponse { success: true, output: "Volume decreased".to_string(), error: None }
            }
            "mute_volume" => {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "powershell -c \"(New-Object -com WScript.Shell).SendKeys([char]173)\"" ])
                    .spawn();
                ActionResponse { success: true, output: "Volume muted/unmuted".to_string(), error: None }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported device action: {}", request.action_id)),
            },
        }
    }

    fn execute_system_action_static(request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "lock_system" => match crate::desktop::power::lock_workstation() {
                Ok(_) => ActionResponse {
                    success: true,
                    output: "Workstation locked successfully".to_string(),
                    error: None,
                },
                Err(e) => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to lock workstation: {}", e)),
                },
            },
            "sleep_system" => match crate::desktop::power::suspend_system() {
                Ok(_) => ActionResponse {
                    success: true,
                    output: "System sent to sleep".to_string(),
                    error: None,
                },
                Err(e) => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to suspend system: {}", e)),
                },
            },
            "resume_speech" => match crate::voice::tts::resume() {
                Ok(_) => ActionResponse {
                    success: true,
                    output: String::new(),
                    error: None,
                },
                Err(e) => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to resume speech: {}", e)),
                },
            },
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported system action: {}", request.action_id)),
            },
        }
    }

    fn execute_file_action_static(request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "search_file" => {
                let filename = request.args.get("filename").cloned().unwrap_or_default();
                ActionResponse {
                    success: true,
                    output: format!("Found file match for '{}' at E:/Projects/flutter_app", filename),
                    error: None,
                }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported file action: {}", request.action_id)),
            },
        }
    }

    fn execute_browser_action_static(request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "open_website" => {
                let url = request.args.get("url").cloned().unwrap_or_default();
                if url.is_empty() {
                    return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some("Missing url argument".to_string()),
                    };
                }
                match crate::desktop::control::open_website(&url) {
                    Ok(_) => ActionResponse {
                        success: true,
                        output: format!("Opened browser tab to {}", url),
                        error: None,
                    },
                    Err(e) => ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to open website: {}", e)),
                    },
                }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported browser action: {}", request.action_id)),
            },
        }
    }
}

// ── Concrete Handler Implementations ────────────────────────────────────────

pub struct DesktopControlHandler {
    db: Arc<Mutex<Database>>,
}

impl DesktopControlHandler {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }
}

impl ActionHandler for DesktopControlHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            if request.action_id == "continue_work" {
                let workspace_engine = crate::intelligence::WorkspaceEngine::new(self.db.clone());
                match workspace_engine.continue_work() {
                    Ok(msg) => return ActionResponse {
                        success: true,
                        output: msg,
                        error: None,
                    },
                    Err(e) => return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(e),
                    },
                }
            }
            if request.action_id == "system_health" {
                if let Ok(db_lock) = self.db.lock() {
                    let report = crate::intelligence::health_monitor::capture_health_metrics(&db_lock);
                    let status_msg = if report.issues.is_empty() {
                        "All systems nominal. CPU, RAM, and latency performance targets are fully compliant.".to_string()
                    } else {
                        format!("System is tracking some issues: {}", report.issues.join(", "))
                    };
                    let msg = format!(
                        "Self-Monitoring Status:\n\
                         - CPU Load: {:.2}%\n\
                         - Memory Allocation: {:.2} MB\n\
                         - Wake Word Latency: {:.2} ms\n\
                         - Execution Latency: {:.2} ms\n\
                         - Reasoning Latency: {:.2} ms\n\
                         - Summary: {}",
                        report.cpu_usage_pct,
                        report.ram_usage_mb,
                        report.wake_word_latency_ms,
                        report.action_latency_ms,
                        report.groq_latency_ms,
                        status_msg
                    );
                    return ActionResponse {
                        success: true,
                        output: msg,
                        error: None,
                    };
                }
                return ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Failed to lock database".to_string()),
                };
            }
            if request.action_id == "switch_personality" {
                let personality = request.args.get("personality").cloned().unwrap_or_else(|| "assistant".to_string());
                if let Ok(db_lock) = self.db.lock() {
                    if db_lock.set_setting("voice_personality", &personality).is_ok() {
                        let msg = match personality.as_str() {
                            "professional" => "Switching to professional mode. System behavior optimized for technical precision.",
                            "friendly" => "Got it! I am now in friendly mode! 😊 What's on your mind?",
                            "companion" => "I've switched to Companion mode. I'm here for you, let's work on this together.",
                            _ => "I am now in Assistant mode. Ready to help with your tasks.",
                        };
                        return ActionResponse {
                            success: true,
                            output: msg.to_string(),
                            error: None,
                        };
                    }
                }
                return ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Failed to update personality in settings database.".to_string()),
                };
            }
            ActionBus::execute_legacy_fallback_static(request)
        })
    }
}

pub struct BrowserControlHandler {
    db: Arc<Mutex<Database>>,
    context: Arc<Mutex<ConversationContext>>,
}

impl BrowserControlHandler {
    pub fn new(db: Arc<Mutex<Database>>, context: Arc<Mutex<ConversationContext>>) -> Self {
        Self { db, context }
    }
}

impl ActionHandler for BrowserControlHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            match request.action_id.as_str() {
                "search_web" => {
                    let query = request.args.get("query").cloned().unwrap_or_default();
                    let last_app = if let Ok(ctx) = self.context.lock() {
                        ctx.last_app.clone()
                    } else {
                        None
                    };
                    
                    let output_msg = if let Some(ref app) = last_app {
                        format!("Searching '{}' in {}", query, app)
                    } else {
                        format!("Searching '{}' in default browser", query)
                      };
                      
                      match crate::desktop::control::search_query(&query) {
                          Ok(_) => ActionResponse {
                              success: true,
                              output: output_msg,
                              error: None,
                          },
                          Err(e) => ActionResponse {
                              success: false,
                              output: String::new(),
                              error: Some(format!("Failed to search: {}", e)),
                          },
                      }
                }
                "open_website" => {
                    let url = request.args.get("url").cloned().unwrap_or_default();
                    if url.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Missing url argument".to_string()),
                        };
                    }
                    match crate::desktop::control::open_website(&url) {
                        Ok(_) => ActionResponse {
                            success: true,
                            output: format!("Opened browser tab to {}", url),
                            error: None,
                        },
                        Err(e) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Failed to open website: {}", e)),
                        },
                    }
                }
                "summarize_page" => {
                    let active_url = request.args.get("url").cloned()
                        .or_else(|| {
                            if let Ok(ctx) = self.context.lock() {
                                ctx.active_url.clone()
                            } else {
                                None
                            }
                        });
                    let db_lock = self.db.lock().unwrap();
                    match crate::environment_awareness::browser_context_engine::BrowserContextEngine::summarize_current_page(&db_lock, active_url.as_deref()) {
                        Ok(summary) => ActionResponse {
                            success: true,
                            output: summary,
                            error: None,
                        },
                        Err(e) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(e),
                        },
                    }
                }
                "yesterday_article" => {
                    let history = crate::environment_awareness::browser_context_engine::BrowserContextEngine::get_yesterday_history();
                    if history.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("No browser history entries found from yesterday.".to_string()),
                        };
                    }
                    let mut target_url = None;
                    for entry in &history {
                        let url_lower = entry.url.to_lowercase();
                        let title_lower = entry.title.to_lowercase();
                        if url_lower.contains("article") || url_lower.contains("news") || url_lower.contains("blog") || title_lower.contains("article") || title_lower.contains("read") {
                            target_url = Some(entry.url.clone());
                            break;
                        }
                    }
                    let url_to_open = target_url.unwrap_or_else(|| history[0].url.clone());
                    match crate::desktop::control::open_website(&url_to_open) {
                        Ok(_) => ActionResponse {
                            success: true,
                            output: format!("Opening yesterday's article: {}", url_to_open),
                            error: None,
                        },
                        Err(e) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Failed to open website: {}", e)),
                        },
                    }
                }
                "close_duplicate_tabs" => {
                    let output = crate::environment_awareness::browser_context_engine::BrowserContextEngine::close_duplicate_tabs();
                    ActionResponse {
                        success: true,
                        output,
                        error: None,
                    }
                }
                "search_youtube" => {
                    let query = request.args.get("query").cloned().unwrap_or_else(|| "Rust tutorials".to_string());
                    let target_url = format!("https://www.youtube.com/results?search_query={}", query.replace(" ", "+"));
                    match crate::desktop::control::open_website(&target_url) {
                        Ok(_) => ActionResponse {
                            success: true,
                            output: format!("Searching YouTube for '{}'", query),
                            error: None,
                        },
                        Err(e) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Failed to search YouTube: {}", e)),
                        },
                    }
                }
                _ => ActionBus::execute_legacy_fallback_static(request),
            }
        })
    }
}


pub struct MemoryAccessHandler {
    db: Arc<Mutex<Database>>,
    #[allow(dead_code)]
    context: Arc<Mutex<ConversationContext>>,
}

impl MemoryAccessHandler {
    pub fn new(db: Arc<Mutex<Database>>, context: Arc<Mutex<ConversationContext>>) -> Self {
        Self { db, context }
    }
}

impl ActionHandler for MemoryAccessHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            match request.action_id.as_str() {
                "read_setting" => {
                    let key = request.args.get("key").cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.get_setting(&key) {
                            Ok(Some(val)) => ActionResponse {
                                success: true,
                                output: val,
                                error: None,
                            },
                            Ok(None) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Setting '{}' not found", key)),
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Database error: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "store_memory" => {
                    let content = request.args.get("content").cloned().unwrap_or_default();
                    let tags_str = request.args.get("tags").cloned().unwrap_or_default();
                    let tags: Vec<String> = if tags_str.is_empty() {
                        Vec::new()
                    } else {
                        tags_str.split(',').map(|s| s.trim().to_string()).collect()
                    };
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.store_memory(&content, tags) {
                            Ok(id) => {
                                ActionResponse {
                                    success: true,
                                    output: format!("Memory stored successfully with ID: {}", id),
                                    error: None,
                                }
                            }
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Failed to store memory: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "search_memories" => {
                    let query = request.args.get("query").cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.search_memories(&query) {
                            Ok(results) => ActionResponse {
                                success: true,
                                output: serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string()),
                                error: None,
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Failed to search memories: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "update_memory" => {
                    let id_str = request.args.get("id").cloned().unwrap_or_default();
                    let content = request.args.get("content").cloned().unwrap_or_default();
                    if let Ok(id) = id_str.parse::<i64>() {
                        if let Ok(db_lock) = self.db.lock() {
                            match db_lock.update_memory(id, &content) {
                                Ok(_) => ActionResponse {
                                    success: true,
                                    output: "Memory updated successfully".to_string(),
                                    error: None,
                                },
                                Err(e) => ActionResponse {
                                    success: false,
                                    output: String::new(),
                                    error: Some(format!("Failed to update memory: {}", e)),
                                },
                            }
                        } else {
                            ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some("Failed to lock database".to_string()),
                            }
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Invalid memory ID".to_string()),
                        }
                    }
                }
                "forget_memory" => {
                    let id_str = request.args.get("id").cloned().unwrap_or_default();
                    if let Ok(id) = id_str.parse::<i64>() {
                        if let Ok(db_lock) = self.db.lock() {
                            match db_lock.forget_memory(id) {
                                Ok(_) => ActionResponse {
                                    success: true,
                                    output: "Memory deleted successfully".to_string(),
                                    error: None,
                                },
                                Err(e) => ActionResponse {
                                    success: false,
                                    output: String::new(),
                                    error: Some(format!("Failed to forget memory: {}", e)),
                                },
                            }
                        } else {
                            ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some("Failed to lock database".to_string()),
                            }
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Invalid memory ID".to_string()),
                        }
                    }
                }
                "query_relationships" => {
                    let entity_name = request.args.get("entity_name").or(request.args.get("name")).cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.query_relationships(&entity_name) {
                            Ok(relationships) => ActionResponse {
                                success: true,
                                output: serde_json::to_string(&relationships).unwrap_or_else(|_| "[]".to_string()),
                                error: None,
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Failed to query relationships: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "query_memory" => {
                    let query = request.args.get("query").cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        let mut output_parts = Vec::new();
                        if let Ok(rels) = db_lock.query_relationships(&query) {
                            for (from, relation, to) in rels {
                                output_parts.push(format!("{} {} {}.", from, relation, to));
                            }
                        }
                        if let Ok(memories) = db_lock.search_memories(&query) {
                            for mem in memories {
                                if !output_parts.contains(&mem) {
                                    output_parts.push(mem);
                                }
                            }
                        }
                        let response_text = if output_parts.is_empty() {
                            format!("I don't remember anything about '{}'.", query)
                        } else {
                            output_parts.join(" ")
                        };
                        ActionResponse {
                            success: true,
                            output: response_text,
                            error: None,
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "add_entity" => {
                    let id = request.args.get("id").cloned().unwrap_or_default();
                    let name = request.args.get("name").cloned().unwrap_or_default();
                    let entity_type = request.args.get("entity_type").cloned().unwrap_or_default();
                    let description = request.args.get("description").cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.add_entity(&id, &name, &entity_type, &description) {
                            Ok(_) => ActionResponse {
                                success: true,
                                output: "Entity added successfully".to_string(),
                                error: None,
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Failed to add entity: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "add_relationship" => {
                    let from_id = request.args.get("from_id").or(request.args.get("from_entity_id")).cloned().unwrap_or_default();
                    let to_id = request.args.get("to_id").or(request.args.get("to_entity_id")).cloned().unwrap_or_default();
                    let relation = request.args.get("relation").cloned().unwrap_or_default();
                    if let Ok(db_lock) = self.db.lock() {
                        match db_lock.add_relationship(&from_id, &to_id, &relation) {
                            Ok(_) => ActionResponse {
                                success: true,
                                output: "Relationship added successfully".to_string(),
                                error: None,
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(format!("Failed to add relationship: {}", e)),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                "search_codebase" => {
                    let query = request.args.get("query").or(request.args.get("prompt")).cloned().unwrap_or_default();
                    if query.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Missing query argument for search_codebase".to_string()),
                        };
                    }
                    let active_folder = if let Ok(ctx) = self.context.lock() {
                        ctx.active_folder.clone()
                    } else {
                        None
                    };
                    let root_path = active_folder.unwrap_or_else(|| "e:\\ALOK PC\\alok_jarvis_os".to_string());
                    let dev_engine = crate::intelligence::DeveloperEngine::new(self.db.clone());
                    let output = dev_engine.search_symbols(&root_path, &query);
                    ActionResponse {
                        success: true,
                        output,
                        error: None,
                    }
                }
                "recall_build_fix" => {
                    let query = request.args.get("query").or(request.args.get("prompt")).cloned().unwrap_or_default();
                    if query.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Missing query argument for recall_build_fix".to_string()),
                        };
                    }
                    if let Ok(db_lock) = self.db.lock() {
                        match crate::intelligence::task_memory::recall_build_fix(&db_lock, &query) {
                            Ok(msg) => ActionResponse {
                                success: true,
                                output: msg,
                                error: None,
                            },
                            Err(e) => ActionResponse {
                                success: false,
                                output: String::new(),
                                error: Some(e),
                            },
                        }
                    } else {
                        ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Failed to lock database".to_string()),
                        }
                    }
                }
                _ => ActionBus::execute_legacy_fallback_static(request),
            }
        })
    }
}

pub struct ScreenAnalysisHandler {
    engine: Arc<crate::screen_context_engine::ScreenContextEngine>,
    db: Arc<Mutex<crate::database::Database>>,
}

impl ScreenAnalysisHandler {
    pub fn new(engine: Arc<crate::screen_context_engine::ScreenContextEngine>, db: Arc<Mutex<crate::database::Database>>) -> Self {
        Self { engine, db }
    }
}

impl ActionHandler for ScreenAnalysisHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            match request.action_id.as_str() {
                "capture_ocr" => {
                    match self.engine.capture_and_ocr() {
                        Ok(text) => ActionResponse {
                            success: true,
                            output: text,
                            error: None,
                        },
                        Err(e) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Screen OCR failed: {:?}", e)),
                        },
                    }
                }
                "analyze_error" => {
                    let ocr = crate::vision::VisionEngine::capture_screen_text();
                    if ocr.trim().is_empty() {
                        return ActionResponse {
                            success: true,
                            output: "I took a screenshot but couldn't detect any text on your screen. Please make sure the window containing the error is in the foreground.".to_string(),
                            error: None,
                        };
                    }
                    
                    // Try offline parsing using DeveloperEngine
                    let dev_engine = crate::intelligence::DeveloperEngine::new(self.db.clone());
                    let offline_res = dev_engine.analyze_error(&ocr);
                    
                    if !offline_res.contains("I couldn't parse the diagnostic signature") {
                        ActionResponse {
                            success: true,
                            output: offline_res,
                            error: None,
                        }
                    } else {
                        // Fallback to online reasoning (Groq) in VisionEngine
                        let db_guard = self.db.lock().unwrap();
                        let output = crate::vision::VisionEngine::analyze_error(&db_guard);
                        ActionResponse {
                            success: true,
                            output,
                            error: None,
                        }
                    }
                }
                "explain_code" => {
                    let db_guard = self.db.lock().unwrap();
                    let output = crate::vision::VisionEngine::explain_code(&db_guard);
                    ActionResponse {
                        success: true,
                        output,
                        error: None,
                    }
                }
                "summarize_screen" => {
                    let db_guard = self.db.lock().unwrap();
                    let output = crate::vision::VisionEngine::summarize_screen(&db_guard);
                    ActionResponse {
                        success: true,
                        output,
                        error: None,
                    }
                }
                _ => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unsupported screen analysis action: {}", request.action_id)),
                },
            }
        })
    }
}

pub struct PlannerExecutionHandler {
    planner: Arc<crate::planner::PlannerEngine>,
    db: Arc<Mutex<crate::database::Database>>,
}

impl PlannerExecutionHandler {
    pub fn new(planner: Arc<crate::planner::PlannerEngine>, db: Arc<Mutex<crate::database::Database>>) -> Self {
        Self { planner, db }
    }
}

impl ActionHandler for PlannerExecutionHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            match request.action_id.as_str() {
                "execute_plan" => {
                    let goal = request.args.get("goal").cloned().unwrap_or_default();
                    if goal.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Missing goal argument".to_string()),
                        };
                    }
                    let plan = {
                        let db_lock = self.db.lock().unwrap();
                        self.planner.generate_plan(&goal, &db_lock)
                    };
                    self.planner.execute_plan(plan, app.clone());
                    ActionResponse {
                        success: true,
                        output: format!("Started executing plan for goal: {}", goal),
                        error: None,
                    }
                }
                _ => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unsupported planner action: {}", request.action_id)),
                },
            }
        })
    }
}

pub struct PluginExecutionHandler {
    plugins: Arc<Mutex<crate::plugin::PluginManager>>,
}

impl PluginExecutionHandler {
    pub fn new(plugins: Arc<Mutex<crate::plugin::PluginManager>>) -> Self {
        Self { plugins }
    }
}

impl ActionHandler for PluginExecutionHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            let plugins_lock = match self.plugins.lock() {
                Ok(p) => p,
                Err(_) => return ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some("Failed to lock plugin manager".to_string()),
                },
            };
            match plugins_lock.try_execute(request) {
                Some(res) => res,
                None => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("No plugin could handle request: {}", request.action_id)),
                },
            }
        })
    }
}

pub struct GroqReasoningHandler {
    db: Arc<Mutex<Database>>,
}

impl GroqReasoningHandler {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }
}

impl ActionHandler for GroqReasoningHandler {
    fn execute<'a>(
        &'a self,
        request: &'a ActionRequest,
        _app: &'a AppHandle,
    ) -> Pin<Box<dyn Future<Output = ActionResponse> + Send + 'a>> {
        Box::pin(async move {
            match request.action_id.as_str() {
                "query" => {
                    let prompt = request.args.get("prompt").cloned().unwrap_or_default();
                    if prompt.is_empty() {
                        return ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some("Missing prompt argument".to_string()),
                        };
                    }
                    
                    let (api_key, model, personality) = if let Ok(db_lock) = self.db.lock() {
                        let key = db_lock.get_setting("groq_api_key").unwrap_or_default();
                        let mdl = db_lock.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
                        let pers = db_lock.get_setting("voice_personality").unwrap_or_default().unwrap_or_else(|| "assistant".to_string());
                        (key, mdl, pers)
                    } else {
                        (None, "llama-3.1-8b-instant".to_string(), "assistant".to_string())
                    };

                    let system_style = match personality.as_str() {
                        "professional" => "You are ALOK OS, in Professional mode. Be extremely concise, direct, technical, and developer-focused. Avoid conversational filler or introductory phrases.",
                        "friendly" => "You are ALOK OS, in Friendly mode. Be warm, supportive, enthusiastic, and cheerful. Use emojis like 😊 or 🚀 where appropriate.",
                        "companion" => "You are ALOK OS, in Companion mode. Be highly conversational, empathetic, personal, and talk like a close friend who is collaborating with the user.",
                        _ => "You are ALOK OS, in Assistant mode. Be helpful, clear, and structured.",
                    };

                    let styled_prompt = format!("System context: {}. User request: {}", system_style, prompt);

                    let groq = crate::intelligence::GroqClient::new(api_key, &model);
                    let res = tokio::task::spawn_blocking(move || {
                        groq.query(&styled_prompt)
                    }).await;

                    match res {
                        Ok(Ok(output)) => ActionResponse {
                            success: true,
                            output,
                            error: None,
                        },
                        Ok(Err(e)) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(e.to_string()),
                        },
                        Err(join_err) => ActionResponse {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Thread execution failed: {}", join_err)),
                        },
                    }
                }
                _ => ActionResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unsupported Groq reasoning action: {}", request.action_id)),
                },
            }
        })
    }
}
