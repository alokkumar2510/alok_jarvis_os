use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use crate::database::Database;
use crate::planner::{Plan, Step};
use crate::AppState;

pub struct GoalExecutionEngine;

impl GoalExecutionEngine {
    pub async fn execute_goal(goal: &str, state: &AppState) -> Result<String, String> {
        println!("GoalExecutionEngine: Planning outcome goal: '{}'", goal);
        
        let goal_lower = goal.to_lowercase();
        
        if goal_lower.contains("downloads") && (goal_lower.contains("clean") || goal_lower.contains("organize") || goal_lower.contains("clear")) {
            return Self::clean_downloads_workflow(state).await;
        }
        
        let db_guard = state.db.lock().map_err(|e| e.to_string())?;
        let plan = state.planner.generate_plan(goal, &*db_guard);
        let goal_title = plan.goal.clone();
        
        let mut executed_steps = Vec::new();
        let mut success = true;
        let mut report = format!("# Goal Execution Summary: {}\n\n", goal_title);
        
        for step in plan.steps {
            report.push_str(&format!("### Step: {}\n", step.name));
            report.push_str(&format!("* Description: {}\n", step.description));
            
            let mut step_success = true;
            let start = Instant::now();
            
            if let Some(ref cmd) = step.exec_cmd {
                report.push_str(&format!("* Command: `{}`\n", cmd));
                match crate::planner::task_executor::TaskExecutor::execute_cmd(cmd) {
                    Ok(_) => {
                        report.push_str(&format!("* Output: Success\n"));
                    }
                    Err(e) => {
                        report.push_str(&format!("* Error: {}\n", e));
                        step_success = false;
                        success = false;
                    }
                }
            }
            
            if step_success {
                let verify_cmd = step.verify_cmd.as_deref();
                let verified = crate::planner::verification_engine::VerificationEngine::verify_step(verify_cmd);
                if verified {
                    report.push_str("* Verification: PASSED\n\n");
                } else {
                    report.push_str("* Verification: FAILED\n\n");
                    step_success = false;
                    success = false;
                }
            } else {
                report.push_str("* Verification: SKIPPED (Execution Failed)\n\n");
            }
            
            executed_steps.push(step);
            if !step_success {
                break;
            }
        }
        
        report.push_str("## Final Status\n");
        if success {
            report.push_str("Outcome: **SUCCESS**\n");
        } else {
            report.push_str("Outcome: **FAILED**\n");
        }
        
        if let Ok(db) = state.db.lock() {
            let _ = db.log_self_improvement_outcome(goal, success);
            let _ = crate::intelligence::solution_database::save_solution(&db, goal, &report, "goal_execution");
        }
        
        Ok(report)
    }

    async fn clean_downloads_workflow(state: &AppState) -> Result<String, String> {
        println!("GoalExecutionEngine: Launching Downloads folder cleanup workflow.");
        
        let task_id = "clean_downloads";
        crate::control::task_manager::TASK_MANAGER.register_task(
            task_id,
            "Clean Downloads Folder",
            crate::control::priority_manager::Priority::P2,
            100, // Arbitrary steps or count
        );

        let home_dir = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| "C:\\Users\\alokk".to_string());
        let downloads_path = Path::new(&home_dir).join("Downloads");
        
        if !downloads_path.exists() {
            crate::control::task_manager::TASK_MANAGER.update_status(task_id, crate::control::task_manager::TaskStatus::Failed);
            return Err("Downloads folder path not found".to_string());
        }
        
        let mut total_scanned = 0;
        let mut duplicates_moved = 0;
        let mut space_saved_bytes = 0u64;
        let mut files_organized = 0;
        
        let mut duplicate_log = Vec::new();
        let mut organized_log = Vec::new();
        
        let dup_dir = downloads_path.join("Duplicates_Archive");
        let _ = fs::create_dir_all(&dup_dir);
        
