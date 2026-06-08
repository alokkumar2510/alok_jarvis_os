use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use crate::database::Database;
use crate::planner::Step;

pub struct AutomationAgent;

impl AutomationAgent {
    pub async fn run(goal: &str, _db: Arc<Mutex<Database>>, app: AppHandle) -> Result<String, String> {
        println!("AutomationAgent: Starting folder organization for goal: '{}'", goal);

        // Decompose the goal
        let mut steps = vec![
            Step {
                name: "Locate and Scan Folder".to_string(),
                description: "Scan the Downloads directory for unorganized files".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Create Category Folders".to_string(),
                description: "Create Documents, Images, Archives, Installers, and Code subdirectories".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Sort and Relocate Files".to_string(),
                description: "Move files into their respective category directories".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
            Step {
                name: "Verify Structure".to_string(),
                description: "Confirm that files were successfully relocated".to_string(),
                status: "Pending".to_string(),
                exec_cmd: None,
                verify_cmd: None,
            },
        ];

        Self::update_ui(&app, goal, &steps);

        // Step 1: Locate and Scan Folder
        steps[0].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        // Resolve Downloads path
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let target_dir = if goal.to_lowercase().contains("downloads") {
            PathBuf::from(user_profile).join("Downloads")
        } else {
            // Default to a temporary work folder or user's home folder if not specified
            PathBuf::from("e:\\ALOK PC\\Downloads")
        };

        if !target_dir.exists() {
            // Failure recovery: create directory if it doesn't exist
            let _ = fs::create_dir_all(&target_dir);
        }

        // Scan files
        let mut files_to_organize = Vec::new();
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files_to_organize.push(path);
                }
            }
        }

        println!("AutomationAgent: Scanned {} file(s) in {}", files_to_organize.len(), target_dir.display());
        steps[0].status = "Completed".to_string();
        Self::update_ui(&app, goal, &steps);

        // Step 2: Create Category Folders
        steps[1].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let categories = ["Documents", "Images", "Archives", "Installers", "Code", "Other"];
        let mut folder_paths = std::collections::HashMap::new();

        for cat in &categories {
            let cat_dir = target_dir.join(cat);
            if !cat_dir.exists() {
                let _ = fs::create_dir_all(&cat_dir);
            }
            folder_paths.insert(cat.to_string(), cat_dir);
        }

        steps[1].status = "Completed".to_string();
        Self::update_ui(&app, goal, &steps);

        // Step 3: Sort and Relocate Files
        steps[2].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        let mut moved_count = 0;
        let mut skipped_count = 0;

        for file_path in &files_to_organize {
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
                let category = match ext.as_str() {
                    "pdf" | "doc" | "docx" | "txt" | "xlsx" | "pptx" | "csv" | "md" => "Documents",
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => "Images",
                    "zip" | "rar" | "7z" | "tar" | "gz" => "Archives",
                    "exe" | "msi" => "Installers",
                    "rs" | "py" | "js" | "html" | "css" | "json" | "toml" | "cpp" | "h" | "go" | "java" => "Code",
                    _ => "Other",
                };

                if let Some(dest_dir) = folder_paths.get(category) {
                    if let Some(filename) = file_path.file_name() {
                        let dest_path = dest_dir.join(filename);
                        
                        // Move file. Handle failure recovery: skip locked files and log.
                        if fs::rename(file_path, &dest_path).is_ok() {
                            moved_count += 1;
                        } else {
                            // Try copy-and-delete fallback
                            if fs::copy(file_path, &dest_path).is_ok() {
                                let _ = fs::remove_file(file_path);
                                moved_count += 1;
                            } else {
                                println!("AutomationAgent: Failed to move locked file: {}", file_path.display());
                                skipped_count += 1;
                            }
                        }
                    }
                }
            } else {
                // Move file without extension to "Other"
                if let Some(dest_dir) = folder_paths.get("Other") {
                    if let Some(filename) = file_path.file_name() {
                        let dest_path = dest_dir.join(filename);
                        if fs::rename(file_path, &dest_path).is_ok() {
                            moved_count += 1;
                        } else {
                            let _ = fs::copy(file_path, &dest_path);
                            let _ = fs::remove_file(file_path);
                            moved_count += 1;
                        }
                    }
                }
            }
        }

        steps[2].status = "Completed".to_string();
        steps[2].description = format!("Moved {} files (skipped {} locked/in-use)", moved_count, skipped_count);
        Self::update_ui(&app, goal, &steps);

        // Step 4: Verify Structure
        steps[3].status = "Running".to_string();
        Self::update_ui(&app, goal, &steps);

        // Verify that root directory contains fewer unorganized files
        let mut remaining_unorganized = 0;
        if let Ok(entries) = fs::read_dir(&target_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    remaining_unorganized += 1;
                }
            }
        }

        if remaining_unorganized <= skipped_count {
            steps[3].status = "Completed".to_string();
            Self::update_ui(&app, goal, &steps);
            Ok(format!("Folder organized successfully! Sorted {} files in {}.", moved_count, target_dir.display()))
        } else {
            steps[3].status = "Failed".to_string();
            Self::update_ui(&app, goal, &steps);
            Err(format!("Verification failed: {} unorganized files remaining in root.", remaining_unorganized))
        }
    }

    fn update_ui(app: &AppHandle, goal: &str, steps: &[Step]) {
        let _ = app.emit("planner-update", serde_json::json!({
            "goal": goal.to_string(),
            "steps": steps.to_vec(),
        }));
    }
}
