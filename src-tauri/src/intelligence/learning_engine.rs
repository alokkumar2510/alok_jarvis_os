use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::database::Database;

pub struct LearningEngine {
    db: Arc<Mutex<Database>>,
}

impl LearningEngine {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Automatically scans user input for explicit alias definitions (e.g. "WA = WhatsApp" or "WA is WhatsApp")
    /// and records them in the learned_patterns database.
    pub fn learn_from_input(&self, text: &str) -> Option<String> {
        let text_trimmed = text.trim();
        let text_lower = text_trimmed.to_lowercase();

        // Check for explicit assignment templates: "X = Y" or "X is Y"
        let parts: Vec<&str> = if text_lower.contains(" = ") {
            text_trimmed.split(" = ").collect()
        } else if text_lower.contains(" is ") {
            text_trimmed.split(" is ").collect()
        } else {
            return None;
        };

        if parts.len() == 2 {
            let key = parts[0].trim();
            let val = parts[1].trim();

            // Check if key is a single short abbreviation (<= 5 characters, no spaces)
            if !key.is_empty() && key.len() <= 5 && !key.contains(' ') && !val.is_empty() {
                if let Ok(db_lock) = self.db.lock() {
                    let _ = db_lock.record_learned_pattern("alias", key, val);
                    println!("LearningEngine: Automatically learned alias shortcut: {} -> {}", key, val);
                    return Some(format!("I have learned that '{}' refers to '{}'.", key, val));
                }
            }
        }
        None
    }

    /// Replaces learned shortcuts (e.g. "WA") in user commands with fully-resolved names (e.g. "WhatsApp").
    pub fn pre_process_aliases(&self, text: &str) -> String {
        let aliases = if let Ok(db_lock) = self.db.lock() {
            db_lock.get_learned_patterns("alias").unwrap_or_default()
        } else {
            return text.to_string();
        };

        if aliases.is_empty() {
            return text.to_string();
        }

        let words: Vec<String> = text.split_whitespace().map(|word| {
            // Remove common punctuation to extract clean key
            let word_clean = word.to_lowercase()
                .replace(".", "")
                .replace(",", "")
                .replace("?", "")
                .replace("!", "")
                .replace("\"", "")
                .replace("'", "");
            
            for (key, val, _conf) in &aliases {
                if word_clean == key.to_lowercase() {
                    // Retain trailing punctuation if present in the original word
                    let mut punc = String::new();
                    if word.ends_with('.') { punc.push('.'); }
                    if word.ends_with(',') { punc.push(','); }
                    if word.ends_with('?') { punc.push('?'); }
                    if word.ends_with('!') { punc.push('!'); }
                    return format!("{}{}", val, punc);
                }
            }
            word.to_string()
        }).collect();

        words.join(" ")
    }

    /// Automatically records application and service usage frequency to predict preferred apps.
    pub fn record_command_usage(&self, intent: &str, args: &HashMap<String, String>) {
        if intent == "launch_app" {
            if let Some(app_name) = args.get("app_name") {
                let app_lower = app_name.to_lowercase();
                
                // Map app to generic categories if matched
                let category = if app_lower.contains("chrome") || app_lower.contains("edge") || app_lower.contains("brave") || app_lower.contains("firefox") {
                    Some("browser")
                } else if app_lower.contains("vscode") || app_lower.contains("code") || app_lower.contains("notepad") {
                    Some("editor")
                } else if app_lower.contains("terminal") || app_lower.contains("cmd") || app_lower.contains("powershell") {
                    Some("terminal")
                } else {
                    None
                };

                if let Some(cat) = category {
                    if let Ok(db_lock) = self.db.lock() {
                        let _ = db_lock.record_learned_pattern("preferred_app", cat, app_name);
                        println!("LearningEngine: Recorded preferred application usage: {} -> {}", cat, app_name);
                    }
                }
            }
        }
    }
}