        if let Ok(entries) = fs::read_dir(&downloads_path) {
            let mut file_sizes: std::collections::HashMap<u64, Vec<PathBuf>> = std::collections::HashMap::new();
            let entries_vec: Vec<_> = entries.flatten().collect();
            let total_entries = entries_vec.len();

            for (idx, entry) in entries_vec.into_iter().enumerate() {
                // Check pause or cancel
                if let Err(e) = crate::control::task_cancellation::pause_point(task_id).await {
                    println!("clean_downloads_workflow cancelled mid-run: {}", e);
                    crate::control::task_manager::TASK_MANAGER.update_status(task_id, crate::control::task_manager::TaskStatus::Cancelled);
                    return Err(e);
                }

                crate::control::task_manager::TASK_MANAGER.update_progress(task_id, idx + 1, total_entries);

                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if name == "desktop.ini" || name.starts_with("Duplicates_Archive") {
                        continue;
                    }
                    
                    total_scanned += 1;
                    let metadata = path.metadata().ok();
                    let size = metadata.map(|m| m.len()).unwrap_or(0);
                    
                    let is_dup = if let Some(existing_paths) = file_sizes.get(&size) {
                        let mut found_dup = false;
                        for ep in existing_paths {
                            if ep.file_name() == path.file_name() {
                                found_dup = true;
                                break;
                            }
                            let ep_name = ep.file_name().and_then(|s| s.to_str()).unwrap_or("");
                            if name.contains(" (") && name.replace(" (1)", "").replace(" (2)", "") == ep_name {
                                found_dup = true;
                                break;
                            }
                        }
                        found_dup
                    } else {
                        false
                    };
                    
                    if is_dup {
                        let target_path = dup_dir.join(name);
                        if crate::control::task_cancellation::safe_rename(task_id, &path, &target_path).await.is_ok() {
                            duplicates_moved += 1;
                            space_saved_bytes += size;
                            duplicate_log.push(name.to_string());
                        }
                    } else {
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                        let folder_name = match ext.as_str() {
                            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" => Some("Documents"),
                            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => Some("Images"),
                            "mp3" | "wav" | "ogg" | "flac" => Some("Audio"),
                            "mp4" | "mkv" | "avi" | "mov" | "wmv" => Some("Videos"),
                            "zip" | "rar" | "7z" | "tar" | "gz" => Some("Archives"),
                            "exe" | "msi" | "apk" => Some("Installers"),
                            _ => None,
                        };
                        
                        if let Some(fold) = folder_name {
                            let target_fold = downloads_path.join(fold);
                            let _ = fs::create_dir_all(&target_fold);
                            let target_path = target_fold.join(name);
                            if crate::control::task_cancellation::safe_rename(task_id, &path, &target_path).await.is_ok() {
                                files_organized += 1;
                                organized_log.push(format!("Moved {} -> {}", name, fold));
                            }
                        }
                        
                        file_sizes.entry(size).or_default().push(path);
                    }
                }
            }
        }
        
        let space_saved_mb = (space_saved_bytes as f64) / (1024.0 * 1024.0);
        
        let mut report = String::new();
        report.push_str("# Downloads Folder Clean Up Report\n\n");
        report.push_str(&format!("* **Files Scanned:** {}\n", total_scanned));
        report.push_str(&format!("* **Duplicates Moved:** {}\n", duplicates_moved));
        report.push_str(&format!("* **Space Saved:** {:.2} MB\n", space_saved_mb));
        report.push_str(&format!("* **Files Organized:** {}\n\n", files_organized));
        
        if !duplicate_log.is_empty() {
            report.push_str("### Duplicate Files Archived:\n");
            for f in duplicate_log {
                report.push_str(&format!("- {}\n", f));
            }
            report.push_str("\n");
        }
        
        if !organized_log.is_empty() {
            report.push_str("### Organized Files:\n");
            for f in organized_log {
                report.push_str(&format!("- {}\n", f));
            }
        }
        
        if let Ok(db) = state.db.lock() {
            let _ = db.log_self_improvement_outcome("Clean Downloads Folder", true);
            let _ = crate::intelligence::solution_database::save_solution(&db, "Clean Downloads Folder", &report, "downloads_cleanup");
        }

        crate::control::task_manager::TASK_MANAGER.update_status(task_id, crate::control::task_manager::TaskStatus::Completed);
        
        Ok(report)
    }
}
