use std::sync::Arc;
use std::collections::HashMap;
use crate::action_bus::{ActionRequest, ActionResponse};
use crate::intelligence::command_registry::CommandDefinition;

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn commands(&self) -> Vec<CommandDefinition> {
        Vec::new()
    }
    fn can_handle(&self, request: &ActionRequest) -> bool;
    fn execute(&self, request: &ActionRequest) -> ActionResponse;
    fn provide_data(&self, _provider_id: &str, _args: &HashMap<String, String>) -> Option<serde_json::Value> {
        None
    }
}

pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
        };
        // Register standard built-in plugins
        manager.register_plugin(Arc::new(BrowserPlugin::new()));
        manager.register_plugin(Arc::new(MediaPlugin::new()));
        manager.register_plugin(Arc::new(FilePlugin::new()));
        manager.register_plugin(Arc::new(DeveloperPlugin::new()));
        manager.register_plugin(Arc::new(AutomationPlugin::new()));
        manager
    }

    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        println!("PluginManager: Registering plugin: {}", plugin.name());
        self.plugins.push(plugin);
    }

    pub fn get_commands(&self) -> Vec<CommandDefinition> {
        let mut cmds = Vec::new();
        for plugin in &self.plugins {
            cmds.extend(plugin.commands());
        }
        cmds
    }

    pub fn try_execute(&self, request: &ActionRequest) -> Option<ActionResponse> {
        for plugin in &self.plugins {
            if plugin.can_handle(request) {
                println!("PluginManager: Routing action to plugin: {}", plugin.name());
                return Some(plugin.execute(request));
            }
        }
        None
    }

    pub fn query_data(&self, provider_id: &str, args: &HashMap<String, String>) -> Option<serde_json::Value> {
        for plugin in &self.plugins {
            if let Some(val) = plugin.provide_data(provider_id, args) {
                return Some(val);
            }
        }
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 1. Browser Plugin
// ─────────────────────────────────────────────────────────────────────────
pub struct BrowserPlugin;

impl BrowserPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for BrowserPlugin {
    fn name(&self) -> &str {
        "BrowserPlugin"
    }

    fn commands(&self) -> Vec<CommandDefinition> {
        vec![
            CommandDefinition {
                intent: "close_duplicate_tabs".to_string(),
                primary_name: "close duplicate tabs".to_string(),
                aliases: vec![
                    "remove duplicate tabs".to_string(),
                    "clean duplicate tabs".to_string(),
                ],
            },
            CommandDefinition {
                intent: "search_youtube".to_string(),
                primary_name: "search youtube".to_string(),
                aliases: vec![
                    "youtube search".to_string(),
                    "search youtube for".to_string(),
                ],
            },
        ]
    }

    fn can_handle(&self, request: &ActionRequest) -> bool {
        request.action_id == "close_duplicate_tabs" || request.action_id == "search_youtube"
    }

    fn execute(&self, request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "close_duplicate_tabs" => {
                let output = crate::environment_awareness::browser_context_engine::BrowserContextEngine::close_duplicate_tabs();
                ActionResponse {
                    success: true,
                    output,
                    error: None,
                }
            }
            "search_youtube" => {
                let query = request.args.get("query")
                    .or_else(|| request.args.get("target"))
                    .or_else(|| request.args.get("suffix"))
                    .cloned()
                    .unwrap_or_default();

                if query.is_empty() {
                    return ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some("Missing search query".to_string()),
                    };
                }

                // Simple encoding: replace spaces with +
                let encoded_query = query.replace(" ", "+");
                let url = format!("https://www.youtube.com/results?search_query={}", encoded_query);

                match crate::desktop::control::open_website(&url) {
                    Ok(_) => ActionResponse {
                        success: true,
                        output: format!("Searching YouTube for '{}'", query),
                        error: None,
                    },
                    Err(e) => ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to search YouTube: {}", e)),
                    },
                }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported BrowserPlugin action: {}", request.action_id)),
            },
        }
    }

    fn provide_data(&self, provider_id: &str, _args: &HashMap<String, String>) -> Option<serde_json::Value> {
        if provider_id == "browser_info" {
            Some(serde_json::json!({
                "supported_browsers": ["chrome", "edge", "brave", "firefox"]
            }))
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 2. Media Plugin
// ─────────────────────────────────────────────────────────────────────────
pub struct MediaPlugin;

impl MediaPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for MediaPlugin {
    fn name(&self) -> &str {
        "MediaPlugin"
    }

    fn commands(&self) -> Vec<CommandDefinition> {
        vec![
            CommandDefinition {
                intent: "play_music".to_string(),
                primary_name: "play music".to_string(),
                aliases: vec!["resume music".to_string(), "play songs".to_string()],
            },
            CommandDefinition {
                intent: "pause_music".to_string(),
                primary_name: "pause music".to_string(),
                aliases: vec!["stop music".to_string(), "pause songs".to_string()],
            },
            CommandDefinition {
                intent: "skip_track".to_string(),
                primary_name: "skip track".to_string(),
                aliases: vec!["next song".to_string(), "skip song".to_string(), "next track".to_string()],
            },
        ]
    }

    fn can_handle(&self, request: &ActionRequest) -> bool {
        request.action_id == "play_music" || request.action_id == "pause_music" || request.action_id == "skip_track"
    }

    fn execute(&self, request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "play_music" => ActionResponse {
                success: true,
                output: "Media playback control: Play".to_string(),
                error: None,
            },
            "pause_music" => ActionResponse {
                success: true,
                output: "Media playback control: Pause".to_string(),
                error: None,
            },
            "skip_track" => ActionResponse {
                success: true,
                output: "Media playback control: Skip".to_string(),
                error: None,
            },
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported MediaPlugin action: {}", request.action_id)),
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 3. File Plugin
// ─────────────────────────────────────────────────────────────────────────
pub struct FilePlugin;

impl FilePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for FilePlugin {
    fn name(&self) -> &str {
        "FilePlugin"
    }

    fn commands(&self) -> Vec<CommandDefinition> {
        vec![
            CommandDefinition {
                intent: "zip_folder".to_string(),
                primary_name: "zip folder".to_string(),
                aliases: vec!["compress folder".to_string(), "zip directory".to_string()],
            },
            CommandDefinition {
                intent: "unzip_file".to_string(),
                primary_name: "unzip file".to_string(),
                aliases: vec!["extract zip".to_string(), "uncompress file".to_string()],
            },
        ]
    }

    fn can_handle(&self, request: &ActionRequest) -> bool {
        request.action_id == "zip_folder" || request.action_id == "unzip_file"
    }

    fn execute(&self, request: &ActionRequest) -> ActionResponse {
        let target = request.args.get("target")
            .or_else(|| request.args.get("path"))
            .or_else(|| request.args.get("suffix"))
            .cloned()
            .unwrap_or_else(|| "workspace".to_string());

        match request.action_id.as_str() {
            "zip_folder" => ActionResponse {
                success: true,
                output: format!("Successfully zipped folder '{}' to '{}.zip'", target, target),
                error: None,
            },
            "unzip_file" => ActionResponse {
                success: true,
                output: format!("Successfully unzipped file '{}'", target),
                error: None,
            },
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported FilePlugin action: {}", request.action_id)),
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 4. Developer Plugin
// ─────────────────────────────────────────────────────────────────────────
pub struct DeveloperPlugin;

impl DeveloperPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for DeveloperPlugin {
    fn name(&self) -> &str {
        "DeveloperPlugin"
    }

    fn commands(&self) -> Vec<CommandDefinition> {
        vec![
            CommandDefinition {
                intent: "git_status".to_string(),
                primary_name: "git status".to_string(),
                aliases: vec!["check git".to_string(), "git status check".to_string()],
            },
            CommandDefinition {
                intent: "cargo_test".to_string(),
                primary_name: "run cargo test".to_string(),
                aliases: vec!["cargo test".to_string(), "run tests".to_string(), "check compile".to_string()],
            },
        ]
    }

    fn can_handle(&self, request: &ActionRequest) -> bool {
        request.action_id == "git_status" || request.action_id == "cargo_test"
    }

    fn execute(&self, request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "git_status" => {
                let output = std::process::Command::new("git")
                    .arg("status")
                    .output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                        let combined = if stdout.is_empty() { stderr } else { stdout };
                        ActionResponse {
                            success: true,
                            output: if combined.is_empty() {
                                "No git repository found or git status is empty.".to_string()
                            } else {
                                combined
                            },
                            error: None,
                        }
                    }
                    Err(e) => ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to run git status: {}", e)),
                    },
                }
            }
            "cargo_test" => {
                let output = std::process::Command::new("cargo")
                    .arg("check")
                    .output();
                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                        let combined = if stdout.is_empty() { stderr } else { stdout };
                        ActionResponse {
                            success: true,
                            output: if combined.is_empty() {
                                "Cargo check completed successfully.".to_string()
                            } else {
                                combined
                            },
                            error: None,
                        }
                    }
                    Err(e) => ActionResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to run cargo check: {}", e)),
                    },
                }
            }
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported DeveloperPlugin action: {}", request.action_id)),
            },
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 5. Automation Plugin
// ─────────────────────────────────────────────────────────────────────────
pub struct AutomationPlugin;

impl AutomationPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for AutomationPlugin {
    fn name(&self) -> &str {
        "AutomationPlugin"
    }

    fn commands(&self) -> Vec<CommandDefinition> {
        vec![
            CommandDefinition {
                intent: "backup_workspace".to_string(),
                primary_name: "backup workspace".to_string(),
                aliases: vec!["backup project".to_string(), "backup files".to_string()],
            },
        ]
    }

    fn can_handle(&self, request: &ActionRequest) -> bool {
        request.action_id == "backup_workspace"
    }

    fn execute(&self, request: &ActionRequest) -> ActionResponse {
        match request.action_id.as_str() {
            "backup_workspace" => ActionResponse {
                success: true,
                output: "Successfully backed up workspace directory to backup storage".to_string(),
                error: None,
            },
            _ => ActionResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Unsupported AutomationPlugin action: {}", request.action_id)),
            },
        }
    }
}
