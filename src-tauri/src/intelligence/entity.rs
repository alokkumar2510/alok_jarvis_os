use std::collections::HashMap;

pub struct EntityExtractor;

impl EntityExtractor {
    pub fn extract_entities(text: &str, intent: &str, user_vocab: &HashMap<String, String>) -> HashMap<String, String> {
        let text_lower = text.to_lowercase();
        let mut entities = HashMap::new();

        match intent {
            "launch_app" => {
                let mut app_name = String::new();

                // Strip "open X in/inside [browser]" → extract just X as app_name
                // e.g. "open instagram web inside chrome" → app_name = "instagram"
                let browser_qualifiers = [" inside chrome", " in chrome", " inside firefox", " in firefox",
                                          " inside edge", " in edge", " inside browser", " in browser",
                                          " inside the browser", " in the browser"];
                let cleaned = text_lower.clone();
                for bq in &browser_qualifiers {
                    if cleaned.contains(bq) {
                        // This is actually an open_url command, not a launch_app
                        // Extract the site name and insert as url instead
                        let site = cleaned.replace(bq, "")
                            .replace("open ", "")
                            .replace(" web", "").replace(" site", "").replace(" page", "")
                            .trim().to_string();
                        entities.insert("url".to_string(), site);
                        // Signal to caller that this should be treated as open_url
                        entities.insert("_redirect_intent".to_string(), "open_url".to_string());
                        return entities;
                    }
                }

                if text_lower.starts_with("open ") {
                    app_name = text_lower.replace("open ", "").trim().to_string();
                } else if text_lower.starts_with("launch ") {
                    app_name = text_lower.replace("launch ", "").trim().to_string();
                } else if text_lower.starts_with("start ") {
                    app_name = text_lower.replace("start ", "").trim().to_string();
                } else if text_lower.ends_with(" kholo") {
                    app_name = text_lower.replace(" kholo", "").trim().to_string();
                }

                // Strip trailing web/site/page from app name
                let app_name = app_name
                    .trim_end_matches(" web").trim_end_matches(" site")
                    .trim_end_matches(" page").trim_end_matches(" app").to_string();

                let cleaned_name = clean_app_name(&app_name, user_vocab);
                if !cleaned_name.is_empty() {
                    entities.insert("app_name".to_string(), cleaned_name);
                    return entities;
                }

                // Check vocabulary fallback
                for word in text_lower.split_whitespace() {
                    if let Some(val) = user_vocab.get(word) {
                        if val == "app" {
                            entities.insert("app_name".to_string(), word.to_string());
                            return entities;
                        }
                    }
                }
            }
            "close_app" => {
                let mut app_name = String::new();
                if text_lower.starts_with("close ") {
                    app_name = text_lower.replace("close ", "").trim().to_string();
                } else if text_lower.starts_with("exit ") {
                    app_name = text_lower.replace("exit ", "").trim().to_string();
                } else if text_lower.starts_with("stop ") {
                    app_name = text_lower.replace("stop ", "").trim().to_string();
                } else if text_lower.starts_with("kill ") {
                    app_name = text_lower.replace("kill ", "").trim().to_string();
                }
                let cleaned_name = clean_app_name(&app_name, user_vocab);
                if !cleaned_name.is_empty() {
                    entities.insert("app_name".to_string(), cleaned_name);
                }
            }
            "search_web" => {
                let mut query = String::new();
                if text_lower.starts_with("search for ") {
                    query = text_lower.replace("search for ", "").trim().to_string();
                } else if text_lower.starts_with("search ") {
                    query = text_lower.replace("search ", "").trim().to_string();
                }
                
                // Strip trailing " on youtube", " on google", etc. if present
                if !query.is_empty() {
                    let clean_query = if query.ends_with(" on youtube") {
                        query.replace(" on youtube", "")
                    } else if query.ends_with(" on google") {
                        query.replace(" on google", "")
                    } else {
                        query
                    };
                    entities.insert("query".to_string(), clean_query.trim().to_string());
                }
            }
            "open_url" => {
                let mut url = String::new();
                if text_lower.starts_with("go to ") {
                    url = text_lower.replace("go to ", "").trim().to_string();
                } else if text_lower.starts_with("open page ") {
                    url = text_lower.replace("open page ", "").trim().to_string();
                } else if text_lower.starts_with("open url ") {
                    url = text_lower.replace("open url ", "").trim().to_string();
                } else if text_lower.starts_with("open website ") {
                    url = text_lower.replace("open website ", "").trim().to_string();
                }
                
                if url.is_empty() {
                    let mut u = text_lower.clone();
                    for suffix in &[" website", " site", " page", " web", " url"] {
                        if u.ends_with(suffix) {
                            u = u[..u.len() - suffix.len()].trim().to_string();
                        }
                    }
                    for prefix in &["open ", "go to "] {
                        if u.starts_with(prefix) {
                            u = u[prefix.len()..].trim().to_string();
                        }
                    }
                    url = u;
                }

                if !url.is_empty() {
                    entities.insert("url".to_string(), url);
                }
            }
            "file_operation" => {
                let mut file = String::new();
                if text_lower.starts_with("search file ") {
                    file = text_lower.replace("search file ", "").trim().to_string();
                } else if text_lower.starts_with("find file ") {
                    file = text_lower.replace("find file ", "").trim().to_string();
                } else if text_lower.starts_with("locate file ") {
                    file = text_lower.replace("locate file ", "").trim().to_string();
                } else if text_lower.starts_with("open file ") {
                    file = text_lower.replace("open file ", "").trim().to_string();
                } else if text_lower.starts_with("search ") {
                    file = text_lower.replace("search ", "").trim().to_string();
                }
                if !file.is_empty() {
                    entities.insert("filename".to_string(), file);
                }
            }
            "planner_goal" => {
                let mut goal = String::new();
                if text_lower.starts_with("set up ") {
                    goal = text_lower.replace("set up ", "").trim().to_string();
                } else if text_lower.starts_with("setup ") {
                    goal = text_lower.replace("setup ", "").trim().to_string();
                } else if text_lower.starts_with("plan ") {
                    goal = text_lower.replace("plan ", "").trim().to_string();
                } else if text_lower.starts_with("planner goal ") {
                    goal = text_lower.replace("planner goal ", "").trim().to_string();
                }
                if !goal.is_empty() {
                    entities.insert("goal".to_string(), goal);
                }
            }
            "search_youtube" => {
                let mut query = String::new();
                if text_lower.starts_with("search youtube for ") {
                    query = text_lower.replace("search youtube for ", "").trim().to_string();
                } else if text_lower.starts_with("search youtube ") {
                    query = text_lower.replace("search youtube ", "").trim().to_string();
                } else if text_lower.contains("youtube") {
                    query = text_lower.replace("search ", "").replace("youtube ", "").replace("for ", "").trim().to_string();
                }
                if !query.is_empty() {
                    entities.insert("query".to_string(), query);
                }
            }
            "switch_personality" => {
                let personality = if text_lower.contains("professional") {
                    "professional"
                } else if text_lower.contains("friendly") {
                    "friendly"
                } else if text_lower.contains("assistant") {
                    "assistant"
                } else if text_lower.contains("companion") {
                    "companion"
                } else {
                    "assistant"
                };
                entities.insert("personality".to_string(), personality.to_string());
            }
            "search_codebase" => {
                let mut query = String::new();
                if text_lower.contains("search codebase for ") {
                    query = text_lower.replace("search codebase for ", "");
                } else if text_lower.contains("find in codebase ") {
                    query = text_lower.replace("find in codebase ", "");
                } else if text_lower.contains("search symbols for ") {
                    query = text_lower.replace("search symbols for ", "");
                } else if text_lower.contains("search symbol for ") {
                    query = text_lower.replace("search symbol for ", "");
                } else if text_lower.starts_with("show me where ") && text_lower.ends_with(" is used") {
                    query = text_lower.replace("show me where ", "").replace(" is used", "");
                } else if text_lower.contains("where is ") {
                    query = text_lower.replace("where is ", "");
                }
                
                if query.is_empty() {
                    query = text_lower.replace("search codebase", "")
                        .replace("find in codebase", "")
                        .replace("search symbols", "")
                        .replace("search symbol", "")
                        .trim().to_string();
                }
                entities.insert("query".to_string(), query.trim().to_string());
            }
            "recall_build_fix" => {
                let mut query = String::new();
                if text_lower.contains("how did i fix ") {
                    query = text_lower.replace("how did i fix ", "");
                } else if text_lower.contains("how did i resolve ") {
                    query = text_lower.replace("how did i resolve ", "");
                } else if text_lower.contains("recall build fix for ") {
                    query = text_lower.replace("recall build fix for ", "");
                } else if text_lower.contains("recall build fix ") {
                    query = text_lower.replace("recall build fix ", "");
                }
                if query.is_empty() {
                    query = text_lower.replace("how did i fix", "")
                        .replace("recall build fix", "")
                        .replace("how did i resolve", "")
                        .trim().to_string();
                }
                entities.insert("query".to_string(), query.trim().to_string());
            }
            "conversation" => {
                entities.insert("prompt".to_string(), text.to_string());
            }
            _ => {}
        }
        entities
    }
}

fn clean_app_name(raw_name: &str, user_vocab: &HashMap<String, String>) -> String {
    let name = raw_name.to_lowercase()
        .replace("the ", "")
        .replace("application", "")
        .replace("program", "")
        .replace("window", "")
        .trim()
        .to_string();
        
    // Check if the cleaned name contains any known app word from vocabulary
    for word in name.split_whitespace() {
        if let Some(val) = user_vocab.get(word) {
            if val == "app" {
                return word.to_string();
            }
        }
    }
    // Fallback to substring match check
    for (key, val) in user_vocab {
        if val == "app" && name.contains(key) {
            return key.clone();
        }
    }
    name
}
