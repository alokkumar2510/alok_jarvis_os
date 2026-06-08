use crate::database::Database;
use std::collections::{HashMap, HashSet};

pub fn detect_patterns(db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let logs = db.get_behavior_logs()?;
    if logs.is_empty() {
        return Ok(());
    }

    // 1. Determine unique days in logs
    let mut unique_days = HashSet::new();
    for entry in &logs {
        let ts = &entry.5;
        if ts.len() >= 10 {
            unique_days.insert(ts[0..10].to_string());
        }
    }
    let total_days = unique_days.len() as f64;
    if total_days == 0.0 {
        return Ok(());
    }

    // 2. Count occurrences of (hour, target)
    let mut counts: HashMap<(String, String, i32), HashSet<String>> = HashMap::new();

    for entry in &logs {
        let process = &entry.0;
        let url_opt = &entry.2;
        let ts = &entry.5;

        if ts.len() < 13 {
            continue;
        }
        let date = ts[0..10].to_string();
        let hour_str = &ts[11..13];
        let hour = match hour_str.parse::<i32>() {
            Ok(h) => h,
            Err(_) => continue,
        };

        // Track App launch habits
        if process != "unknown" && process != "unknown.exe" && process != "alok_jarvis_os.exe" && !process.is_empty() {
            let key = ("app_launch".to_string(), process.clone(), hour);
            counts.entry(key).or_insert_with(HashSet::new).insert(date.clone());
        }

        // Track Browser behavior
        if let Some(url) = url_opt {
            if !url.is_empty() && url != "unknown" {
                let target = if url.contains("search?q=") {
                    if let Some(pos) = url.find("q=") {
                        let query = &url[pos+2..];
                        let query_clean = query.split('&').next().unwrap_or(query);
                        let query_decoded = query_clean.replace("%20", " ").replace("+", " ");
                        format!("search:{}", query_decoded.to_lowercase())
                    } else {
                        url.clone()
                    }
                } else {
                    url.clone()
                };

                let key = ("browser_visit".to_string(), target, hour);
                counts.entry(key).or_insert_with(HashSet::new).insert(date.clone());
            }
        }
    }

    // 3. Evaluate confidence and update database habits table
    db.clear_user_habits()?;

    for ((habit_type, target, hour), dates) in counts {
        let days_occurred = dates.len() as f64;
        let confidence = days_occurred / total_days;

        if confidence >= 0.4 {
            db.add_user_habit(&habit_type, &target, hour, confidence)?;
            println!("PatternDetector: Detected habit '{}' [{}] at hour {} (confidence: {:.2})", target, habit_type, hour, confidence);
        }
    }

    Ok(())
}
