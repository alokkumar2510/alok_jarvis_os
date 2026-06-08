use std::collections::HashMap;
use crate::action_bus::{ActionRequest, ActionType};
use crate::intelligence::command_registry::{CommandRegistry, CommandDefinition};
use crate::intelligence::entity::EntityExtractor;

pub struct IntentEngine {
    registry: CommandRegistry,
}

impl IntentEngine {
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
        }
    }

    /// Parses transcribed user speech to determine offline desktop actions or planning tasks,
    /// employing Regexes, fuzzy matching, and confidence scores.
    pub fn parse_intent(&self, text: &str, plugin_commands: &[CommandDefinition]) -> Option<ActionRequest> {
        let mut clean_text = text.to_lowercase()
            .replace(".", "")
            .replace(",", "")
            .replace("?", "")
            .replace("!", "");
            
        // Homophone corrections for common Whisper anomalies
        clean_text = clean_text.split_whitespace()
            .map(|w| match w {
                "claude" | "clothes" | "closed" | "clothe" => "close",
                "piles" | "tiles" => "files",
                _ => w,
            })
            .collect::<Vec<&str>>()
            .join(" ");

        let text_trimmed = clean_text.trim();

        if text_trimmed.is_empty() {
            return None;
        }

        // Check for resume/continue command
        if text_trimmed == "continue" || text_trimmed == "resume" || text_trimmed == "go on" || text_trimmed == "what were you saying" || text_trimmed == "carry on" || text_trimmed == "go ahead" {
            let mut args = HashMap::new();
            args.insert("confidence_score".to_string(), "1.00".to_string());
            return Some(ActionRequest {
                action_type: ActionType::DesktopControl,
                category: "system".to_string(),
                action_id: "resume_speech".to_string(),
                args,
                timeout_seconds: None,
                max_retries: None,
            });
        }

        // Check for Remember/Store Memory command
        if text_trimmed.starts_with("remember:") || text_trimmed.starts_with("remember ") {
            let content = text.chars().skip(9).collect::<String>().trim().to_string();
            let mut args = HashMap::new();
            args.insert("content".to_string(), content);
            args.insert("confidence_score".to_string(), "1.00".to_string());
            return Some(ActionRequest {
                action_type: ActionType::MemoryAccess,
                category: "memory".to_string(),
                action_id: "store_memory".to_string(),
                args,
                timeout_seconds: None,
                max_retries: None,
            });
        }

        // Check for "who is X?", "what is X?", "where is X?"
        if text_trimmed.starts_with("who is ") || text_trimmed.starts_with("what is ") || text_trimmed.starts_with("where is ") {
            let query = if text_trimmed.starts_with("who is ") {
                text.chars().skip(7).collect::<String>().trim().trim_end_matches('?').to_string()
            } else if text_trimmed.starts_with("what is ") {
                text.chars().skip(8).collect::<String>().trim().trim_end_matches('?').to_string()
            } else {
                text.chars().skip(9).collect::<String>().trim().trim_end_matches('?').to_string()
            };
            let mut args = HashMap::new();
            args.insert("query".to_string(), query);
            args.insert("confidence_score".to_string(), "1.00".to_string());
            return Some(ActionRequest {
                action_type: ActionType::MemoryAccess,
                category: "memory".to_string(),
                action_id: "query_memory".to_string(),
                args,
                timeout_seconds: None,
                max_retries: None,
            });
        }

        println!("IntentEngine: Parsing text = '{}'", text_trimmed);

        let mut best_intent: Option<String> = None;
        let mut best_action_id: Option<String> = None;
        let mut best_score: f64 = 0.0;
        let mut matched_phrase = String::new();

        // Combine core commands and plugin commands
        let mut all_commands = self.registry.get_commands().to_vec();
        for pcmd in plugin_commands {
            all_commands.push(pcmd.clone());
        }

        // 1. Direct Pattern/Exact Match on Commands and Aliases
        for cmd in &all_commands {
            // Check primary name
            if text_trimmed.contains(&cmd.primary_name) {
                best_intent = Some(cmd.intent.clone());
                best_score = 0.9;
                matched_phrase = cmd.primary_name.clone();
            }
            // Check aliases
            for alias in &cmd.aliases {
                if text_trimmed == *alias {
                    best_intent = Some(cmd.intent.clone());
                    best_score = 1.0;
                    matched_phrase = alias.clone();
                    break;
                }
                if text_trimmed.contains(alias) {
                    best_intent = Some(cmd.intent.clone());
                    best_score = best_score.max(0.95);
                    matched_phrase = alias.clone();
                }
            }
        }

        // 2. Fuzzy Match on Command Registry Aliases (if confidence is still low)
        if best_score < 0.8 {
            for cmd in &all_commands {
                let score_primary = fuzzy_match(text_trimmed, &cmd.primary_name);
                if score_primary > best_score {
                    best_score = score_primary;
                    best_intent = Some(cmd.intent.clone());
                    matched_phrase = cmd.primary_name.clone();
                }
                for alias in &cmd.aliases {
                    let score_alias = fuzzy_match(text_trimmed, alias);
                    if score_alias > best_score {
                        best_score = score_alias;
                        best_intent = Some(cmd.intent.clone());
                        matched_phrase = alias.clone();
                    }
                }
            }
        }

        // 3. Fallback Keyword Rules if confidence is still low
        if best_score < 0.7 {
            if (text_trimmed.starts_with("open ") || text_trimmed.starts_with("launch ") || text_trimmed.starts_with("start ") || text_trimmed.ends_with(" kholo"))
                || (text_trimmed.contains("open ") && text_trimmed.split_whitespace().count() <= 5)
            {
                best_intent = Some("launch_app".to_string());
                best_action_id = Some("launch_app".to_string());
                best_score = 0.8;
            } else if text_trimmed.starts_with("close ") || text_trimmed.starts_with("exit ") || text_trimmed.starts_with("stop ") || text_trimmed.starts_with("kill ")
                || (text_trimmed.contains("close ") && text_trimmed.split_whitespace().count() <= 5)
            {
                best_intent = Some("close_app".to_string());
                best_action_id = Some("close_app".to_string());
                best_score = 0.8;
            } else if text_trimmed.contains("volume") || text_trimmed.contains("louder") || text_trimmed.contains("quieter") || text_trimmed.contains("mute") {
                best_intent = Some("system_control".to_string());
                best_action_id = Some(if text_trimmed.contains("up") || text_trimmed.contains("louder") { "volume_up".to_string() } else if text_trimmed.contains("mute") { "mute_volume".to_string() } else { "volume_down".to_string() });
                best_score = 0.85;
            } else if text_trimmed.contains("lock") {
                best_intent = Some("system_control".to_string());
                best_action_id = Some("lock_system".to_string());
                best_score = 0.8;
            } else if text_trimmed.contains("sleep") || text_trimmed.contains("suspend") {
                best_intent = Some("system_control".to_string());
                best_action_id = Some("sleep_system".to_string());
                best_score = 0.8;
            } else if text_trimmed.contains("search") || text_trimmed.contains("find file") {
                best_intent = Some("file_operation".to_string());
                best_action_id = Some("search_file".to_string());
                best_score = 0.8;
            } else if text_trimmed.starts_with("go to ") || text_trimmed.starts_with("open page ") || text_trimmed.starts_with("open url ") || text_trimmed.starts_with("open website ")
                || (text_trimmed.contains("website") && text_trimmed.split_whitespace().count() <= 5)
                || (text_trimmed.contains("go to") && text_trimmed.split_whitespace().count() <= 5)
            {
                best_intent = Some("open_url".to_string());
                best_action_id = Some("open_website".to_string());
                best_score = 0.8;
            } else if text_trimmed.contains("what is this error") || text_trimmed.contains("explain error") || text_trimmed.contains("solve error") || text_trimmed.contains("fix error")
                || text_trimmed.contains("why is this failing") || text_trimmed.contains("why is it failing") || text_trimmed.contains("why failing") || text_trimmed.contains("analyze this error") || text_trimmed.contains("analyze error")
            {
                best_intent = Some("analyze_error".to_string());
                best_action_id = Some("analyze_error".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("continue my work") || text_trimmed.contains("continue work") || text_trimmed.contains("resume my work") || text_trimmed.contains("resume work") {
                best_intent = Some("continue_work".to_string());
                best_action_id = Some("continue_work".to_string());
                best_score = 0.95;
            } else if text_trimmed.contains("search codebase") || text_trimmed.contains("find in codebase") || text_trimmed.contains("where is this function") || text_trimmed.contains("show me where this function is used") || text_trimmed.contains("search symbols") || text_trimmed.contains("search symbol") {
                best_intent = Some("search_codebase".to_string());
                best_action_id = Some("search_codebase".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("check system health") || text_trimmed.contains("how is the system") || text_trimmed.contains("check health") || text_trimmed.contains("system health") || text_trimmed.contains("system stats") {
                best_intent = Some("system_health".to_string());
                best_action_id = Some("system_health".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("how did i fix") || text_trimmed.contains("recall build fix") || text_trimmed.contains("how did i resolve") {
                best_intent = Some("recall_build_fix".to_string());
                best_action_id = Some("recall_build_fix".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("explain code") || text_trimmed.contains("what does this code do") || text_trimmed.contains("explain this code") {
                best_intent = Some("explain_code".to_string());
                best_action_id = Some("explain_code".to_string());
                best_score = 0.85;
            } else if text_trimmed.contains("what is on my screen") || text_trimmed.contains("summarize my screen") || text_trimmed.contains("summarize screen") || text_trimmed.contains("what's on my screen") {
                best_intent = Some("summarize_screen".to_string());
                best_action_id = Some("summarize_screen".to_string());
                best_score = 0.85;
            } else if text_trimmed.contains("screen") || text_trimmed.contains("ocr") || text_trimmed.contains("capture") {
                best_intent = Some("screen_query".to_string());
                best_action_id = Some("capture_ocr".to_string());
                best_score = 0.8;
            } else if text_trimmed.contains("flutter") || text_trimmed.contains("plan") || text_trimmed.contains("set up") {
                best_intent = Some("planner_goal".to_string());
                best_action_id = Some("execute_plan".to_string());
                best_score = 0.85;
            } else if text_trimmed.contains("summarize this page") || text_trimmed.contains("summarize page") || text_trimmed.contains("summarize this website") {
                best_intent = Some("summarize_page".to_string());
                best_action_id = Some("summarize_page".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("yesterday's article") || text_trimmed.contains("yesterday article") || text_trimmed.contains("reading yesterday") {
                best_intent = Some("yesterday_article".to_string());
                best_action_id = Some("yesterday_article".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("close duplicate") || text_trimmed.contains("clean duplicate") || text_trimmed.contains("remove duplicate tabs") {
                best_intent = Some("close_duplicate_tabs".to_string());
                best_action_id = Some("close_duplicate_tabs".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("search youtube") || text_trimmed.contains("youtube search") {
                best_intent = Some("search_youtube".to_string());
                best_action_id = Some("search_youtube".to_string());
                best_score = 0.9;
            } else if text_trimmed.contains("switch to") && (text_trimmed.contains("mode") || text_trimmed.contains("personality") || text_trimmed.contains("voice")) {
                best_intent = Some("switch_personality".to_string());
                best_action_id = Some("switch_personality".to_string());
                best_score = 0.9;
            }
        }

        println!("IntentEngine: Matched Intent = {:?}, Score = {:.2}", best_intent, best_score);

        // Reject if match confidence is too low
        if best_score < 0.6 {
            println!("IntentEngine: Confidence too low. Falling back to general conversation.");
            best_intent = Some("conversation".to_string());
            best_action_id = Some("query".to_string());
            best_score = 0.7;
        }

        let matched_intent = best_intent?;
        
        // Determine action type, category, and action_id based on matched intent
        let (action_type, category, act_id) = match matched_intent.as_str() {
            "launch_app" => (ActionType::DesktopControl, "app".to_string(), "launch_app".to_string()),
            "close_app" => (ActionType::DesktopControl, "app".to_string(), "close_app".to_string()),
            "search_web" => (ActionType::BrowserControl, "browser".to_string(), "search_web".to_string()),
            "open_url" => (ActionType::BrowserControl, "browser".to_string(), "open_website".to_string()),
            "system_control" => {
                let act = best_action_id.clone().unwrap_or_else(|| {
                    if text_trimmed.contains("sleep") || text_trimmed.contains("suspend") {
                        "sleep_system".to_string()
                    } else {
                        "lock_system".to_string()
                    }
                });
                (ActionType::DesktopControl, "system".to_string(), act)
            }
            "file_operation" => (ActionType::MemoryAccess, "file".to_string(), "search_file".to_string()),
            "browser_control" => (ActionType::BrowserControl, "browser".to_string(), "browser_control".to_string()),
            "memory_query" => (ActionType::MemoryAccess, "system".to_string(), "read_setting".to_string()),
            "screen_query" => (ActionType::ScreenAnalysis, "system".to_string(), "capture_ocr".to_string()),
            "analyze_error" => (ActionType::ScreenAnalysis, "system".to_string(), "analyze_error".to_string()),
            "explain_code" => (ActionType::ScreenAnalysis, "system".to_string(), "explain_code".to_string()),
            "summarize_screen" => (ActionType::ScreenAnalysis, "system".to_string(), "summarize_screen".to_string()),
            "planner_goal" => (ActionType::PlannerExecution, "system".to_string(), "execute_plan".to_string()),
            "conversation" => (ActionType::GroqReasoning, "system".to_string(), "query".to_string()),
            "summarize_page" => (ActionType::BrowserControl, "browser".to_string(), "summarize_page".to_string()),
            "yesterday_article" => (ActionType::BrowserControl, "browser".to_string(), "yesterday_article".to_string()),
            "close_duplicate_tabs" => (ActionType::BrowserControl, "browser".to_string(), "close_duplicate_tabs".to_string()),
            "search_youtube" => (ActionType::BrowserControl, "browser".to_string(), "search_youtube".to_string()),
            "switch_personality" => (ActionType::DesktopControl, "system".to_string(), "switch_personality".to_string()),
            "continue_work" => (ActionType::DesktopControl, "system".to_string(), "continue_work".to_string()),
            "search_codebase" => (ActionType::MemoryAccess, "system".to_string(), "search_codebase".to_string()),
            "system_health" => (ActionType::DesktopControl, "system".to_string(), "system_health".to_string()),
            "recall_build_fix" => (ActionType::MemoryAccess, "system".to_string(), "recall_build_fix".to_string()),
            _ => {
                // If it is a plugin-specific intent, map to ActionType::PluginExecution
                if plugin_commands.iter().any(|c| c.intent == matched_intent) {
                    (ActionType::PluginExecution, "plugin".to_string(), matched_intent.clone())
                } else {
                    return None;
                }
            }
        };

        // Extract entities
        let mut args = EntityExtractor::extract_entities(&clean_text, &matched_intent, self.registry.get_user_vocabulary());
        
        // Add confidence score to args
        args.insert("confidence_score".to_string(), format!("{:.2}", best_score));

        // For plugin commands, extract argument suffix if not already extracted
        if !matched_phrase.is_empty() {
            let suffix = text_trimmed.replace(&matched_phrase, "").trim().to_string();
            if !suffix.is_empty() && !args.contains_key("query") && !args.contains_key("goal") && !args.contains_key("app_name") && !args.contains_key("url") && !args.contains_key("filename") {
                args.insert("suffix".to_string(), suffix.clone());
                args.insert("target".to_string(), suffix.clone());
                args.insert("path".to_string(), suffix.clone());
            }
        }

        Some(ActionRequest {
            action_type,
            category,
            action_id: act_id,
            args,
            timeout_seconds: None,
            max_retries: None,
        })
    }
}

// ── Fuzzy Matching Helper Functions ────────────────────────────────────────

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();
    
    let mut dp = vec![vec![0; len2 + 1]; len1 + 1];
    for i in 0..=len1 { dp[i][0] = i; }
    for j in 0..=len2 { dp[0][j] = j; }
    
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
            dp[i][j] = std::cmp::min(
                std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                dp[i - 1][j - 1] + cost
            );
        }
    }
    dp[len1][len2]
}

fn fuzzy_match(input: &str, target: &str) -> f64 {
    let dist = levenshtein_distance(input, target);
    let max_len = std::cmp::max(input.len(), target.len());
    if max_len == 0 {
        return 1.0;
    }
    1.0 - (dist as f64 / max_len as f64)
}
