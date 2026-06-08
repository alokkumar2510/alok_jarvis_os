use alok_jarvis_os_lib::database::Database;
use alok_jarvis_os_lib::intelligence::pattern_detector::detect_patterns;
use alok_jarvis_os_lib::intelligence::behavior_model::predict_next_action;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Habit Learning System & Pattern Detector Integration Test ===");

    // 1. Initialize a clean test database
    let db_path = "e:\\ALOK PC\\alok_jarvis_os_test_habits.db";
    let _ = std::fs::remove_file(db_path);

    let db = Database::new(db_path).expect("Failed to create test database");

    // 2. Insert mock behavior logs across multiple days to simulate habits
    println!("\n[1] Seeding mock behavior logs...");

    // Seed app launching routines: VS Code at 9 AM (hour 09) on 3 different days
    db.log_behavior_with_timestamp("vscode", "main.rs", None, None, None, "2026-06-04 09:05:22")?;
    db.log_behavior_with_timestamp("vscode", "lib.rs", None, None, None, "2026-06-05 09:12:45")?;
    db.log_behavior_with_timestamp("vscode", "mod.rs", None, None, None, "2026-06-06 09:01:10")?;

    // Seed browser searching routines: google search for "DAA" at 6 PM (hour 18) on 3 different days
    db.log_behavior_with_timestamp("chrome", "Google Search", Some("https://www.google.com/search?q=DAA"), None, None, "2026-06-04 18:32:01")?;
    db.log_behavior_with_timestamp("chrome", "Google Search", Some("https://www.google.com/search?q=DAA&client=firefox"), None, None, "2026-06-05 18:44:12")?;
    db.log_behavior_with_timestamp("chrome", "Google Search", Some("https://www.google.com/search?q=DAA"), None, None, "2026-06-06 18:15:30")?;

    // Add some random noise entries to ensure confidence math works correctly (3 unique days total)
    db.log_behavior_with_timestamp("spotify", "Spotify", None, None, None, "2026-06-04 12:00:00")?;
    db.log_behavior_with_timestamp("notepad", "Notes", None, None, None, "2026-06-05 15:30:00")?;

    // Verify raw logs were successfully written
    let raw_logs = db.get_behavior_logs()?;
    println!("Stored behavior logs count: {}", raw_logs.len());
    assert_eq!(raw_logs.len(), 8);

    // 3. Trigger pattern sweep
    println!("\n[2] Running Pattern Detector sweep...");
    detect_patterns(&db)?;

    // 4. Retrieve learned habits
    println!("\n[3] Verifying learned habits from database...");
    let learned_habits = db.get_all_habits()?;
    for (habit_type, target, hour, confidence) in &learned_habits {
        println!("  Learned: type='{}', target='{}', hour={}, confidence={:.2}", habit_type, target, hour, confidence);
    }

    // Verify vscode habit at 9 AM is present with high confidence
    let vscode_habit = learned_habits.iter().find(|h| h.1 == "vscode" && h.2 == 9);
    assert!(vscode_habit.is_some(), "vscode habit at 9 AM was not detected!");
    assert!(vscode_habit.unwrap().3 >= 0.9, "vscode habit confidence should be high!");

    // Verify search:daa habit at 6 PM is present
    let daa_habit = learned_habits.iter().find(|h| h.1 == "search:daa" && h.2 == 18);
    assert!(daa_habit.is_some(), "search:daa habit at 18:00 was not detected!");
    assert!(daa_habit.unwrap().3 >= 0.9, "search:daa habit confidence should be high!");

    // 5. Test Behavior Model predictions
    println!("\n[4] Querying predictions from behavior model...");
    let predictions_9am = predict_next_action(&db, 9)?;
    println!("Predictions for 9 AM:");
    for pred in &predictions_9am {
        println!("  - Target: '{}' (type: '{}', confidence: {:.2})", pred.target, pred.habit_type, pred.confidence);
    }
    assert_eq!(predictions_9am.first().unwrap().target, "vscode");

    let predictions_6pm = predict_next_action(&db, 18)?;
    println!("Predictions for 6 PM:");
    for pred in &predictions_6pm {
        println!("  - Target: '{}' (type: '{}', confidence: {:.2})", pred.target, pred.habit_type, pred.confidence);
    }
    assert_eq!(predictions_6pm.first().unwrap().target, "search:daa");

    println!("\n=== Habit Learning Integration Test Completed Successfully ===");
    Ok(())
}
