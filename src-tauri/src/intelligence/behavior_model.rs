use crate::database::Database;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHabit {
    pub habit_type: String, // "app_launch", "browser_visit"
    pub target: String,
    pub hour: i32,
    pub confidence: f64,
}

pub fn predict_next_action(db: &Database, hour: i32) -> Result<Vec<UserHabit>, Box<dyn std::error::Error + Send + Sync>> {
    let db_habits = db.get_habits_for_hour(hour)?;
    let habits = db_habits.into_iter().map(|(habit_type, target, hour, confidence)| {
        UserHabit {
            habit_type,
            target,
            hour,
            confidence,
        }
    }).collect();
    Ok(habits)
}
