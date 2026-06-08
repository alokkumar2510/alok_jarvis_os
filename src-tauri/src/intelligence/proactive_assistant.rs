use crate::AppState;
use tauri::{AppHandle, Manager, Emitter};
use std::time::Duration;
use crate::voice::tts;
use std::collections::{HashSet, HashMap};
use crate::environment_awareness;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use std::path::Path;
use std::fs;
use std::process::Command;

fn check_battery() -> Option<(u8, bool)> {
    unsafe {
        let mut status = SYSTEM_POWER_STATUS::default();
        if GetSystemPowerStatus(&mut status).is_ok() {
            // ACLineStatus: 1 = Online (Charging / plugged in), 0 = Offline
            let online = status.ACLineStatus == 1;
            let percent = status.BatteryLifePercent;
            // 255 represents unknown
            if percent <= 100 {
                return Some((percent, online));
            }
        }
    }
    None
}

pub fn check_battery_health() -> Option<(f64, f64)> {
    let temp_dir = std::env::temp_dir();
    let report_path = temp_dir.join("alok_jarvis_battery_report.html");
    let report_path_str = report_path.to_string_lossy().to_string();

    let output = Command::new("powercfg")
        .args(["/batteryreport", "/output", &report_path_str])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = fs::read_to_string(&report_path).ok()?;
    let _ = fs::remove_file(&report_path);

    // Remove newlines and compress spaces
    let cleaned: String = content.lines()
        .map(|line| line.trim())
        .collect::<Vec<&str>>()
        .join(" ");

    // Parse DESIGN CAPACITY
    let design_idx = cleaned.find("DESIGN CAPACITY")?;
    let design_sub = &cleaned[design_idx..cleaned.len().min(design_idx + 200)];
    let design_val = parse_capacity_from_substring(design_sub)?;

    // Parse FULL CHARGE CAPACITY
    let full_idx = cleaned.find("FULL CHARGE CAPACITY")?;
    let full_sub = &cleaned[full_idx..cleaned.len().min(full_idx + 200)];
    let full_val = parse_capacity_from_substring(full_sub)?;

    Some((design_val, full_val))
}

fn parse_capacity_from_substring(sub: &str) -> Option<f64> {
    let mut num_str = String::new();
    let mut found_digit = false;
    
    for c in sub.chars() {
        if c.is_ascii_digit() {
            found_digit = true;
            num_str.push(c);
        } else if found_digit {
            if c == ',' || c == '.' {
                continue;
            } else {
                break;
            }
        }
    }
    
    if num_str.is_empty() {
        None
    } else {
        num_str.parse::<f64>().ok()
    }
}

fn check_git_commits(active_folder: &str) -> Option<u64> {
    let folder_path = Path::new(active_folder);
    if !folder_path.join(".git").exists() {
        return None;
    }

    // Run git log -1 --format=%ct to get timestamp of last commit
    let output = Command::new("git")
        .args(["-C", active_folder, "log", "-1", "--format=%ct"])
        .output()
        .ok()?;

    if output.status.success() {
        let ts_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Ok(last_commit_time) = ts_str.parse::<u64>() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            if now > last_commit_time {
                let elapsed_secs = now - last_commit_time;
                let elapsed_days = elapsed_secs / (24 * 3600);
                return Some(elapsed_days);
            }
        }
    }
    None
}

fn check_downloads_count() -> Option<usize> {
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    if user_profile.is_empty() {
        return None;
    }

    let downloads_path = Path::new(&user_profile).join("Downloads");
    if !downloads_path.exists() {
        return None;
    }

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&downloads_path) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                count += 1;
            }
        }
    }
    Some(count)
}

fn check_disk_space() -> Option<f64> {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetDiskFreeSpaceExW(
                lpdirectoryname: *const u16,
                lpfreebytesavailabletocaller: *mut u64,
                lptotalnumberofbytes: *mut u64,
                lptotalnumberoffreebytes: *mut u64,
            ) -> i32;
        }
        let mut free_bytes = 0u64;
        let mut total_bytes = 0u64;
        let mut total_free = 0u64;
        let drive = [0x0043, 0x003a, 0x005c, 0]; // "C:\"
        if GetDiskFreeSpaceExW(drive.as_ptr(), &mut free_bytes, &mut total_bytes, &mut total_free) != 0 {
            let gb = (total_free as f64) / (1024.0 * 1024.0 * 1024.0);
            return Some((gb * 10.0).round() / 10.0);
        }
    }
    None
}

