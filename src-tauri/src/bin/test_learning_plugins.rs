use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::AppState;
use alok_jarvis_os_lib::intelligence::learning_engine::LearningEngine;
use alok_jarvis_os_lib::execute_resolved_command;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Continuous Learning Engine & Plugins Integration Test ===");

    // 1. Initialize DB and AppState in a test database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_test_learning.db";
    let _ = std::fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to create test database");
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

    // Register handlers to action bus
    action_bus.register_handler(alok_jarvis_os_lib::action_bus::ActionType::PluginExecution, Arc::new(alok_jarvis_os_lib::action_bus::PluginExecutionHandler::new(plugins.clone())));

    let state = AppState {
        db: db_arc.clone(),
        context,
        action_bus,
        planner,
        plugins: plugins.clone(),
        ocr_engine,
        whisper,
        voice_state,
        intelligence_core,
        agent_activity: Arc::new(Mutex::new(HashMap::new())),
        presence_state: Arc::new(Mutex::new(alok_jarvis_os_lib::consciousness::PresenceState::Monitoring)),
    };

    let app = tauri::Builder::default()
        .manage(state)
        .build(tauri::generate_context!())
        .expect("Failed to build tauri app context");
    
    let handle = app.handle();

    let learning_engine = LearningEngine::new(db_arc.clone());

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: Explicit Alias Learning and Resolution
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[1] Testing Explicit Alias Learning...");
    
    // Define an alias explicitly
    let learn_res = learning_engine.learn_from_input("WA = WhatsApp");
    assert!(learn_res.is_some(), "Should learn alias from 'WA = WhatsApp'");
    println!("Response: {}", learn_res.unwrap());

    // Verify alias is recorded in DB
    {
        let db_lock = db_arc.lock().unwrap();
        let aliases = db_lock.get_learned_patterns("alias")?;
        assert!(aliases.iter().any(|(k, v, _)| k == "wa" && v == "whatsapp"), "Alias 'wa' should be stored in DB");
        println!("Alias correctly stored in DB.");
    }

    // Verify alias pre-processing
    let processed = learning_engine.pre_process_aliases("open WA to chat");
    println!("Processed text: '{}'", processed);
    assert_eq!(processed, "open whatsapp to chat", "Alias replacement failed");

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: Preferred App Learning
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[2] Testing Preferred App Learning...");

    let mut args = HashMap::new();
    args.insert("app_name".to_string(), "Chrome".to_string());
    
    // Simulate multiple launches to build frequency and confidence
    for _ in 0..3 {
        learning_engine.record_command_usage("launch_app", &args);
    }

    // Verify database preferred app resolution
    {
        let db_lock = db_arc.lock().unwrap();
        let preferred = db_lock.get_preferred_app("browser");
        assert_eq!(preferred, Some("chrome".to_string()), "Preferred browser should resolve to chrome");
        println!("Preferred browser correctly resolved to chrome.");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: Plugin Commands Integration & Execution
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[3] Testing Plugin Commands and Dispatch...");

    // Execute plugin command "git status"
    println!("Executing: 'git status' via execute_resolved_command...");
    let outcome1 = execute_resolved_command("git status".to_string(), None, handle.state::<AppState>().inner(), &handle).await;
    println!("Git Status outcome success: {}, output length: {}", outcome1.intent_score > 0.6, outcome1.output.len());
    assert!(outcome1.output.contains("On branch") || outcome1.output.contains("No git repository") || outcome1.output.contains("not a git repository"), "Output did not seem to be Git status output");

    // Execute plugin command "search youtube rust programming"
    println!("Executing: 'search youtube rust programming' via execute_resolved_command...");
    let outcome2 = execute_resolved_command("search youtube rust programming".to_string(), None, handle.state::<AppState>().inner(), &handle).await;
    println!("YouTube Search output: '{}'", outcome2.output);
    assert!(outcome2.output.contains("Searching YouTube for"), "YouTube plugin action failed");

    // Execute plugin command "zip folder workspace"
    println!("Executing: 'zip folder project_files' via execute_resolved_command...");
    let outcome3 = execute_resolved_command("zip folder project_files".to_string(), None, handle.state::<AppState>().inner(), &handle).await;
    println!("File zip output: '{}'", outcome3.output);
    assert!(outcome3.output.contains("Successfully zipped folder 'project_files'"), "File plugin zip action failed");

    // Clean up test database file
    let _ = std::fs::remove_file(db_path);

    println!("\n=== Integration Test Completed Successfully ===");
    Ok(())
}
