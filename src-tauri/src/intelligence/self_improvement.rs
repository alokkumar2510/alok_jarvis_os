use std::sync::{Arc, Mutex};
use crate::database::Database;
use crate::conversation_context::ConversationContext;

pub struct SelfImprovementEngine {
    db: Arc<Mutex<Database>>,
}

impl SelfImprovementEngine {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Checks if the user prompt is a correction of a previous action.
    /// If so, logs the correction, adjusts confidence/habits in the database, and rewrites the command.
    pub fn detect_and_handle_correction(&self, user_input: &str, context: &ConversationContext) -> Option<(String, String)> {
        let input_trimmed = user_input.trim();
        let input_lower = input_trimmed.to_lowercase();

        // Correction trigger prefixes
        let triggers = [
            "no, i meant ",
            "no, open ",
            "no, use ",
            "actually use ",
            "actually open ",
            "that was wrong, ",
            "no, i said ",
        ];

        let mut matched_trigger = None;
        for t in &triggers {
            if input_lower.starts_with(t) {
                matched_trigger = Some(*t);
                break;
            }
        }

        if let Some(trigger) = matched_trigger {
            let correction_payload = &input_trimmed[trigger.len()..];
            println!("SelfImprovementEngine: Detected correction input: '{}'", correction_payload);

            // Fetch last action/intent details from context
            if let Some(ref last_intent) = context.last_intent {
                let db_lock = self.db.lock().ok()?;
                
                // 1. Log correction in database
                let incorrect_desc = format!(
                    "Intent: '{}', App: '{:?}', File: '{:?}', URL: '{:?}'",
                    last_intent,
                    context.last_app,
                    context.last_file,
                    context.last_url
                );
                let _ = db_lock.add_correction(&incorrect_desc, correction_payload, Some(last_intent));

                // 2. Adjust learned app preferences if intent was launch_app or app control
                if last_intent == "launch_app" {
                    // Learn the new preferred app
                    let category = if correction_payload.to_lowercase().contains("chrome")
                        || correction_payload.to_lowercase().contains("edge")
                        || correction_payload.to_lowercase().contains("brave")
                        || correction_payload.to_lowercase().contains("firefox")
                    {
                        "browser"
                    } else if correction_payload.to_lowercase().contains("code")
                        || correction_payload.to_lowercase().contains("vscode")
                        || correction_payload.to_lowercase().contains("notepad")
                    {
                        "editor"
                    } else {
                        "generic"
                    };

                    let _ = db_lock.record_learned_pattern("preferred_app", category, correction_payload);
                }

                // 3. Rewrite command based on correction payload
                let rewritten_cmd = if last_intent == "launch_app" || last_intent == "open_app" {
                    format!("open {}", correction_payload)
                } else if last_intent == "open_website" || last_intent == "open_url" {
                    format!("open website {}", correction_payload)
                } else if last_intent == "search_youtube" || last_intent == "search_web" {
                    format!("search for {}", correction_payload)
                } else {
                    correction_payload.to_string()
                };

                return Some((rewritten_cmd, format!("Apologies, let me do that with '{}' instead.", correction_payload)));
            }
        }
        None
    }

    /// Logs the outcome (success or failure) of a command execution.
    pub fn log_command_outcome(&self, command: &str, success: bool) {
        if let Ok(db_lock) = self.db.lock() {
            let _ = db_lock.log_self_improvement_outcome(command, success);
        }
    }
}
