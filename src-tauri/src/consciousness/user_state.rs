use serde::{Serialize, Deserialize};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub user_name: String,
    pub focus_state: String,
    pub predicted_habit: Option<String>,
}

pub fn get_user_state(state: &AppState) -> UserState {
    let mut predicted_habit = None;
    
    if let Ok(db) = state.db.lock() {
        if let Ok(habits) = db.get_learned_patterns("preferred_app") {
            if let Some((app_key, app_val, _)) = habits.first() {
                predicted_habit = Some(format!("Launch {} ({})", app_val, app_key));
            }
        }
    }

    let focus_state = if let Some(ctx) = crate::screen_context_engine::ScreenContextEngine::get_current_context() {
        if ctx.process_name.contains("code") || ctx.process_name.contains("studio") {
            "Focused Work".to_string()
        } else {
            "Active".to_string()
        }
    } else {
        "Idle".to_string()
    };

    UserState {
        user_name: "Alok".to_string(),
        focus_state,
        predicted_habit,
    }
}
