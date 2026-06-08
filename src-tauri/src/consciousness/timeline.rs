use serde::{Serialize, Deserialize};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: String,
    pub event_type: String,
    pub description: String,
}

pub fn get_recent_timeline(state: &AppState) -> Vec<TimelineEntry> {
    let mut entries = Vec::new();
    
    if let Ok(db) = state.db.lock() {
        if let Ok(execs) = db.get_recent_task_executions(5) {
            for e in execs {
                entries.push(TimelineEntry {
                    timestamp: e.timestamp,
                    event_type: "Command".to_string(),
                    description: format!("Executed '{}' - success: {}", e.task_name, e.success),
                });
            }
        }
        
        if let Ok(corrs) = db.get_recent_corrections(3) {
            for (incorrect, corrected, time) in corrs {
                entries.push(TimelineEntry {
                    timestamp: time,
                    event_type: "Correction".to_string(),
                    description: format!("Correction: '{}' -> '{}'", incorrect, corrected),
                });
            }
        }
    }

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(10);
    entries
}
