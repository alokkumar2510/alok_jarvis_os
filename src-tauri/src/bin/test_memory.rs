use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::AppState;
use alok_jarvis_os_lib::intelligence::memory_engine::{
    consolidate_turn, consolidate_memories, generate_weekly_summary, decay_memories
};
use std::sync::{Arc, Mutex};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Autonomous Memory Engine Integration Test ===");

    // 1. Initialize DB and AppState in a test database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_test.db";
    // Delete existing test db if any
    let _ = std::fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to create test database");
    
    // Seed Groq API Key and Model for LLM queries to succeed
    let test_key = std::env::var("GROQ_API_KEY").unwrap_or_else(|_| "gsk_dummy_placeholder_for_tests".to_string());
    db.set_setting("groq_api_key", &test_key)?;
    db.set_setting("groq_model", "llama-3.1-8b-instant")?;

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

    // 2. Build mock/real tauri app instance
    let app = tauri::Builder::default()
        .manage(state)
        .build(tauri::generate_context!())
        .expect("Failed to build tauri app context");
    
    let handle = app.handle().clone();

    // 3. Test consolidated turns
    println!("\n[1] Testing Automatic Memory Consolidation...");
    
    let turns = vec![
        ("I love writing Rust code.", "That is great! Rust is a very safe and fast language."),
        ("Neha is my manager at work.", "Understood. I will remember that Neha is your manager."),
        ("I prefer drinking black coffee in the morning.", "Got it. Black coffee in the morning it is.")
    ];

    for (user, assistant) in turns {
        println!("Consolidating turn: User: '{}'", user);
        consolidate_turn(handle.clone(), user.to_string(), assistant.to_string()).await?;
    }

    // Inspect stored memories
    println!("\n--- Stored Memories after turns ---");
    {
        let db_lock = db_arc.lock().unwrap();
        let memories = db_lock.get_all_memories_summary_raw()?;
        for (content, importance, decay, freq) in &memories {
            println!("- {} (importance: {}, decay: {}, freq: {})", content, importance, decay, freq);
        }
    }

    // Inspect Graph nodes/edges
    println!("\n--- Memory Graph Nodes after turns ---");
    {
        let db_lock = db_arc.lock().unwrap();
        let nodes = db_lock.get_nodes()?;
        for node in &nodes {
            println!("  Node: id={}, label={}, type={}", node.id, node.label, node.node_type);
        }
        let edges = db_lock.get_edges()?;
        for edge in &edges {
            println!("  Edge: {} --[{}]--> {}", edge.from_id, edge.relation, edge.to_id);
        }
    }

    // 4. Test Memory Consolidation / De-duplication
    println!("\n[2] Testing memory de-duplication/merging...");
    // Let's add a redundant memory
    {
        let db_lock = db_arc.lock().unwrap();
        db_lock.store_memory_autonomous("User prefers strong black coffee.", vec!["preference".to_string()], 0.65)?;
    }
    
    println!("Consolidating all memories...");
    consolidate_memories(handle.clone()).await?;

    println!("\n--- Stored Memories after de-duplication ---");
    {
        let db_lock = db_arc.lock().unwrap();
        let memories = db_lock.get_all_memories_summary_raw()?;
        for (content, importance, decay, freq) in &memories {
            println!("- {} (importance: {}, decay: {}, freq: {})", content, importance, decay, freq);
        }
    }

    // 5. Test Weekly Memory Summary
    println!("\n[3] Generating Weekly Memory Summary...");
    let summary = generate_weekly_summary(handle.clone()).await?;
    println!("Weekly Memory Summary:\n{}", summary);

    // 6. Test Memory Ranking
    println!("\n[4] Testing Memory Ranking...");
    {
        let db_lock = db_arc.lock().unwrap();
        // Query empty string gets ranked memories
        let ranked = db_lock.search_memories("")?;
        for (i, r) in ranked.iter().enumerate() {
            println!("  Rank {}: {}", i + 1, r);
        }
    }

    // 7. Test Memory Forgetting / Decay
    println!("\n[5] Testing Memory Forgetting & Decay...");
    // Update last_recalled_at for Neha memory to be 1000 hours in the past
    // And store a very low importance memory and set its recalled at to 200 hours in the past
    {
        let db_lock = db_arc.lock().unwrap();
        let low_id = db_lock.store_memory_autonomous("User ate pizza for lunch.", vec!["food".to_string()], 0.1)?;
        
        db_lock.update_last_recalled_at_for_test(low_id, 200.0)?;
        
        println!("Running first decay sweep (low importance memory)...");
        decay_memories(&db_lock)?;
        
        let memories = db_lock.get_all_memories_summary_raw()?;
        let pizza_exists = memories.iter().any(|(content, _, _, _)| content.contains("pizza"));
        println!("  Pizza memory exists after 200 hours? {} (should have decayed, but not forgotten yet)", pizza_exists);

        // Put it 2000 hours in past
        db_lock.update_last_recalled_at_for_test(low_id, 2000.0)?;
        
        println!("Running second decay sweep (forgotten threshold)...");
        decay_memories(&db_lock)?;
        
        let memories2 = db_lock.get_all_memories_summary_raw()?;
        let pizza_exists2 = memories2.iter().any(|(content, _, _, _)| content.contains("pizza"));
        println!("  Pizza memory exists after 2000 hours? {} (should be forgotten/deleted)", pizza_exists2);
    }

    println!("\n=== Integration Test Completed Successfully ===");
    Ok(())
}