fn check_system_memory() -> Option<f64> {
    unsafe {
        #[repr(C)]
        struct MEMORYSTATUSEX {
            dwLength: u32,
            dwMemoryLoad: u32,
            ullTotalPhys: u64,
            ullAvailPhys: u64,
            ullTotalPageFile: u64,
            ullAvailPageFile: u64,
            ullTotalVirtual: u64,
            ullAvailPageFile_v: u64, // unused placeholder
            ullAvailExtendedVirtual: u64,
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalMemoryStatusEx(lpmemoryStatus: *mut MEMORYSTATUSEX) -> i32;
        }
        let mut mem_info = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailPageFile_v: 0,
            ullAvailExtendedVirtual: 0,
        };
        if GlobalMemoryStatusEx(&mut mem_info) != 0 {
            return Some(mem_info.dwMemoryLoad as f64);
        }
    }
    None
}

fn check_git_uncommitted(active_folder: &str) -> Option<usize> {
    let folder_path = Path::new(active_folder);
    if !folder_path.join(".git").exists() {
        return None;
    }

    let output = Command::new("git")
        .args(["-C", active_folder, "status", "--porcelain"])
        .output()
        .ok()?;

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        let count = out_str.lines().filter(|l| !l.trim().is_empty()).count();
        return Some(count);
    }
    None
}

