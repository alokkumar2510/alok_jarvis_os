use crate::database::Database;
use crate::intelligence::GroqClient;
use tauri::AppHandle;
use crate::AppState;
use tauri::Manager;
use serde_json::Value;

pub async fn consolidate_turn(
    app: AppHandle,
    user_msg: String,
    assistant_msg: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = app.state::<AppState>();
    
    let (api_key, model) = {
        let db = state.db.lock().map_err(|_| "Failed to lock database")?;
        let api_key = db.get_setting("groq_api_key").unwrap_or_default();
        let model = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        (api_key, model)
    };

    let api_key = match api_key {
        Some(k) if !k.is_empty() => Some(k),
        _ => {
            // Local extraction fallback when API key is empty
            if let Some((content, tags, importance)) = extract_memory_locally(&user_msg) {
                let db = state.db.lock().map_err(|_| "Failed to lock database")?;
                db.store_memory_autonomous(&content, tags, importance)?;
                println!("MemoryEngine (Local Fallback): Stored memory: '{}'", content);
            }
            return Ok(());
        }
    };

    let groq = GroqClient::new(api_key, &model);

    let prompt = format!(
        "You are the Autonomous Memory Engine of ALOK JARVIS OS.\n\
         Analyze the following dialogue turn between the User and the Assistant:\n\n\
         User: \"{}\"\n\
         Assistant: \"{}\"\n\n\
         Determine if this interaction contains any of the following items about the user:\n\
         1. Preferences (e.g., likes/dislikes, preferred tools, settings)\n\
         2. Habits (e.g., routines, recurring schedules)\n\
         3. Frequent actions (e.g., launching specific apps/files regularly)\n\
         4. Important facts (e.g., names of relatives, work info, study fields)\n\
         5. Relationships (e.g., 'Neha is my colleague')\n\n\
         If there are new memories to store, output them in a JSON array. Each object MUST have:\n\
         - \"content\": string, a clear, standalone statement of the memory (e.g. \"User prefers dark mode\"). Do not refer to 'the user' as 'I', use 'User'.\n\
         - \"tags\": array of strings (choose from: \"preference\", \"habit\", \"action\", \"fact\", \"relationship\")\n\
         - \"importance\": number between 0.1 and 1.0 (e.g. user name or relation is 0.9, preferring dark mode is 0.6)\n\n\
         If there are no memories to store, output an empty array: []\n\
         Respond ONLY with the JSON array. Do not include any markdown fences (like ```json), explanations, or trailing text.",
        user_msg, assistant_msg
    );

    let prompt_clone = prompt.clone();
    let res = tokio::task::spawn_blocking(move || {
        groq.query(&prompt_clone)
    }).await?;

    let memories: Vec<Value> = match res {
        Ok(reply) => {
            let res_clean = reply.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
            if res_clean.is_empty() || res_clean == "[]" {
                vec![]
            } else {
                match serde_json::from_str(res_clean) {
                    Ok(m) => m,
                    Err(e) => {
                        println!("MemoryEngine: Failed to parse Groq response: {}. Error: {}. Falling back to local extractor.", res_clean, e);
                        if let Some((content, tags, importance)) = extract_memory_locally(&user_msg) {
                            vec![serde_json::json!({
                                "content": content,
                                "tags": tags,
                                "importance": importance
                            })]
                        } else {
                            vec![]
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("MemoryEngine: Groq query failed: {:?}. Falling back to local extractor.", e);
            if let Some((content, tags, importance)) = extract_memory_locally(&user_msg) {
                vec![serde_json::json!({
                    "content": content,
                    "tags": tags,
                    "importance": importance
                })]
            } else {
                vec![]
            }
        }
    };

    {
        let db = state.db.lock().map_err(|_| "Failed to lock database")?;
        for mem in memories {
            if let Some(content) = mem.get("content").and_then(|c| c.as_str()) {
                let tags: Vec<String> = mem.get("tags")
                    .and_then(|t| t.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let importance = mem.get("importance").and_then(|i| i.as_f64()).unwrap_or(0.5);

                if let Ok(existing) = db.search_memories(content) {
                    if existing.contains(&content.to_string()) {
                        continue;
                    }
                }

                db.store_memory_autonomous(content, tags, importance)?;
                println!("MemoryEngine: Automatically stored memory: '{}' (importance: {})", content, importance);
            }
        }
    }

    Ok(())
}

pub fn decay_memories(db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    db.decay_memories()?;
    Ok(())
}

pub async fn consolidate_memories(
    app: AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = app.state::<AppState>();

    let (list, api_key, model) = {
        let db = state.db.lock().map_err(|_| "Failed to lock database")?;
        let api_key = db.get_setting("groq_api_key").unwrap_or_default();
        let model = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        let list = db.get_all_memories_raw()?;
        (list, api_key, model)
    };

    if list.len() < 2 {
        return Ok(());
    }

    let api_key = match api_key {
        Some(k) if !k.is_empty() => Some(k),
        _ => return Ok(()),
    };

    let groq = GroqClient::new(api_key, &model);

    let mut list_str = String::new();
    for (id, content, importance) in &list {
        list_str.push_str(&format!("ID {}: \"{}\" (importance: {})\n", id, content, importance));
    }

    let prompt = format!(
        "You are the Memory Consolidation Engine of ALOK JARVIS OS.\n\
         Here is the list of memories stored about the user:\n\n\
         {}\n\n\
         Analyze the list. Look for redundant, overlapping, or contradicting memories.\n\
         Specify which memories should be merged or deleted.\n\
         - \"MERGE\": Combines multiple memory IDs into a single unified memory. Provide \"ids\" (array of IDs to merge), \"new_content\" (the consolidated statement), and \"importance\" (new importance score 0.1-1.0).\n\
         - \"DELETE\": Deletes a memory ID that is completely redundant or contradicted by a more recent/specific memory.\n\n\
         Output the actions strictly as a JSON array of objects. Example:\n\
         [\n\
           {{\"action\": \"MERGE\", \"ids\": [1, 3], \"new_content\": \"User prefers black coffee\", \"importance\": 0.6}},\n\
           {{\"action\": \"DELETE\", \"ids\": [2]}}\n\
         ]\n\n\
         If no consolidation is needed, output: []\n\
         Respond ONLY with the JSON array. Do not include markdown fences, explanations, or any other text.",
        list_str
    );

    let prompt_clone = prompt.clone();
    let res = tokio::task::spawn_blocking(move || {
        groq.query(&prompt_clone)
    }).await??;

    let res_clean = res.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim();
    if res_clean.is_empty() || res_clean == "[]" {
        return Ok(());
    }

    let actions: Vec<Value> = match serde_json::from_str(res_clean) {
        Ok(a) => a,
        Err(e) => {
            println!("MemoryEngine: Failed to parse consolidation actions: {}. Error: {}", res_clean, e);
            return Ok(());
        }
    };

    {
        let db = state.db.lock().map_err(|_| "Failed to lock database")?;
        for act in actions {
            if let Some(action_type) = act.get("action").and_then(|a| a.as_str()) {
                match action_type {
                    "MERGE" => {
                        let ids: Vec<i64> = act.get("ids")
                            .and_then(|i| i.as_array())
                            .map(|a| a.iter().filter_map(|v| v.as_i64()).collect())
                            .unwrap_or_default();
                        let new_content = act.get("new_content").and_then(|n| n.as_str()).unwrap_or("");
                        let importance = act.get("importance").and_then(|i| i.as_f64()).unwrap_or(0.5);

                        if !ids.is_empty() && !new_content.is_empty() {
                            for &id in &ids {
                                let _ = db.forget_memory(id);
                            }
                            db.store_memory_autonomous(new_content, vec!["preference".to_string(), "consolidated".to_string()], importance)?;
                            println!("MemoryEngine: Consolidated memory IDs {:?} into '{}'", ids, new_content);
                        }
                    }
                    "DELETE" => {
                        let ids: Vec<i64> = act.get("ids")
                            .and_then(|i| i.as_array())
                            .map(|a| a.iter().filter_map(|v| v.as_i64()).collect())
                            .unwrap_or_default();
                        for id in ids {
                            let _ = db.forget_memory(id);
                            println!("MemoryEngine: Deleted redundant memory ID {}", id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

pub async fn generate_weekly_summary(
    app: AppHandle,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let state = app.state::<AppState>();

    let (list, api_key, model) = {
        let db = state.db.lock().map_err(|_| "Failed to lock database")?;
        let api_key = db.get_setting("groq_api_key").unwrap_or_default();
        let model = db.get_setting("groq_model").unwrap_or_default().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        let list = db.get_all_memories_summary_raw()?;
        (list, api_key, model)
    };

    if list.is_empty() {
        return Ok("No memories recorded yet.".to_string());
    }

    let api_key = match api_key {
        Some(k) if !k.is_empty() => Some(k),
        _ => return Err("Groq is not configured".into()),
    };

    let groq = GroqClient::new(api_key, &model);

    let mut list_str = String::new();
    for (content, importance, decay, freq) in &list {
        list_str.push_str(&format!(
            "- {} (importance: {}, decay: {:.2}, recalled: {} times)\n",
            content, importance, decay, freq
        ));
    }

    let prompt = format!(
        "You are the Autonomous Memory Summary Engine of ALOK JARVIS OS.\n\
         Here is the list of active memories stored in the system:\n\n\
         {}\n\n\
         Generate a structured, high-fidelity Weekly Memory Summary of user preferences, habits, frequent actions, important facts, and relationships.\n\
         Use professional formatting with clear markdown headers (e.g., '# Weekly Memory Summary', '## Preferences & Habits', '## Important Facts & Relationships').\n\
         Do not include any conversational intro/outro text. Output only the markdown summary.",
         list_str
    );

    let prompt_clone = prompt.clone();
    let summary = tokio::task::spawn_blocking(move || {
        groq.query(&prompt_clone)
    }).await??;

    Ok(summary)
}

pub fn extract_memory_locally(user_msg: &str) -> Option<(String, Vec<String>, f64)> {
    let msg = user_msg.trim().trim_end_matches('?').trim_end_matches('.').to_lowercase();
    
    let clean_msg = if msg.starts_with("remember that ") {
        msg.chars().skip(14).collect::<String>()
    } else if msg.starts_with("remember: ") {
        msg.chars().skip(10).collect::<String>()
    } else if msg.starts_with("remember ") {
        msg.chars().skip(9).collect::<String>()
    } else {
        msg.clone()
    };
    
    let clean_msg = clean_msg.trim();
    if clean_msg.is_empty() {
        return None;
    }

    // Pattern 1: My name is X
    if clean_msg.starts_with("my name is ") {
        let name = clean_msg.replace("my name is ", "");
        let name = title_case(&name);
        return Some((format!("User name is {}", name), vec!["fact".to_string()], 0.9));
    }

    // Pattern 2: I prefer X / I like X
    if clean_msg.starts_with("i prefer ") {
        let pref = clean_msg.replace("i prefer ", "");
        return Some((format!("User prefers {}", pref), vec!["preference".to_string()], 0.6));
    }
    if clean_msg.starts_with("i like ") {
        let pref = clean_msg.replace("i like ", "");
        return Some((format!("User likes {}", pref), vec!["preference".to_string()], 0.6));
    }

    // Pattern 3: X is my manager / colleague / friend / wife / husband
    for relation in &["manager", "colleague", "coworker", "boss", "friend", "wife", "husband", "brother", "sister", "mother", "father"] {
        let is_my_pat = format!(" is my {}", relation);
        if clean_msg.contains(&is_my_pat) {
            let parts: Vec<&str> = clean_msg.split(&is_my_pat).collect();
            if !parts[0].is_empty() {
                let name = title_case(parts[0].trim());
                return Some((
                    format!("{} is User's {}", name, relation),
                    vec!["fact".to_string(), "relationship".to_string()],
                    0.8
                ));
            }
        }
        
        let my_pat_is = format!("my {} is ", relation);
        if clean_msg.starts_with(&my_pat_is) {
            let name = clean_msg.replace(&my_pat_is, "");
            let name = title_case(name.trim());
            return Some((
                format!("{} is User's {}", name, relation),
                vec!["fact".to_string(), "relationship".to_string()],
                0.8
            ));
        }
    }

    // Default fallback: if it explicitly starts with "remember", save it as a fact
    if msg.starts_with("remember ") || msg.starts_with("remember:") {
        let sentence = clean_msg.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default() + &clean_msg.chars().skip(1).collect::<String>();
        return Some((sentence, vec!["fact".to_string()], 0.5));
    }

    None
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

