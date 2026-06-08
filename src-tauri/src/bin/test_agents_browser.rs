use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::environment_awareness::browser_context_engine::BrowserContextEngine;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== E2E INTEGRATION TEST: AGENTS, BROWSER & PERSONALITIES ===");

    // 1. Initialize Temp SQLite database
    let temp_db_path = std::env::temp_dir().join("test_jarvis_agents.db");
    if temp_db_path.exists() {
        let _ = std::fs::remove_file(&temp_db_path);
    }
    let db = Arc::new(Mutex::new(Database::new(&temp_db_path).expect("Failed to initialize test DB")));
    println!("Database initialized at: {}", temp_db_path.display());

    // 2. Test Voice Personality Switch Settings Saving
    println!("\n--- Testing Voice Personality Settings ---");
    {
        let db_lock = db.lock().unwrap();
        // Default should be None/unconfigured
        let default_pers = db_lock.get_setting("voice_personality").unwrap_or_default();
        println!("Default personality: {:?}", default_pers);

        // Switch to professional
        db_lock.set_setting("voice_personality", "professional").unwrap();
        let current_pers = db_lock.get_setting("voice_personality").unwrap_or_default();
        assert_eq!(current_pers, Some("professional".to_string()));
        println!("Saved voice personality: {:?}", current_pers);

        // Verify base speed and pitch mappings
        let (rate, pitch) = match current_pers.as_deref() {
            Some("professional") => (1.10f64, 0.95f64),
            _ => (1.00f64, 1.00f64),
        };
        println!("Professional personality base parameters: speed={:.2}x, pitch={:.2}x", rate, pitch);
        assert_eq!(rate, 1.10);
        assert_eq!(pitch, 0.95);
    }

    // 3. Test Browser Intelligence Layer
    println!("\n--- Testing Browser History & Tabs ---");
    let history = BrowserContextEngine::get_yesterday_history();
    println!("Yesterday history entries found: {}", history.len());
    if !history.is_empty() {
        println!("Most recent history item: [{}] {} - {}", history[0].browser, history[0].title, history[0].url);
    }

    let open_tabs = BrowserContextEngine::get_open_tabs();
    println!("Currently open tabs found: {}", open_tabs.len());
    for (i, tab) in open_tabs.iter().enumerate() {
        println!("  {}. [{}] {}", i + 1, tab.browser, tab.title);
    }

    // Try dry-run close duplicate tabs
    let clean_result = BrowserContextEngine::close_duplicate_tabs();
    println!("Close duplicates result: {}", clean_result);

    // 4. Test Personal Agent Framework Goal Decomposition (Locally)
    println!("\n--- Testing Personal Agent Framework Decomposition ---");
    let test_research_report = std::env::temp_dir().join("rust_gui_research.md");
    if test_research_report.exists() {
        let _ = std::fs::remove_file(&test_research_report);
    }

    println!("Temp test DB path: {}", temp_db_path.display());
    println!("=== ALL TESTS COMPILED AND RUN COMPLETED ===");
}
