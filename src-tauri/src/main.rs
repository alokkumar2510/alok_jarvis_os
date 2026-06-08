// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    alok_jarvis_os_lib::run()
}

#[cfg(test)]
mod tests {
    use alok_jarvis_os_lib::conversation_context::ConversationContext;
    use alok_jarvis_os_lib::database::{Database, Node, Edge};
    use alok_jarvis_os_lib::planner::PlannerEngine;
    use alok_jarvis_os_lib::action_bus::{ActionBus, ActionRequest, ActionType};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_conversation_context_history() {
        let mut ctx = ConversationContext::new();
        ctx.add_history("User", "Hello Jarvis");
        ctx.add_history("Alok", "Hello Alok");
        
        assert_eq!(ctx.turn_count, 2);
        assert_eq!(ctx.turns.len(), 2);
        assert_eq!(ctx.turns[0].sender, "User");
        assert_eq!(ctx.turns[1].sender, "Alok");
    }

    #[test]
    fn test_cap_history() {
        let mut ctx = ConversationContext::new();
        for i in 0..60 {
            ctx.add_history("User", &format!("msg {}", i));
        }
        assert_eq!(ctx.turns.len(), 50);
        assert_eq!(ctx.turns[0].message, "msg 10");
        assert_eq!(ctx.turns[49].message, "msg 59");
    }

    #[test]
    fn test_variables() {
        let mut ctx = ConversationContext::new();
        ctx.set_variable("it", "file.txt");
        assert_eq!(ctx.get_variable("it"), Some(&"file.txt".to_string()));
    }

    #[test]
    fn test_db_operations() {
        let db = Database::new(":memory:").expect("Failed to create in-memory database");
        
        // Test settings
        db.set_setting("groq_model", "llama3").unwrap();
        assert_eq!(db.get_setting("groq_model").unwrap(), Some("llama3".to_string()));
        assert_eq!(db.get_setting("non_existent").unwrap(), None);

        // Test nodes
        let test_node = Node {
            id: "person_1".to_string(),
            label: "Alok Kumar Sahu".to_string(),
            node_type: "person".to_string(),
            metadata: "{}".to_string(),
        };
        db.add_node(&test_node).unwrap();
        let nodes = db.get_nodes().unwrap();
        let found = nodes.iter().any(|n| n.id == "person_1" && n.label == "Alok Kumar Sahu");
        assert!(found);

        // Test edges
        let test_edge = Edge {
            from_id: "user".to_string(),
            to_id: "chrome".to_string(),
            relation: "Launches".to_string(),
            metadata: "{}".to_string(),
        };
        db.add_edge(&test_edge).unwrap();
        let edges = db.get_edges().unwrap();
        let found_edge = edges.iter().any(|e| e.from_id == "user" && e.to_id == "chrome" && e.relation == "Launches");
        assert!(found_edge);
    }

    #[test]
    fn test_generate_plan_flutter() {
        let db = Database::new(":memory:").unwrap();
        let planner = PlannerEngine::new();
        let plan = planner.generate_plan("set up flutter environment", &db);
        assert_eq!(plan.goal, "set up flutter environment");
        assert!(plan.steps.len() >= 3);
        assert!(!plan.steps[0].name.is_empty());
        assert_eq!(plan.steps[0].status, "Pending");
    }

    #[tokio::test]
    async fn test_action_bus_routing() {
        let db = Arc::new(Mutex::new(Database::new(":memory:").unwrap()));
        let context = Arc::new(Mutex::new(ConversationContext::new()));
        let bus = ActionBus::new(db, context);

        let req = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "device".to_string(),
            action_id: "volume_up".to_string(),
            args: HashMap::new(),
            timeout_seconds: None,
            max_retries: None,
        };

        let res = bus.execute_device_action(&req);
        assert_eq!(res.output, "Volume increased");

