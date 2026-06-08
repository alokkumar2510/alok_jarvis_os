use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DialogueTurn {
    pub sender: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationContext {
    pub turns: Vec<DialogueTurn>,
    pub variables: HashMap<String, String>, // Dialog variables like "it", "that", "current_dir"
    pub active_window_title: String,
    pub turn_count: u32,

    // Store properties
    pub last_intent: Option<String>,
    pub last_app: Option<String>,
    pub last_file: Option<String>,
    pub last_url: Option<String>,
    pub last_action: Option<String>,

    // Active environment context fields
    pub active_url: Option<String>,
    pub active_folder: Option<String>,
    pub active_file: Option<String>,
    pub active_selection: Option<String>,
    pub active_clipboard: Option<String>,
    pub previous_file: Option<String>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            turns: Vec::new(),
            variables: HashMap::new(),
            active_window_title: String::new(),
            turn_count: 0,
            last_intent: None,
            last_app: None,
            last_file: None,
            last_url: None,
            last_action: None,
            active_url: None,
            active_folder: None,
            active_file: None,
            active_selection: None,
            active_clipboard: None,
            previous_file: None,
        }
    }

    pub fn add_history(&mut self, sender: &str, message: &str) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.turns.push(DialogueTurn {
            sender: sender.to_string(),
            message: message.to_string(),
            timestamp: time,
        });

        // Cap history length to 50 turns
        if self.turns.len() > 50 {
            self.turns.remove(0);
        }

        self.turn_count += 1;
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn update_context(
        &mut self,
        intent: Option<String>,
        app: Option<String>,
        file: Option<String>,
        url: Option<String>,
        action: Option<String>,
    ) {
        if intent.is_some() {
            self.last_intent = intent;
        }
        if app.is_some() {
            self.last_app = app;
        }
        if file.is_some() {
            self.last_file = file;
        }
        if url.is_some() {
            self.last_url = url;
        }
        if action.is_some() {
            self.last_action = action;
        }
    }

    pub fn update_environment_context(&mut self, env: &crate::environment_awareness::EnvironmentContext) {
        let proc_lower = env.active_window.process_name.to_lowercase();
        if proc_lower.contains("alok_jarvis_os") || proc_lower.contains("alok-jarvis-os") {
            return;
        }

        // Track previous file transition
        if let Some(ref current_file) = env.file.active_file {
            if self.active_file.as_ref() != Some(current_file) {
                self.previous_file = self.active_file.clone();
            }
            self.active_file = Some(current_file.clone());
            self.last_file = Some(current_file.clone());
        }

        self.active_url = env.browser.url.clone();
        if env.browser.url.is_some() {
            self.last_url = env.browser.url.clone();
        }

        self.active_folder = env.file.active_folder.clone();
        self.active_selection = env.clipboard.selected_text.clone();
        self.active_clipboard = env.clipboard.clipboard_text.clone();
        self.active_window_title = env.active_window.title.clone();
        self.last_app = Some(env.active_window.process_name.clone());
    }

    pub fn resolve_text(&self, text: &str) -> String {
        let mut resolved = text.to_string();

        let replace_phrase = |source: &str, phrase: &str, replacement: &str| -> String {
            let mut result = source.to_string();
            let mut start_idx = 0;
            let phrase_lower = phrase.to_lowercase();
            while let Some(pos) = result.to_lowercase()[start_idx..].find(&phrase_lower) {
                let actual_pos = start_idx + pos;
                
                let before_char_ok = actual_pos == 0 || {
                    let prev_char = result.chars().nth(actual_pos - 1).unwrap_or(' ');
                    !prev_char.is_alphanumeric()
                };
                let after_char_ok = actual_pos + phrase.len() == result.len() || {
                    let next_char = result.chars().nth(actual_pos + phrase.len()).unwrap_or(' ');
                    !next_char.is_alphanumeric()
                };

                if before_char_ok && after_char_ok {
                    result.replace_range(actual_pos..actual_pos + phrase.len(), replacement);
                    start_idx = actual_pos + replacement.len();
                } else {
                    start_idx = actual_pos + 1;
                }
            }
            result
        };

        if let Some(ref url) = self.active_url {
            resolved = replace_phrase(&resolved, "this page", url);
            resolved = replace_phrase(&resolved, "this website", url);
            resolved = replace_phrase(&resolved, "this link", url);
        }

        if let Some(ref folder) = self.active_folder {
            resolved = replace_phrase(&resolved, "this folder", folder);
            resolved = replace_phrase(&resolved, "this directory", folder);
            resolved = replace_phrase(&resolved, "this path", folder);
        }

        if let Some(ref file) = self.active_file {
            resolved = replace_phrase(&resolved, "this file", file);
        }

        if let Some(ref prev) = self.previous_file {
            resolved = replace_phrase(&resolved, "previous file", prev);
        }

        // Resolve "this" (standalone)
        let this_replacement = if let Some(ref selection) = self.active_selection {
            if !selection.trim().is_empty() {
                Some(format!("the selected text: '{}'", selection))
            } else {
                None
            }
        } else {
            None
        };
        let this_replacement = this_replacement
            .or_else(|| self.active_url.as_ref().map(|url| format!("the page URL '{}'", url)))
            .or_else(|| self.active_file.as_ref().map(|file| format!("the file '{}'", file)));

        if let Some(ref replacement) = this_replacement {
            resolved = replace_phrase(&resolved, "this", replacement);
        }

        // Resolve "it"
        let it_replacement = self.last_app.clone()
            .or_else(|| self.last_file.clone())
            .or_else(|| self.last_url.clone());
        if let Some(ref replacement) = it_replacement {
            resolved = replace_phrase(&resolved, "it", replacement);
        }

        // Resolve "that"
        let that_replacement = self.last_url.clone()
            .or_else(|| self.last_file.clone())
            .or_else(|| self.last_app.clone());
        if let Some(ref replacement) = that_replacement {
            resolved = replace_phrase(&resolved, "that", replacement);
        }

        // Resolve "second tab"
        if resolved.to_lowercase().contains("second tab") {
            let tabs = crate::environment_awareness::browser_context_engine::BrowserContextEngine::get_open_tabs();
            if tabs.len() >= 2 {
                resolved = replace_phrase(&resolved, "second tab", &tabs[1].title);
            }
        }

        // Resolve "first result"
        if resolved.to_lowercase().contains("first result") {
            let mut resolved_to = None;
            if let Some(var) = self.get_variable("first_result") {
                resolved_to = Some(var.clone());
            } else {
                let ocr = crate::vision::VisionEngine::capture_screen_text();
                let lines: Vec<&str> = ocr.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
                if !lines.is_empty() {
                    resolved_to = Some(lines[0].to_string());
                }
            }
            if let Some(r_val) = resolved_to {
                resolved = replace_phrase(&resolved, "first result", &r_val);
            }
        }

        // Resolve "last file"
        if resolved.to_lowercase().contains("last file") {
            if let Some(ref file) = self.last_file.as_ref().or(self.active_file.as_ref()) {
                resolved = replace_phrase(&resolved, "last file", file);
            }
        }

        // Resolve "previous one"
        if resolved.to_lowercase().contains("previous one") {
            if let Some(ref prev) = self.previous_file.as_ref()
                .or(self.last_url.as_ref())
                .or(self.last_app.as_ref())
                .or(self.last_action.as_ref()) 
            {
                resolved = replace_phrase(&resolved, "previous one", prev);
            }
        }

        // Resolve "them"
        if resolved.to_lowercase().contains("them") {
            let mut resolved_to = None;
            if let Some(ref sel) = self.active_selection {
                if !sel.trim().is_empty() {
                    resolved_to = Some(sel.clone());
                }
            }
            if resolved_to.is_none() {
                if let Some(var) = self.get_variable("them") {
                    resolved_to = Some(var.clone());
                }
            }
            if let Some(r_val) = resolved_to {
                resolved = replace_phrase(&resolved, "them", &r_val);
            }
        }

        resolved
    }

    pub fn resolve_args(&self, args: &mut HashMap<String, String>) {
        for value in args.values_mut() {
            let val_lower = value.to_lowercase();
            if val_lower == "it" || val_lower == "that" {
                if let Some(ref app) = self.last_app {
                    *value = app.clone();
                } else if let Some(ref file) = self.last_file {
                    *value = file.clone();
                } else if let Some(ref url) = self.last_url {
                    *value = url.clone();
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.turns.clear();
        self.variables.clear();
        self.turn_count = 0;
        self.last_intent = None;
        self.last_app = None;
        self.last_file = None;
        self.last_url = None;
        self.last_action = None;
        self.active_url = None;
        self.active_folder = None;
        self.active_file = None;
        self.active_selection = None;
        self.active_clipboard = None;
        self.previous_file = None;
    }
}



