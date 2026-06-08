use std::sync::{Arc, Mutex};
use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::AppState;
use alok_jarvis_os_lib::consciousness::get_current_state;
use alok_jarvis_os_lib::intelligence::self_diagnostics::run_diagnostics;

fn main() {
    println!("=== Testing Consciousness and Diagnostics Subsystems ===");

    // 1. Initialize DB and AppState in a test database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_test_diag.db";
    let db = Database::new(db_path).expect("Failed to initialize test DB");
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

    // 2. Test Self Diagnostics
    println!("Running Diagnostics Check...");
    {
        let db_lock = db_arc.lock().unwrap();
        let report = run_diagnostics(&db_lock);
        println!("Health Score: {}", report.health_score);
        println!("Voice: {}", report.voice_subsystem);
        println!("WakeWord: {}", report.wakeword_subsystem);
        println!("Memory: {}", report.memory_subsystem);
        println!("Automation: {}", report.automation_subsystem);
        println!("Plugin: {}", report.plugin_subsystem);
        println!("Failures: {:?}", report.failures);

        assert!(report.health_score > 0, "Health score should be non-zero");
    }

    // 3. Test Consciousness state aggregation
    println!("\nQuerying Consciousness State...");
    let current_state = get_current_state(&state);
    println!("Consciousness State JSON:\n{}", serde_json::to_string_pretty(&current_state).unwrap());

    println!("Consciousness & Diagnostics tests completed successfully.");
    
    // Clean up test database file
    let _ = std::fs::remove_file(db_path);
}
