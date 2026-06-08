use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CommandDefinition {
    pub intent: String,
    pub primary_name: String,
    pub aliases: Vec<String>,
}

pub struct CommandRegistry {
    commands: Vec<CommandDefinition>,
    user_vocabulary: HashMap<String, String>, // term -> class (e.g. "chrome" -> "app")
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut commands = Vec::new();

        // 1. launch_app — primary name is the app word only (matched with "open X" context)
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open chrome".to_string(),
            aliases: vec!["launch chrome".to_string(), "start chrome".to_string(), "google chrome".to_string(), "chrome browser".to_string(), "chrome kholo".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open vs code".to_string(),
            aliases: vec!["open vscode".to_string(), "launch vscode".to_string(), "open visual studio code".to_string(), "start vscode".to_string(), "vs code kholo".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open notepad".to_string(),
            aliases: vec!["launch notepad".to_string(), "start notepad".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open explorer".to_string(),
            aliases: vec!["open file explorer".to_string(), "open files".to_string(), "open my files".to_string(), "launch explorer".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open terminal".to_string(),
            aliases: vec!["open cmd".to_string(), "open command prompt".to_string(), "open powershell".to_string(), "launch terminal".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open spotify".to_string(),
            aliases: vec!["launch spotify".to_string(), "start spotify".to_string(), "play spotify".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open discord".to_string(),
            aliases: vec!["launch discord".to_string(), "start discord".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "launch_app".to_string(),
            primary_name: "open chatgpt".to_string(),
            aliases: vec!["open chat gpt".to_string(), "launch chatgpt".to_string(), "open ai chat".to_string()],
        });

        // 2. close_app — primary name must include "close" or "exit" verb to avoid collision
        commands.push(CommandDefinition {
            intent: "close_app".to_string(),
            primary_name: "close chrome".to_string(),
            aliases: vec!["exit chrome".to_string(), "quit chrome".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "close_app".to_string(),
            primary_name: "close vs code".to_string(),
            aliases: vec!["close vscode".to_string(), "exit vs code".to_string(), "quit vscode".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "close_app".to_string(),
            primary_name: "close notepad".to_string(),
            aliases: vec!["exit notepad".to_string()],
        });

        // 3. search_web
        commands.push(CommandDefinition {
            intent: "search_web".to_string(),
            primary_name: "search".to_string(),
            aliases: vec!["google search".to_string(), "search youtube".to_string(), "lookup".to_string(), "search for".to_string()],
        });

        // 4. open_url
        commands.push(CommandDefinition {
            intent: "open_url".to_string(),
            primary_name: "open url".to_string(),
            aliases: vec!["go to website".to_string(), "open website".to_string(), "open page".to_string(), "navigate to".to_string()],
        });

        // 5. system_control
        commands.push(CommandDefinition {
            intent: "system_control".to_string(),
            primary_name: "lock".to_string(),
            aliases: vec!["lock screen".to_string(), "lock workstation".to_string(), "lock my computer".to_string(), "lock my pc".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "system_control".to_string(),
            primary_name: "sleep".to_string(),
            aliases: vec!["sleep computer".to_string(), "suspend system".to_string(), "sleep workstation".to_string(), "sleep my pc".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "system_control".to_string(),
            primary_name: "volume up".to_string(),
            aliases: vec!["increase volume".to_string(), "louder".to_string(), "turn up volume".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "system_control".to_string(),
            primary_name: "volume down".to_string(),
            aliases: vec!["decrease volume".to_string(), "quieter".to_string(), "turn down volume".to_string()],
        });
        commands.push(CommandDefinition {
            intent: "system_control".to_string(),
            primary_name: "mute".to_string(),
            aliases: vec!["mute sound".to_string(), "silence".to_string(), "mute volume".to_string()],
        });

        // 6. file_operation
        commands.push(CommandDefinition {
            intent: "file_operation".to_string(),
            primary_name: "search file".to_string(),
            aliases: vec!["find file".to_string(), "locate file".to_string()],
        });

        // Setup default user vocabulary
        let mut user_vocabulary = HashMap::new();
        user_vocabulary.insert("chrome".to_string(), "app".to_string());
        user_vocabulary.insert("vscode".to_string(), "app".to_string());
        user_vocabulary.insert("vs code".to_string(), "app".to_string());
        user_vocabulary.insert("code".to_string(), "app".to_string());
        user_vocabulary.insert("notepad".to_string(), "app".to_string());
        user_vocabulary.insert("explorer".to_string(), "app".to_string());
        user_vocabulary.insert("terminal".to_string(), "app".to_string());
        user_vocabulary.insert("spotify".to_string(), "app".to_string());
        user_vocabulary.insert("discord".to_string(), "app".to_string());
        user_vocabulary.insert("chatgpt".to_string(), "app".to_string());
        user_vocabulary.insert("files".to_string(), "app".to_string());
        user_vocabulary.insert("file".to_string(), "app".to_string());
        user_vocabulary.insert("file explorer".to_string(), "app".to_string());

        Self {
            commands,
            user_vocabulary,
        }
    }


    pub fn get_commands(&self) -> &[CommandDefinition] {
        &self.commands
    }

    pub fn get_user_vocabulary(&self) -> &HashMap<String, String> {
        &self.user_vocabulary
    }

    pub fn add_user_vocabulary(&mut self, word: &str, entity_type: &str) {
        self.user_vocabulary.insert(word.to_lowercase(), entity_type.to_string());
    }
}