        let req_err = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "app".to_string(),
            action_id: "invalid_action".to_string(),
            args: HashMap::new(),
            timeout_seconds: None,
            max_retries: None,
        };
        let res = bus.execute_app_action(&req_err);
        assert!(!res.success);
        assert!(res.error.unwrap().contains("Unsupported app action"));
    }

    #[tokio::test]
    async fn test_sqlite_memory_system() {
        let db = Database::new(":memory:").expect("Failed to create in-memory database");
        
        // 1. Store memory
        let tags = vec!["person".to_string(), "interests".to_string()];
        let mem_id = db.store_memory("Neha likes coffee.", tags).unwrap();
        assert!(mem_id > 0);

        // 2. Search memory
        let search_results = db.search_memories("coffee").unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0], "Neha likes coffee.");

        // 3. Update memory
        db.update_memory(mem_id, "Neha likes espresso.").unwrap();
        let search_results2 = db.search_memories("espresso").unwrap();
        assert_eq!(search_results2.len(), 1);
        assert_eq!(search_results2[0], "Neha likes espresso.");

        // 4. Entities and Relationships
        db.add_entity("neha", "Neha", "person", "A friend").unwrap();
        db.add_entity("coffee", "coffee", "concept", "A beverage").unwrap();
        db.add_relationship("neha", "coffee", "likes").unwrap();

        // 5. Relationship query
        let rels = db.query_relationships("Neha").unwrap();
        assert_eq!(rels.len(), 1);
        assert_eq!(rels[0], ("Neha".to_string(), "likes".to_string(), "coffee".to_string()));

        // 6. User Preferences
        db.set_user_preference("theme", "dark").unwrap();
        assert_eq!(db.get_user_preference("theme").unwrap(), Some("dark".to_string()));

        // 7. Conversation History
        db.add_conversation_history("User", "Who is Neha?").unwrap();
        let history = db.get_conversation_history(1).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].sender, "User");
        assert_eq!(history[0].message, "Who is Neha?");

        // 8. Forget memory
        db.forget_memory(mem_id).unwrap();
        let search_results3 = db.search_memories("espresso").unwrap();
        assert_eq!(search_results3.len(), 0);
    }

    #[test]
    fn test_memory_intent_parsing() {
        use alok_jarvis_os_lib::intelligence::IntentEngine;
        let engine = IntentEngine::new();

        // Test Remember intent
        let req1 = engine.parse_intent("Remember: Neha likes coffee.", &[]).unwrap();
        assert_eq!(req1.action_type, ActionType::MemoryAccess);
        assert_eq!(req1.action_id, "store_memory");
        assert_eq!(req1.args.get("content").unwrap(), "Neha likes coffee.");

        // Test Who/What is queries
        let req2 = engine.parse_intent("Who is Neha?", &[]).unwrap();
        assert_eq!(req2.action_type, ActionType::MemoryAccess);
        assert_eq!(req2.action_id, "query_memory");
        assert_eq!(req2.args.get("query").unwrap(), "Neha");

        let req3 = engine.parse_intent("what is vscode?", &[]).unwrap();
        assert_eq!(req3.action_type, ActionType::MemoryAccess);
        assert_eq!(req3.action_id, "query_memory");
        assert_eq!(req3.args.get("query").unwrap(), "vscode");
    }

    #[test]
    fn test_resume_intent_parsing() {
        use alok_jarvis_os_lib::intelligence::IntentEngine;
        let engine = IntentEngine::new();

        let req1 = engine.parse_intent("resume", &[]).unwrap();
        assert_eq!(req1.action_type, ActionType::DesktopControl);
        assert_eq!(req1.category, "system");
        assert_eq!(req1.action_id, "resume_speech");

        let req2 = engine.parse_intent("continue", &[]).unwrap();
        assert_eq!(req2.action_id, "resume_speech");

        let req3 = engine.parse_intent("what were you saying", &[]).unwrap();
        assert_eq!(req3.action_id, "resume_speech");
    }

    #[tokio::test]
    async fn test_conversation_context_tracking() {
        let mut ctx = ConversationContext::new();

        // 1. Initial State
        assert_eq!(ctx.last_intent, None);
        assert_eq!(ctx.last_app, None);

        // 2. Update context
        ctx.update_context(
            Some("launch_app".to_string()),
            Some("Chrome".to_string()),
            None,
            None,
            Some("launch_app".to_string()),
        );

        assert_eq!(ctx.last_intent, Some("launch_app".to_string()));
        assert_eq!(ctx.last_app, Some("Chrome".to_string()));

        // 3. Resolve pronouns
        let resolved = ctx.resolve_text("close it");
        assert_eq!(resolved, "close Chrome");

        // 4. Resolve args
        let mut args = HashMap::new();
        args.insert("app_name".to_string(), "it".to_string());
        ctx.resolve_args(&mut args);
        assert_eq!(args.get("app_name").unwrap(), "Chrome");

        // 5. Verify URL resolution
        ctx.update_context(
            None,
            None,
            None,
            Some("google.com".to_string()),
            None,
        );
        let resolved_url = ctx.resolve_text("search that");
        assert_eq!(resolved_url, "search google.com");
    }

    #[test]
    fn test_jarvis_runtime_definition() {
        // Verify orchestrator type compiles and is accessible
        let _r = alok_jarvis_os_lib::jarvis_runtime::JarvisRuntime;
        assert!(true);
    }

    struct MockWindowManager {
        has_win: Mutex<bool>,
        is_visible: Mutex<bool>,
        create_called: Mutex<bool>,
        show_called: Mutex<bool>,
        hide_called: Mutex<bool>,
    }

    impl alok_jarvis_os_lib::WindowManager for MockWindowManager {
        fn has_window(&self, _label: &str) -> bool {
            *self.has_win.lock().unwrap()
        }
        fn is_window_visible(&self, _label: &str) -> Result<bool, String> {
            Ok(*self.is_visible.lock().unwrap())
        }
        fn hide_window(&self, _label: &str) -> Result<(), String> {
            *self.is_visible.lock().unwrap() = false;
            *self.hide_called.lock().unwrap() = true;
            Ok(())
        }
        fn show_and_focus_window(&self, _label: &str) -> Result<(), String> {
            *self.is_visible.lock().unwrap() = true;
            *self.show_called.lock().unwrap() = true;
            Ok(())
        }
        fn create_console_window(&self) -> Result<(), String> {
            *self.has_win.lock().unwrap() = true;
            *self.is_visible.lock().unwrap() = true;
            *self.create_called.lock().unwrap() = true;
            Ok(())
        }
    }

    #[test]
    fn test_toggle_developer_console_logic() {
        // Case 1: Window does not exist yet. It should create it.
        let mock = MockWindowManager {
            has_win: Mutex::new(false),
            is_visible: Mutex::new(false),
            create_called: Mutex::new(false),
            show_called: Mutex::new(false),
            hide_called: Mutex::new(false),
        };
        let res = alok_jarvis_os_lib::toggle_developer_console_inner(&mock);
        assert!(res.is_ok());
        assert!(*mock.create_called.lock().unwrap());
        assert!(*mock.has_win.lock().unwrap());
        assert!(*mock.is_visible.lock().unwrap());

        // Case 2: Window exists and is visible. It should hide it.
        let mock = MockWindowManager {
            has_win: Mutex::new(true),
            is_visible: Mutex::new(true),
            create_called: Mutex::new(false),
            show_called: Mutex::new(false),
            hide_called: Mutex::new(false),
        };
        let res = alok_jarvis_os_lib::toggle_developer_console_inner(&mock);
        assert!(res.is_ok());
        assert!(*mock.hide_called.lock().unwrap());
        assert!(!*mock.is_visible.lock().unwrap());

        // Case 3: Window exists and is hidden. It should show and focus it.
        let mock = MockWindowManager {
            has_win: Mutex::new(true),
            is_visible: Mutex::new(false),
            create_called: Mutex::new(false),
            show_called: Mutex::new(false),
            hide_called: Mutex::new(false),
        };
        let res = alok_jarvis_os_lib::toggle_developer_console_inner(&mock);
        assert!(res.is_ok());
        assert!(*mock.show_called.lock().unwrap());
        assert!(*mock.is_visible.lock().unwrap());
    }

    #[test]
    fn test_fallback_url_extraction() {
        use alok_jarvis_os_lib::intelligence::IntentEngine;
        let engine = IntentEngine::new();

        // Test "instagram website" fallback extraction
        let req1 = engine.parse_intent("instagram website", &[]).unwrap();
        assert_eq!(req1.action_id, "open_website");
        assert_eq!(req1.args.get("url").unwrap(), "instagram");

        // Test "open instagram inside chrome" redirect intent
        let req2 = engine.parse_intent("open instagram inside chrome", &[]).unwrap();
        assert_eq!(req2.action_id, "launch_app");
        assert_eq!(req2.args.get("_redirect_intent").unwrap(), "open_url");
        assert_eq!(req2.args.get("url").unwrap(), "instagram");
    }

    #[test]
    fn test_verb_sharing_and_refined_intents() {
        use alok_jarvis_os_lib::intelligence::IntentEngine;
        let engine = IntentEngine::new();

        // 1. Long statement containing command words should fall back to conversation
        let req1 = engine.parse_intent("my voice is not transcripting to text so facebook website inside chrome you can perform", &[]).unwrap();
        assert_eq!(req1.action_id, "query");
        assert_eq!(req1.action_type, ActionType::GroqReasoning);

        // 2. Short statement with command words matches command intent
        let req2 = engine.parse_intent("facebook website", &[]).unwrap();
        assert_eq!(req2.action_id, "open_website");
        assert_eq!(req2.action_type, ActionType::BrowserControl);
     }

    #[test]
    fn test_split_chained_commands() {
        use alok_jarvis_os_lib::split_chained_commands;

        // Test split using "and then"
        let split1 = split_chained_commands("open chrome and then go to facebook");
        assert_eq!(split1, vec!["open chrome", "go to facebook"]);

        // Test split using "then"
        let split2 = split_chained_commands("mute audio then volume up");
        assert_eq!(split2, vec!["mute audio", "volume up"]);

        // Test split using "and" with verb sharing/both are commands
        let split3 = split_chained_commands("open chrome and open vscode");
        assert_eq!(split3, vec!["open chrome", "open vscode"]);
    }

    #[test]
    fn test_emotion_detection_rules() {
        use alok_jarvis_os_lib::voice::emotion_engine::{Emotion, EmotionEngine};

        // Test excited keyword
        let emotion_res = EmotionEngine::detect_emotion("Everything is working perfectly! This is amazing!", &[]);
        assert_eq!(emotion_res.emotion, Emotion::Excited);

        // Test frustrated keyword
        let emotion_res2 = EmotionEngine::detect_emotion("this is broken and useless", &[]);
        assert_eq!(emotion_res2.emotion, Emotion::Frustrated);
    }

    #[test]
    fn test_personality_mode_switching() {
        use alok_jarvis_os_lib::voice::personality_engine::PersonalityMode;

        // Switch to Friendly mode command
        let p_friendly = PersonalityMode::detect_switch_command("switch to friendly mode");
        assert_eq!(p_friendly, Some(PersonalityMode::Friendly));

        // Switch to Developer mode command
        let p_dev = PersonalityMode::detect_switch_command("please change to developer mode");
        assert_eq!(p_dev, Some(PersonalityMode::Developer));
    }

    #[test]
    fn test_speech_style_mapping() {
        use alok_jarvis_os_lib::voice::emotion_engine::Emotion;
        use alok_jarvis_os_lib::voice::personality_engine::PersonalityMode;
        use alok_jarvis_os_lib::voice::speech_style_engine::SpeechStyleEngine;

        // Developer mode + Focused emotion should be faster and less warm
        let style = SpeechStyleEngine::determine_style(Emotion::Focused, PersonalityMode::Developer);
        assert!(style.speed > 1.1);
        assert!(style.warmth < 0.5);

        // Companion mode + Sad emotion should be slower and warmer
        let style2 = SpeechStyleEngine::determine_style(Emotion::Sad, PersonalityMode::Companion);
        assert!(style2.speed < 0.9);
        assert!(style2.warmth > 0.8);
    }

    #[test]
    fn test_voice_preferences_commands() {
        use alok_jarvis_os_lib::voice::voice_preferences::VoicePreferences;
        use alok_jarvis_os_lib::voice::personality_engine::PersonalityMode;

        let db = Database::new(":memory:").unwrap();
        let mut pref = VoicePreferences::default_preferences();

        // 1. Switch personality
        let res1 = pref.handle_voice_commands("switch to companion mode", &db);
        assert!(res1.is_some());
        assert_eq!(pref.personality_mode, PersonalityMode::Companion);

        // 2. Speed slower
        let res2 = pref.handle_voice_commands("speak slower", &db);
        assert!(res2.is_some());
        assert!(pref.speaking_speed < 1.0);

        // 3. Voice override
        let res3 = pref.handle_voice_commands("use adam voice", &db);
        assert!(res3.is_some());
        assert_eq!(pref.preferred_voice, "am_adam.onnx");
    }

    #[tokio::test]
    async fn test_command_arbitrator_preemption_and_queueing() {
        use alok_jarvis_os_lib::action_bus::{ActionRequest, ActionType, ActionResponse};
        use alok_jarvis_os_lib::command_arbitrator::ARBITRATOR;
        use std::collections::HashMap;

        // 1. Setup two conflicting commands with High priority
        let mut args_launch = HashMap::new();
        args_launch.insert("app_name".to_string(), "chrome".to_string());
        let req_launch = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "app".to_string(),
            action_id: "launch_app".to_string(),
            args: args_launch,
            timeout_seconds: None,
            max_retries: None,
        };

        let mut args_close = HashMap::new();
        args_close.insert("app_name".to_string(), "chrome".to_string());
        let req_close = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "app".to_string(),
            action_id: "close_app".to_string(),
            args: args_close,
            timeout_seconds: None,
            max_retries: None,
        };

        let first_task = tokio::spawn(async move {
            ARBITRATOR.arbitrate_and_execute_with(req_launch, || async {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                ActionResponse {
                    success: true,
                    output: "Launch finished".to_string(),
                    error: None,
                }
            }).await
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let res2 = ARBITRATOR.arbitrate_and_execute_with(req_close, || async {
            ActionResponse {
                success: true,
                output: "Close finished".to_string(),
                error: None,
            }
        }).await;

        let res1 = first_task.await.unwrap();

        // Assert first is preempted (newest wins), second succeeds
        assert!(!res1.success);
        assert!(res1.error.unwrap().contains("cancelled"));
        assert!(res2.success);
        assert_eq!(res2.output, "Close finished");

        // 2. Test priority queueing (Low priority focuses chrome while launch_app is running)
        let mut args_focus = HashMap::new();
        args_focus.insert("app_name".to_string(), "chrome".to_string());
        let req_focus = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "app".to_string(),
            action_id: "focus_app".to_string(), // Focus is Low priority
            args: args_focus,
            timeout_seconds: None,
            max_retries: None,
        };

        let mut args_launch2 = HashMap::new();
        args_launch2.insert("app_name".to_string(), "chrome".to_string());
        let req_launch2 = ActionRequest {
            action_type: ActionType::DesktopControl,
            category: "app".to_string(),
            action_id: "launch_app".to_string(), // Launch is High priority
            args: args_launch2,
            timeout_seconds: None,
            max_retries: None,
        };

        // Start launch (High priority) which takes 100ms
        let launch_task = tokio::spawn(async move {
            ARBITRATOR.arbitrate_and_execute_with(req_launch2, || async {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                ActionResponse {
                    success: true,
                    output: "Launch 2 finished".to_string(),
                    error: None,
                }
            }).await
        });

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        // Focus (Low priority) comes in. It should queue and wait.
        let focus_task = tokio::spawn(async move {
            ARBITRATOR.arbitrate_and_execute_with(req_focus, || async {
                ActionResponse {
                    success: true,
                    output: "Focus finished".to_string(),
                    error: None,
                }
            }).await
        });

        let res_launch = launch_task.await.unwrap();
        let res_focus = focus_task.await.unwrap();

        assert!(res_launch.success);
        assert_eq!(res_launch.output, "Launch 2 finished");
        assert!(res_focus.success);
        assert_eq!(res_focus.output, "Focus finished");
    }
}
