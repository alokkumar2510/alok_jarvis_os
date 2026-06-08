use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::AppState;
use alok_jarvis_os_lib::conversation_context::ConversationContext;
use alok_jarvis_os_lib::intelligence::execution_history;
use std::sync::{Arc, Mutex};
use std::fs;
use tauri::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Advanced Conversational Intelligence Engine Integration Test ===");

    // 1. Setup Test Database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_conv_test.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to initialize test SQLite database");
    let db_arc = Arc::new(Mutex::new(db));
    let context = Arc::new(Mutex::new(ConversationContext::new()));
    let action_bus = Arc::new(alok_jarvis_os_lib::action_bus::ActionBus::new(db_arc.clone(), context.clone()));
    let planner = Arc::new(alok_jarvis_os_lib::planner::PlannerEngine::new());
    let plugins = Arc::new(Mutex::new(alok_jarvis_os_lib::plugin::PluginManager::new()));
    let ocr_engine = Arc::new(alok_jarvis_os_lib::screen_context_engine::ScreenContextEngine::new());
    let whisper = Arc::new(alok_jarvis_os_lib::voice::WhisperEngine::new());
    let voice_state = Arc::new(Mutex::new(alok_jarvis_os_lib::voice::VoiceState::Idle));
    let intelligence_core = Arc::new(alok_jarvis_os_lib::intelligence::IntelligenceCore::new(
        db_arc.clone(),
        context.clone(),
        planner.clone(),
    ));

    // Register handlers to the ActionBus
    action_bus.register_handler(alok_jarvis_os_lib::action_bus::ActionType::DesktopControl, Arc::new(alok_jarvis_os_lib::action_bus::DesktopControlHandler::new(db_arc.clone())));
    action_bus.register_handler(alok_jarvis_os_lib::action_bus::ActionType::BrowserControl, Arc::new(alok_jarvis_os_lib::action_bus::BrowserControlHandler::new(db_arc.clone(), context.clone())));
    action_bus.register_handler(alok_jarvis_os_lib::action_bus::ActionType::MemoryAccess, Arc::new(alok_jarvis_os_lib::action_bus::MemoryAccessHandler::new(db_arc.clone(), context.clone())));

    let state = AppState {
        db: db_arc.clone(),
        context: context.clone(),
        action_bus,
        planner,
        plugins,
        ocr_engine,
        whisper,
        voice_state,
        intelligence_core,
        agent_activity: Arc::new(Mutex::new(std::collections::HashMap::new())),
        presence_state: Arc::new(Mutex::new(alok_jarvis_os_lib::consciousness::PresenceState::Monitoring)),
    };

    let app = tauri::Builder::default()
        .manage(state)
        .build(tauri::generate_context!())
        .expect("Failed to build tauri app context");

    // 2. Test Conversational Context Reference Resolution
    println!("\n[1] Testing Conversational Context Reference Resolution...");
    {
        let mut ctx = context.lock().unwrap();
        ctx.last_app = Some("chrome.exe".to_string());
        ctx.last_file = Some("task.md".to_string());
        ctx.last_url = Some("https://github.com".to_string());
        ctx.active_file = Some("lib.rs".to_string());
        ctx.previous_file = Some("mod.rs".to_string());
        ctx.active_selection = Some("code lines".to_string());

        // Resolve "it" -> last_app ("chrome.exe") or last_file ("task.md") or last_url ("https://github.com")
        // "it" replacement order is: last_app.clone().or_else(|| last_file).or_else(|| last_url)
        let res_it = ctx.resolve_text("summarize it");
        println!("  'summarize it' resolved to: '{}'", res_it);
        assert!(res_it.contains("chrome.exe"));

        // Resolve "that" -> last_url.clone().or_else(|| last_file).or_else(|| last_app)
        let res_that = ctx.resolve_text("open that");
        println!("  'open that' resolved to: '{}'", res_that);
        assert!(res_that.contains("https://github.com"));

        // Resolve "last file" -> active_file ("lib.rs") or last_file ("task.md")
        let res_last = ctx.resolve_text("view last file");
        println!("  'view last file' resolved to: '{}'", res_last);
        assert!(res_last.contains("lib.rs") || res_last.contains("task.md"));

        // Resolve "previous one" -> previous_file ("mod.rs")
        let res_prev = ctx.resolve_text("open previous one");
        println!("  'open previous one' resolved to: '{}'", res_prev);
        assert!(res_prev.contains("mod.rs"));

        // Resolve "them" -> active_selection ("code lines")
        let res_them = ctx.resolve_text("delete them");
        println!("  'delete them' resolved to: '{}'", res_them);
        assert!(res_them.contains("code lines"));
    }

    // 3. Test Action Verification & Retries via notepad launch
    println!("\n[2] Testing Action Verification via 'launch_app' for notepad...");
    {
        // Assert notepad is not currently running (or check its status)
        let is_running_before = alok_jarvis_os_lib::desktop::control::is_process_running("notepad.exe");
        println!("  Is notepad running before test? {}", is_running_before);

        println!("  Executing launch_app command for notepad...");
        let state_ref = app.state::<AppState>();
        let outcome = alok_jarvis_os_lib::execute_resolved_command("open notepad".to_string(), None, &state_ref, &app.handle()).await;
        println!("  Execution outcome: {}", outcome.output);

        let is_running_after = alok_jarvis_os_lib::desktop::control::is_process_running("notepad.exe");
        println!("  Is notepad running after test? {}", is_running_after);
        assert!(is_running_after, "Notepad should be verified as running");

        // Clean up: close notepad
        println!("  Closing notepad...");
        let _ = alok_jarvis_os_lib::desktop::control::close_app("notepad");
    }

    // 4. Test Task Execution Logging (duration and retries)
    println!("\n[3] Testing Task Execution Memory database records...");
    {
        let db_lock = db_arc.lock().unwrap();
        let history = execution_history::get_history(&db_lock, 5)?;
        assert!(!history.is_empty(), "Execution history should not be empty");
        
        let last_exec = &history[0];
        println!("  Last execution: task_name='{}', command='{}', success={}, duration_ms={}, retries={}",
            last_exec.task_name, last_exec.command, last_exec.success, last_exec.duration_ms, last_exec.retries);

        // Assert duration is greater than or equal to 0
        assert!(last_exec.duration_ms >= 0.0);
    }

    println!("\n=== Integration Test Passed Successfully ===");
    
    // Clean up test DB
    drop(db_arc);
    let _ = fs::remove_file(db_path);
    Ok(())
}