pub fn start_proactive_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut alerted_meetings = HashSet::new();
        let mut battery_alerted = false;
        let mut last_suggested_hour = HashMap::new(); // Key: target, Value: hour
        
        let mut loop_counter = 0;
        
        // Prevent repeated alerts
        let mut last_alerted_git_folder = String::new();
        let mut last_alerted_git_uncommitted_folder = String::new();
        let mut last_alerted_downloads = false;
        let mut last_alerted_battery_health = false;
        let mut last_alerted_disk_space = false;
        let mut last_alerted_memory = false;
        let mut last_alerted_tabs = false;
        let mut last_alerted_duplicates = false;
        let mut last_alerted_failed_goal = String::new();

        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            loop_counter += 1;
            
            let state = match app.try_state::<AppState>() {
                Some(s) => s,
                None => continue,
            };

            // ==========================================================
            // FAST CHECKS (Every 30 seconds)
            // ==========================================================

            // 1. Battery Charge Level Check
            if let Some((percent, online)) = check_battery() {
                if percent <= 15 && !online {
                    if !battery_alerted {
                        let msg = format!("Your battery is low at {} percent. Please plug in your charger.", percent);
                        let _ = tts::speak(&msg);
                        let _ = app.emit("proactive-suggestion", serde_json::json!({
                            "type": "warning",
                            "title": "Low Battery Alert",
                            "message": msg,
                            "action": "none",
                            "target": ""
                        }));
                        battery_alerted = true;
                    }
                } else if online || percent > 20 {
                    battery_alerted = false; // Reset trigger
                }
            }

            // 2. Upcoming Calendar Meetings
            if let Ok(db) = state.db.lock() {
                if let Ok(meetings) = db.get_upcoming_meetings(15) {
                    for (id, title, start_time, _desc) in meetings {
                        if !alerted_meetings.contains(&id) {
                            let msg = format!("You have a meeting starting in 15 minutes: '{}'", title);
                            let _ = tts::speak(&msg);
                            let _ = app.emit("proactive-suggestion", serde_json::json!({
                                "type": "meeting",
                                "title": "Upcoming Meeting",
                                "message": msg,
                                "action": "none",
                                "target": start_time
                            }));
                            alerted_meetings.insert(id);
                        }
                    }
                }
            }

            // 3. Habit Predictions
            let current_hour = {
                if let Ok(db) = state.db.lock() {
                    db.get_current_local_hour().unwrap_or(-1)
                } else {
                    -1
                }
            };

            if current_hour >= 0 {
                let predictions = {
                    if let Ok(db) = state.db.lock() {
                        crate::intelligence::behavior_model::predict_next_action(&db, current_hour).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                };

                if let Some(pred) = predictions.first() {
                    if pred.confidence >= 0.4 {
                        let already_suggested = last_suggested_hour.get(&pred.target).copied() == Some(current_hour);
                        if !already_suggested {
                            let active_context = environment_awareness::capture();
                            let is_currently_active = if pred.habit_type == "app_launch" {
                                active_context.active_window.process_name.to_lowercase().contains(&pred.target.to_lowercase())
                            } else {
                                active_context.browser.url.as_ref().map_or(false, |u| u.to_lowercase().contains(&pred.target.to_lowercase()))
                            };

                            if !is_currently_active {
                                let display_name = if pred.target.starts_with("search:") {
                                    format!("search for '{}'", &pred.target[7..])
                                } else {
                                    pred.target.clone()
                                };
                                let msg = format!("I notice you usually open {} at this time. Should I do that for you?", display_name);
                                let _ = tts::speak(&msg);
                                let _ = app.emit("proactive-suggestion", serde_json::json!({
                                    "type": "habit",
                                    "title": "Daily Routine Suggestion",
                                    "message": msg,
                                    "action": if pred.habit_type == "app_launch" { "launch_app" } else { "browser_visit" },
                                    "target": pred.target.clone()
                                }));
                                last_suggested_hour.insert(pred.target.clone(), current_hour);
                            }
                        }
                    }
                }
            }

            // ==========================================================
            // SLOW AUTONOMOUS CHECKS (Every 15 minutes / 30 iterations)
            // ==========================================================
            if loop_counter % 30 == 1 {
                let active_context = environment_awareness::capture();

                // A. Git Commit Check
                if let Some(ref active_folder) = active_context.file.active_folder {
                    if active_folder != &last_alerted_git_folder {
                        if let Some(days) = check_git_commits(active_folder) {
                            if days >= 3 {
                                let folder_name = Path::new(active_folder)
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("active project");
                                let msg = format!("You have not committed code in the '{}' project for {} days. Consider committing your changes.", folder_name, days);
                                let _ = tts::speak(&msg);
                                let _ = app.emit("proactive-suggestion", serde_json::json!({
                                    "type": "warning",
                                    "title": "Git Commit Warning",
                                    "message": msg,
                                    "action": "none",
                                    "target": ""
                                }));
                                last_alerted_git_folder = active_folder.clone();
                            }
                        }
                    }
                }

                // B. Downloads Folder File Count Check
                if let Some(count) = check_downloads_count() {
                    if count >= 500 {
                        if !last_alerted_downloads {
                            let msg = format!("Your Downloads folder has {} files. Consider running folder organization to clean it up.", count);
                            let _ = tts::speak(&msg);
                            let _ = app.emit("proactive-suggestion", serde_json::json!({
                                "type": "suggestion",
                                "title": "Downloads Folder Overcrowded",
                                "message": msg,
                                "action": "run_intent",
                                "target": "organize downloads folder"
                            }));
                            last_alerted_downloads = true;
                        }
                    } else {
                        last_alerted_downloads = false; // Reset trigger if count falls
                    }
                }

                // C. Battery Health Check
                if let Some((design, full)) = check_battery_health() {
                    if design > 0.0 {
                        let health_pct = (full / design) * 100.0;
                        if health_pct <= 85.0 {
                            if !last_alerted_battery_health {
                                let msg = format!("Your battery health is decreasing (currently at {:.1}% of design capacity). Consider checking system diagnostics.", health_pct);
                                let _ = tts::speak(&msg);
                                let _ = app.emit("proactive-suggestion", serde_json::json!({
                                    "type": "warning",
                                    "title": "Battery Health Degradation",
                                    "message": msg,
                                    "action": "none",
                                    "target": ""
                                }));
                                last_alerted_battery_health = true;
                            }
                        } else {
                            last_alerted_battery_health = false; // Reset if healthy
                        }
                    }
                }

                // D. Low System Disk Space Check
                if let Some(free_gb) = check_disk_space() {
                    if free_gb <= 15.0 {
                        if !last_alerted_disk_space {
                            let msg = format!("Your C: drive has only {:.1} GB of free space left. Consider freeing up some disk space.", free_gb);
                            let _ = tts::speak(&msg);
                            let _ = app.emit("proactive-suggestion", serde_json::json!({
                                "type": "warning",
                                "title": "Low Disk Space",
                                "message": msg,
                                "action": "none",
                                "target": ""
                            }));
                            last_alerted_disk_space = true;
                        }
                    } else {
                        last_alerted_disk_space = false;
                    }
                }

                // E. High System Memory Usage Check
                if let Some(mem_pct) = check_system_memory() {
                    if mem_pct >= 90.0 {
                        if !last_alerted_memory {
                            let msg = format!("System memory usage is very high at {:.1}%. Consider closing inactive applications.", mem_pct);
                            let _ = tts::speak(&msg);
                            let _ = app.emit("proactive-suggestion", serde_json::json!({
                                "type": "warning",
                                "title": "High Memory Usage",
                                "message": msg,
                                "action": "none",
                                "target": ""
                            }));
                            last_alerted_memory = true;
                        }
                    } else {
                        last_alerted_memory = false;
                    }
                }

                // F. Git Uncommitted Changes Check
                if let Some(ref active_folder) = active_context.file.active_folder {
                    if active_folder != &last_alerted_git_uncommitted_folder {
                        if let Some(changes_count) = check_git_uncommitted(active_folder) {
                            if changes_count >= 10 {
                                let folder_name = Path::new(active_folder)
                                    .file_name()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("active project");
                                let msg = format!("You have {} uncommitted files in the '{}' project. Consider committing your changes.", changes_count, folder_name);
                                let _ = tts::speak(&msg);
                                let _ = app.emit("proactive-suggestion", serde_json::json!({
                                    "type": "suggestion",
                                    "title": "Uncommitted Git Changes",
                                    "message": msg,
                                    "action": "none",
                                    "target": ""
                                }));
                                last_alerted_git_uncommitted_folder = active_folder.clone();
                            }
                        }
                    }
                }

                // G. Browser Tab Overload & Duplicate Tabs Check
                let open_tabs = crate::environment_awareness::browser_context_engine::BrowserContextEngine::get_open_tabs();
                let tabs_count = open_tabs.len();
                
                // Tab overload check
                if tabs_count >= 25 {
                    if !last_alerted_tabs {
                        let msg = format!("You have {} open browser tabs. Consider closing unused tabs to save system memory.", tabs_count);
                        let _ = tts::speak(&msg);
                        let _ = app.emit("proactive-suggestion", serde_json::json!({
                            "type": "warning",
                            "title": "Browser Tab Overload",
                            "message": msg,
                            "action": "none",
                            "target": ""
                        }));
                        last_alerted_tabs = true;
                    }
                } else {
                    last_alerted_tabs = false;
                }

                // Duplicate tabs check
                let mut seen_titles = HashSet::new();
                let mut duplicates = Vec::new();
                for tab in &open_tabs {
                    if !tab.title.trim().is_empty() {
                        if seen_titles.contains(&tab.title) {
                            duplicates.push(tab.title.clone());
                        } else {
                            seen_titles.insert(tab.title.clone());
                        }
                    }
                }

                if !duplicates.is_empty() {
                    if !last_alerted_duplicates {
                        let sample = duplicates.first().unwrap();
                        let display_title = if sample.len() > 30 {
                            format!("{}...", &sample[..30])
                        } else {
                            sample.clone()
                        };
                        let msg = format!(
                            "You have {} duplicate browser tabs open (e.g. '{}'). Clean them up to organize your workspace?",
                            duplicates.len(),
                            display_title
                        );
                        let _ = tts::speak(&msg);
                        let _ = app.emit("proactive-suggestion", serde_json::json!({
                            "type": "opportunity",
                            "title": "Duplicate Tabs Detected",
                            "message": msg,
                            "action": "run_intent",
                            "target": "close duplicate tabs"
                        }));
                        last_alerted_duplicates = true;
                    }
                } else {
                    last_alerted_duplicates = false;
                }

                // H. Active Goal Failed Task Check
                if let Some(plan) = state.planner.get_active_plan() {
                    if let Some(failed_step) = plan.steps.iter().find(|step| step.status == "Failed") {
                        let alert_key = format!("{}:{}", plan.goal, failed_step.name);
                        if alert_key != last_alerted_failed_goal {
                            let msg = format!(
                                "Step '{}' of your goal '{}' has failed. Let me know if you would like me to help resolve this.",
                                failed_step.name,
                                plan.goal
                            );
                            let _ = tts::speak(&msg);
                            let _ = app.emit("proactive-suggestion", serde_json::json!({
                                "type": "warning",
                                "title": "Goal Execution Failed",
                                "message": msg,
                                "action": "none",
                                "target": ""
                            }));
                            last_alerted_failed_goal = alert_key;
                        }
                    }
                }
            }
        }
    });
}
