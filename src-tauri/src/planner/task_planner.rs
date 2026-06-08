use crate::database::Database;
use crate::planner::{Plan, Step};
use crate::intelligence::groq::GroqClient;

pub struct TaskPlanner;

impl TaskPlanner {
    pub fn plan_goal(db: &Database, goal: &str) -> Plan {
        println!("TaskPlanner: Creating plan for goal: '{}'", goal);

        // 1. Try Groq dynamic planning
        if let Some(steps) = Self::plan_via_groq(db, goal) {
            return Plan {
                goal: goal.to_string(),
                steps,
            };
        }

        // 2. Fallback to deterministic rules if Groq fails or is unconfigured
        println!("TaskPlanner: Falling back to rule-based planning.");
        let goal_lower = goal.to_lowercase();
        let steps = if goal_lower.contains("flutter") {
            vec![
                Step {
                    name: "Check Flutter SDK".to_string(),
                    description: "Verify if Flutter is installed on PATH".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("flutter --version".to_string()),
                },
                Step {
                    name: "Check Android SDK".to_string(),
                    description: "Verify if Android Studio and SDK are present".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("adb --version".to_string()),
                },
                Step {
                    name: "Check VS Code".to_string(),
                    description: "Verify if VS Code is installed".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("code --version".to_string()),
                },
                Step {
                    name: "Verify Setup".to_string(),
                    description: "Run flutter doctor to confirm environment integrity".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("flutter doctor".to_string()),
                },
            ]
        } else if goal_lower.contains("python") {
            vec![
                Step {
                    name: "Check Python Installation".to_string(),
                    description: "Verify Python 3 is installed on PATH".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("python --version".to_string()),
                },
                Step {
                    name: "Check Pip Package Manager".to_string(),
                    description: "Verify pip is installed".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("pip --version".to_string()),
                },
                Step {
                    name: "Verify Virtual Environment Package".to_string(),
                    description: "Ensure python -m venv is available".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("python -c \"import venv\"".to_string()),
                },
            ]
        } else if goal_lower.contains("node") || goal_lower.contains("npm") {
            vec![
                Step {
                    name: "Check Node.js Installation".to_string(),
                    description: "Verify Node.js is installed on PATH".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("node --version".to_string()),
                },
                Step {
                    name: "Check Npm Package Manager".to_string(),
                    description: "Verify npm package manager is present".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("npm --version".to_string()),
                },
                Step {
                    name: "Verify Npx Runner".to_string(),
                    description: "Check if npx command runner is available".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("npx --version".to_string()),
                },
            ]
        } else if goal_lower.contains("rust") {
            vec![
                Step {
                    name: "Check Rustc Compiler".to_string(),
                    description: "Verify Rust compiler is present".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("rustc --version".to_string()),
                },
                Step {
                    name: "Check Cargo Package Manager".to_string(),
                    description: "Verify Cargo package builder is present".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("cargo --version".to_string()),
                },
                Step {
                    name: "Check Rustup manager".to_string(),
                    description: "Verify rustup toolchain manager is available".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("rustup --version".to_string()),
                },
            ]
        } else if goal_lower.contains("docker") {
            vec![
                Step {
                    name: "Check Docker Engine".to_string(),
                    description: "Verify Docker CLI installation".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("docker --version".to_string()),
                },
                Step {
                    name: "Check Docker Compose".to_string(),
                    description: "Verify Docker Compose plugin".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("docker-compose --version".to_string()),
                },
            ]
        } else if goal_lower.contains("git") {
            vec![
                Step {
                    name: "Check Git Installation".to_string(),
                    description: "Verify git version on PATH".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("git --version".to_string()),
                },
                Step {
                    name: "Verify Git Config Username".to_string(),
                    description: "Ensure git user.name is set".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("git config user.name".to_string()),
                },
                Step {
                    name: "Verify Git Config Email".to_string(),
                    description: "Ensure git user.email is set".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("git config user.email".to_string()),
                },
            ]
        } else if goal_lower.contains("tauri") {
            vec![
                Step {
                    name: "Check Node.js Installation".to_string(),
                    description: "Verify Node.js toolchain".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("node --version".to_string()),
                },
                Step {
                    name: "Check Rustc Compiler".to_string(),
                    description: "Verify Rust compiler".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("rustc --version".to_string()),
                },
                Step {
                    name: "Check Cargo-Tauri CLI".to_string(),
                    description: "Verify cargo-tauri CLI is installed".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: Some("cargo tauri --version".to_string()),
                },
            ]
        } else {
            // Default generic plan
            vec![
                Step {
                    name: "Analyze Prerequisites".to_string(),
                    description: "Check system requirements and dependencies".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: None,
                },
                Step {
                    name: "Execute Goal Command".to_string(),
                    description: "Run the target action".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: None,
                },
                Step {
                    name: "Verify Outcome".to_string(),
                    description: "Perform validation tests".to_string(),
                    status: "Pending".to_string(),
                    exec_cmd: None,
                    verify_cmd: None,
                },
            ]
        };

        Plan {
            goal: goal.to_string(),
            steps,
        }
    }

    fn plan_via_groq(db: &Database, goal: &str) -> Option<Vec<Step>> {
        let api_key = db.get_setting("groq_api_key").ok().flatten();
        let model = db.get_setting("groq_model").ok().flatten().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());

        if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            return None;
        }

        let client = GroqClient::new(api_key, &model);
        
        let system_prompt = format!(
            "You are ALOK OS Task Planner. Break down the user's goal into a list of step-by-step check and verification tasks. The goal is: '{}'.\n\
            Respond ONLY with a valid JSON array of objects (no markdown, no quotes surrounding the JSON blocks). Each object MUST have the following structure:\n\
            {{\n\
              \"name\": \"Name of the step\",\n\
              \"description\": \"Description of what the step does\",\n\
              \"status\": \"Pending\",\n\
              \"exec_cmd\": \"string command or null\",\n\
              \"verify_cmd\": \"string verification command or null\"\n\
            }}\n\
            Keep the plan short, between 3 to 6 steps maximum. Do not include verbose formatting.",
            goal
        );

        match client.query(&system_prompt) {
            Ok(response) => {
                // Sanitize response to isolate the JSON array
                let cleaned = response.trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();

                match serde_json::from_str::<Vec<Step>>(cleaned) {
                    Ok(steps) => {
                        println!("TaskPlanner: Successfully generated dynamic plan containing {} steps.", steps.len());
                        Some(steps)
                    }
                    Err(e) => {
                        eprintln!("TaskPlanner: Failed to deserialize Groq response JSON: {}. Response: {}", e, cleaned);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("TaskPlanner: Groq planning query failed: {:?}", e);
                None
            }
        }
    }
}
