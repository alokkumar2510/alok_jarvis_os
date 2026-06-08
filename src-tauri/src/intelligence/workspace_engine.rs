use crate::database::Database;
use crate::environment_awareness::EnvironmentContext;
use crate::intelligence::{project_detector, session_manager};
use std::sync::{Arc, Mutex};

pub struct WorkspaceEngine {
    db: Arc<Mutex<Database>>,
}

impl WorkspaceEngine {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self { db }
    }

    /// Captures the current EnvironmentContext and saves it if changes occur in the active project/file/url
    pub fn track_current_state(&self, env: &EnvironmentContext) {
        let active_file = env.file.active_file.as_deref();
        let active_folder = env.file.active_folder.as_deref();

        // 1. Detect active project root and name
        let (project_name, _project_type) = match project_detector::detect_project(active_file, active_folder) {
            Some((name, p_type)) => (name, p_type),
            None => return, // If no project is active, don't write checkpoints
        };

        let project_path = active_folder.unwrap_or_else(|| {
            if let Some(file) = active_file {
                std::path::Path::new(file).parent().and_then(|p| p.to_str()).unwrap_or("")
            } else {
                ""
            }
        });

        if project_path.is_empty() {
            return;
        }

        // 2. Query last session to ensure we don't save duplicate checkpoints
        let db_lock = self.db.lock().unwrap();
        if let Ok(Some(last_sess)) = session_manager::get_last_session(&db_lock) {
            if last_sess.project_path == project_path {
                let open_files: Vec<String> = serde_json::from_str(&last_sess.open_files).unwrap_or_default();
                let browser_tabs: Vec<String> = serde_json::from_str(&last_sess.browser_tabs).unwrap_or_default();

                let file_matches = active_file.map_or(true, |f| open_files.contains(&f.to_string()));
                let tab_matches = env.browser.url.as_ref().map_or(true, |url| browser_tabs.contains(url));

                if file_matches && tab_matches {
                    return; // State matches last checkpoint, skip saving
                }
            }
        }

        // 3. Assemble and save session checkpoint
        let open_apps = vec![env.active_window.process_name.clone()];
        let open_files = active_file.map_or(vec![], |f| vec![f.to_string()]);
        let browser_tabs = env.browser.url.as_ref().map_or(vec![], |url| vec![url.to_string()]);

        let _ = session_manager::save_session(
            &db_lock,
            &format!("Auto Session - {}", project_name),
            &project_name,
            project_path,
            &open_apps,
            &open_files,
            &browser_tabs,
        );
        println!("WorkspaceEngine: Saved session checkpoint for '{}'", project_name);
    }

    /// Restores the last active work session
    pub fn continue_work(&self) -> Result<String, String> {
        let db_lock = self.db.lock().unwrap();
        match session_manager::get_last_session(&db_lock)? {
            Some(session) => session_manager::restore_session(&session),
            None => Err("No previous work session found to continue.".to_string()),
        }
    }
}
