use serde::{Serialize, Deserialize};
use crate::database::Database;
use cpal::traits::HostTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsReport {
    pub health_score: u8,
    pub voice_subsystem: String,
    pub wakeword_subsystem: String,
    pub memory_subsystem: String,
    pub automation_subsystem: String,
    pub plugin_subsystem: String,
    pub failures: Vec<String>,
}

pub fn run_diagnostics(db: &Database) -> DiagnosticsReport {
    let mut failures = Vec::new();
    let mut score = 0;

    // 1. Voice Subsystem Check
    let voice_status = match cpal::default_host().input_devices() {
        Ok(mut devices) => {
            if devices.next().is_some() {
                score += 20;
                "Online".to_string()
            } else {
                failures.push("Voice input device offline (no microphone detected)".to_string());
                "Offline".to_string()
            }
        }
        Err(e) => {
            failures.push(format!("CPAL host error: {}", e));
            "Offline".to_string()
        }
    };

    // 2. Wake Word Subsystem Check
    let wakeword_status = if db.get_setting("wakeword_sensitivity").ok().flatten().is_some() {
        score += 20;
        "Online".to_string()
    } else {
        failures.push("Wake word threshold config unavailable".to_string());
        "Offline".to_string()
    };

    // 3. Memory Subsystem Check (SQLite connection integrity)
    let memory_status = match db.get_corrections_count() {
        Ok(_) => {
            score += 20;
            "Online".to_string()
        }
        Err(e) => {
            failures.push(format!("SQLite connection query failed: {}", e));
            "Offline".to_string()
        }
    };

    // 4. Automation Subsystem Check (Win32 FFI handle check)
    let automation_status = {
        unsafe {
            let mut pt = crate::desktop::automation_runtime::POINT::default();
            if crate::desktop::automation_runtime::GetCursorPos(&mut pt) != 0 {
                score += 20;
                "Online".to_string()
            } else {
                failures.push("Win32 cursor FFI query rejected".to_string());
                "Offline".to_string()
            }
        }
    };

    // 5. Plugin Subsystem Check (Checks loaded intents/models)
    let plugin_status = if db.get_setting("groq_model").ok().flatten().is_some() {
        score += 20;
        "Online".to_string()
    } else {
        failures.push("Reasoning model config missing".to_string());
        "Offline".to_string()
    };

    DiagnosticsReport {
        health_score: score,
        voice_subsystem: voice_status,
        wakeword_subsystem: wakeword_status,
        memory_subsystem: memory_status,
        automation_subsystem: automation_status,
        plugin_subsystem: plugin_status,
        failures,
    }
}
