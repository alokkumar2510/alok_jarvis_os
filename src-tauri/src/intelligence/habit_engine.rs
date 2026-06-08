use crate::AppState;
use tauri::{AppHandle, Manager};
use std::time::Duration;
use crate::environment_awareness;

pub fn start_tracker(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_process = String::new();
        let mut last_title = String::new();
        let mut last_log_time = tokio::time::Instant::now();
        
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            
            let context = environment_awareness::capture();
            let process = context.active_window.process_name.clone();
            let title = context.active_window.title.clone();
            
            // Skip the JARVIS app window itself to avoid logging our own interactive UI events
            if process == "alok_jarvis_os.exe" || process == "unknown" {
                continue;
            }

            let state = match app.try_state::<AppState>() {
                Some(s) => s,
                None => continue,
            };

            let should_log = {
                let has_changed = process != last_process || title != last_title;
                let is_timeout = last_log_time.elapsed() >= Duration::from_secs(60);
                has_changed || is_timeout
            };

            if should_log {
                if let Ok(db) = state.db.lock() {
                    let url_opt = context.browser.url.as_deref().filter(|s| !s.is_empty());
                    let folder_opt = context.file.active_folder.as_deref().filter(|s| !s.is_empty());
                    let file_opt = context.file.active_file.as_deref().filter(|s| !s.is_empty());
                    
                    if let Err(e) = db.log_behavior(&process, &title, url_opt, folder_opt, file_opt) {
                        eprintln!("HabitEngine: Failed to log behavior: {:?}", e);
                    } else {
                        last_process = process;
                        last_title = title;
                        last_log_time = tokio::time::Instant::now();
                    }
                }
            }
        }
    });
}
