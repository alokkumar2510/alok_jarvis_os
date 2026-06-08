use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::AppState;
use alok_jarvis_os_lib::intelligence::{
    project_detector, session_manager, workspace_engine::WorkspaceEngine,
    execution_history, solution_database, task_memory,
    developer_engine::DeveloperEngine, health_monitor, performance_tracker,
};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Workspace, Task Memory, Developer & Monitoring System Integration Test ===");

    // 1. Setup Test Database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_workspace_test.db";
    let _ = fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to initialize test SQLite database");
    
    // Seed settings
    let test_key = std::env::var("GROQ_API_KEY").unwrap_or_else(|_| "gsk_dummy_placeholder_for_tests".to_string());
    db.set_setting("groq_api_key", &test_key)?;
    db.set_setting("groq_model", "llama-3.1-8b-instant")?;
    db.set_setting("wakeword_sensitivity", "0.75")?;
    db.set_setting("tts_speed", "1.0")?;

    let db_arc = Arc::new(Mutex::new(db));
    let context = Arc::new(Mutex::new(alok_jarvis_os_lib::conversation_context::ConversationContext::new()));
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

    let state = AppState {
        db: db_arc.clone(),
        context,
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

    let _app = tauri::Builder::default()
        .manage(state)
        .build(tauri::generate_context!())
        .expect("Failed to build tauri app context");
    


    // 2. Test Project Detector
    println!("\n[1] Testing Project Detector...");
    // Let's create a temp cargo workspace structure to test
    let test_dir = "e:\\ALOK PC\\alok_jarvis_os\\src-tauri\\temp_test_workspace";
    fs::create_dir_all(test_dir)?;
    let cargo_toml = Path::new(test_dir).join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"temp_test\"")?;

    let detection = project_detector::detect_project(None, Some(test_dir));
    assert!(detection.is_some(), "Project detection should succeed");
    let (name, proj_type) = detection.unwrap();
    println!("  Detected Project: name={}, type={}", name, proj_type);
    assert_eq!(proj_type, "rust");

    // 3. Test Workspace Session Tracking & Restoration
    println!("\n[2] Testing Workspace Intelligence / Session Tracking...");
    let env_ctx = alok_jarvis_os_lib::environment_awareness::EnvironmentContext {
        active_window: alok_jarvis_os_lib::environment_awareness::ActiveWindowInfo {
            hwnd: 0,
            title: "VS Code".to_string(),
            process_name: "code.exe".to_string(),
        },
        browser: alok_jarvis_os_lib::environment_awareness::BrowserInfo {
            is_browser: true,
            browser_name: Some("chrome".to_string()),
            url: Some("https://github.com/rust-lang/rust".to_string()),
            title: Some("Rust GitHub".to_string()),
        },
        file: alok_jarvis_os_lib::environment_awareness::FileInfo {
            active_file: Some(cargo_toml.to_str().unwrap().to_string()),
            active_folder: Some(test_dir.to_string()),
        },
        clipboard: alok_jarvis_os_lib::environment_awareness::ClipboardInfo {
            clipboard_text: Some("let x = 42;".to_string()),
            selected_text: Some("x = 42".to_string()),
        },
        app: alok_jarvis_os_lib::environment_awareness::app_context::capture("code.exe"),
    };

    let ws_engine = WorkspaceEngine::new(db_arc.clone());
    ws_engine.track_current_state(&env_ctx);

    // Verify it was saved in DB
    {
        let db_lock = db_arc.lock().unwrap();
        let last_sess = session_manager::get_last_session(&db_lock)?;
        assert!(last_sess.is_some(), "Should find saved workspace session");
        let sess = last_sess.unwrap();
        println!("  Saved Session Name: {}", sess.session_name);
        println!("  Saved Project Path: {}", sess.project_path);
        println!("  Saved Browser Tabs: {}", sess.browser_tabs);
        assert!(sess.browser_tabs.contains("github.com/rust-lang/rust"));
    }

    // 4. Test Task Execution Memory & Solution Database
    println!("\n[3] Testing Task Execution Memory & Solution Database...");
    {
        let db_lock = db_arc.lock().unwrap();
        // Log a failing command run
        execution_history::log_command(
            &db_lock,
            "Rust Build",
            "cargo build --bin test_workspace_developer",
            "error[E0382]: use of moved value: `db`",
            false,
            Some("use of moved value"),
            120.0,
            0,
        )?;

        // Log a successful fix run later
        execution_history::log_command(
            &db_lock,
            "Rust Build Fix",
            "cargo build --bin test_workspace_developer",
            "Finished dev profile",
            true,
            None,
            85.0,
            1,
        )?;

        // Save a permanent known solution
        solution_database::save_solution(
            &db_lock,
            "use of moved value",
            "Use clone() or pass a borrow reference (&db) instead of moving the database owner.",
            "cargo build",
        )?;
    }

    // Test recalling build fix
    {
        let db_lock = db_arc.lock().unwrap();
        let fix_report = task_memory::recall_build_fix(&db_lock, "use of moved value")?;
        println!("  Recalled Fix:\n{}", fix_report);
        assert!(fix_report.contains("Use clone() or pass a borrow reference"));
    }

    // 5. Test Developer Assistant Mode
    println!("\n[4] Testing Developer Assistant Mode...");
    let dev_engine = DeveloperEngine::new(db_arc.clone());
    
    // Parse simulated log error
    let simulated_error = "error[E0382]: value moved here: `db` \n --> src/main.rs:10:15";
    let diagnostic = dev_engine.analyze_error(simulated_error);
    println!("  Offline Diagnostic:\n{}", diagnostic);
    assert!(diagnostic.contains("borrow checker error"));

    // Codebase search test
    let search_res = dev_engine.search_symbols(test_dir, "temp_test");
    println!("  Codebase Search:\n{}", search_res);
    assert!(search_res.contains("Cargo.toml"));

    // 6. Test Self Monitoring Telemetry
    println!("\n[5] Testing Self Monitoring Telemetry...");
    {
        let db_lock = db_arc.lock().unwrap();
        performance_tracker::log_latency(&db_lock, "speech_recognition", 240.0);
        performance_tracker::log_latency(&db_lock, "intent_resolution", 12.0);
        performance_tracker::log_latency(&db_lock, "action_execution", 95.0);
        performance_tracker::log_latency(&db_lock, "groq_api", 780.0);
        performance_tracker::log_latency(&db_lock, "wake_word", 280.0);

        let report = health_monitor::capture_health_metrics(&db_lock);
        println!("  Health Report:");
        println!("    RAM Usage: {:.2} MB", report.ram_usage_mb);
        println!("    CPU Load: {:.2}%", report.cpu_usage_pct);
        println!("    STT Latency Average: {:.2} ms", report.wake_word_latency_ms);
        println!("    Action Latency Average: {:.2} ms", report.action_latency_ms);
        println!("    Groq Latency Average: {:.2} ms", report.groq_latency_ms);
        println!("    Issues Found: {:?}", report.issues);
    }

    // Cleanup temp folders/files
    let _ = fs::remove_file(cargo_toml);
    let _ = fs::remove_dir(test_dir);
    let _ = fs::remove_file(db_path);

    println!("\n=== All Integration Tests Passed Successfully ===");
    Ok(())
}
