use serde::{Serialize, Deserialize};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub window_title: String,
    pub process_name: String,
    pub browser_url: Option<String>,
    pub active_file: Option<String>,
    pub active_folder: Option<String>,
    pub clipboard_text: Option<String>,
}

pub fn get_world_state(state: &AppState) -> WorldState {
    let mut active_file = None;
    let mut active_folder = None;

    if let Ok(db) = state.db.lock() {
        if let Ok(Some(sess)) = crate::intelligence::session_manager::get_last_session(&db) {
            active_folder = Some(sess.project_path);
            if !sess.open_files.is_empty() {
                active_file = serde_json::from_str::<Vec<String>>(&sess.open_files)
                    .ok()
                    .and_then(|files| files.first().cloned());
            }
        }
    }

    if let Some(ctx) = crate::screen_context_engine::ScreenContextEngine::get_current_context() {
        WorldState {
            window_title: ctx.window_title,
            process_name: ctx.process_name,
            browser_url: ctx.browser_url,
            active_file: active_file.or(ctx.selected_text.clone()),
            active_folder,
            clipboard_text: ctx.selected_text,
        }
    } else {
        WorldState {
            window_title: "unknown".to_string(),
            process_name: "unknown.exe".to_string(),
            browser_url: None,
            active_file,
            active_folder,
            clipboard_text: None,
        }
    }
}
