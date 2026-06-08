use crate::database::{Database, WorkspaceSession};
use crate::desktop::control;

/// Saves the workspace session checkpoint
pub fn save_session(
    db: &Database,
    name: &str,
    project_name: &str,
    project_path: &str,
    open_apps: &[String],
    open_files: &[String],
    browser_tabs: &[String],
) -> Result<(), String> {
    let open_apps_json = serde_json::to_string(open_apps).unwrap_or_else(|_| "[]".to_string());
    let open_files_json = serde_json::to_string(open_files).unwrap_or_else(|_| "[]".to_string());
    let browser_tabs_json = serde_json::to_string(browser_tabs).unwrap_or_else(|_| "[]".to_string());

    db.save_workspace_session(name, project_name, project_path, &open_apps_json, &open_files_json, &browser_tabs_json)
        .map_err(|e| e.to_string())
}

/// Retrieves the most recent workspace session
pub fn get_last_session(db: &Database) -> Result<Option<WorkspaceSession>, String> {
    db.get_last_workspace_session().map_err(|e| e.to_string())
}

/// Restores a workspace session by launching folders/files in VS Code and opening browser tabs
pub fn restore_session(session: &WorkspaceSession) -> Result<String, String> {
    println!("SessionManager: Restoring session: {}", session.session_name);

    let mut restored_actions = Vec::new();

    // 1. Launch VS Code or explorer for the project root
    if !session.project_path.is_empty() {
        let status = std::process::Command::new("cmd")
            .args(["/C", "code", &session.project_path])
            .spawn();
            
        match status {
            Ok(_) => restored_actions.push(format!("Opened VS Code for project: {}", session.project_name)),
            Err(_) => {
                // Fallback to Explorer
                let _ = std::process::Command::new("explorer")
                    .arg(&session.project_path)
                    .spawn();
                restored_actions.push(format!("Opened project folder: {}", session.project_path));
            }
        }
    }

    // 2. Open browser tabs
    if let Ok(tabs) = serde_json::from_str::<Vec<String>>(&session.browser_tabs) {
        for tab in tabs {
            if !tab.trim().is_empty() && (tab.starts_with("http://") || tab.starts_with("https://") || tab.contains('.')) {
                let _ = control::open_website(&tab);
                restored_actions.push(format!("Opened browser tab: {}", tab));
            }
        }
    }

    // 3. Open specific active files in VS Code
    if let Ok(files) = serde_json::from_str::<Vec<String>>(&session.open_files) {
        for file in files {
            if !file.trim().is_empty() {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "code", &file])
                    .spawn();
                restored_actions.push(format!("Opened file: {}", file));
            }
        }
    }

    if restored_actions.is_empty() {
        Ok("Nothing to restore from the last session.".to_string())
    } else {
        Ok(format!("Restored session: {}", restored_actions.join(", ")))
    }
}
