use crate::database::Database;
use crate::voice::personality_engine::PersonalityMode;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoicePreferences {
    pub preferred_voice: String,
    pub personality_mode: PersonalityMode,
    pub speaking_speed: f32,
    pub volume: f32,
    pub emotional_responsiveness: f32,
}

impl VoicePreferences {
    pub fn default_preferences() -> Self {
        Self {
            preferred_voice: "en_US-lessac-medium.onnx".to_string(),
            personality_mode: PersonalityMode::Assistant,
            speaking_speed: 1.0,
            volume: 1.0,
            emotional_responsiveness: 0.8,
        }
    }

    /// Loads voice preferences from settings database.
    pub fn load_from_db(db: &Database) -> Self {
        let preferred_voice = db.get_setting("voice_preferred_voice")
            .unwrap_or_default()
            .unwrap_or_else(|| "en_US-lessac-medium.onnx".to_string());

        let personality_mode = PersonalityMode::load_from_db(db);

        let speaking_speed = db.get_setting("voice_speaking_speed")
            .unwrap_or_default()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);

        let volume = db.get_setting("voice_volume")
            .unwrap_or_default()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);

        let emotional_responsiveness = db.get_setting("voice_emotional_responsiveness")
            .unwrap_or_default()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.8);

        Self {
            preferred_voice,
            personality_mode,
            speaking_speed,
            volume,
            emotional_responsiveness,
        }
    }

    /// Saves all active preferences to the database.
    pub fn save_to_db(&self, db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.set_setting("voice_preferred_voice", &self.preferred_voice)?;
        self.personality_mode.save_to_db(db)?;
        db.set_setting("voice_speaking_speed", &self.speaking_speed.to_string())?;
        db.set_setting("voice_volume", &self.volume.to_string())?;
        db.set_setting("voice_emotional_responsiveness", &self.emotional_responsiveness.to_string())?;
        Ok(())
    }

    /// Detects voice preferences adjustment commands in the user transcript.
    /// Updates the preferences state and returns a confirmation message if a change occurred.
    pub fn handle_voice_commands(&mut self, transcript: &str, db: &Database) -> Option<String> {
        let text = transcript.to_lowercase();
        let mut updated = false;
        let mut confirmation = String::new();

        // 1. Check personality mode switches
        if let Some(new_mode) = PersonalityMode::detect_switch_command(transcript) {
            self.personality_mode = new_mode;
            updated = true;
            confirmation.push_str(&format!("Personality switched to {}. ", new_mode.as_str()));
        }

        // 2. Check speaking speed commands
        if text.contains("speak slower") || text.contains("talk slower") || text.contains("slower speed") || text.contains("speak slow") {
            self.speaking_speed = (self.speaking_speed - 0.15).max(0.6);
            updated = true;
            confirmation.push_str(&format!("Setting speed to {:.1} multiplier. ", self.speaking_speed));
        } else if text.contains("speak faster") || text.contains("talk faster") || text.contains("faster speed") || text.contains("speak fast") {
            self.speaking_speed = (self.speaking_speed + 0.15).min(1.8);
            updated = true;
            confirmation.push_str(&format!("Setting speed to {:.1} multiplier. ", self.speaking_speed));
        } else if text.contains("normal speed") || text.contains("default speed") {
            self.speaking_speed = 1.0;
            updated = true;
            confirmation.push_str("Resetting speed to normal. ");
        }

        // 3. Check specific voice overrides
        if text.contains("use bella voice") || text.contains("bella voice") {
            self.preferred_voice = "af_bella.onnx".to_string();
            updated = true;
            confirmation.push_str("Using Bella voice. ");
        } else if text.contains("use adam voice") || text.contains("adam voice") {
            self.preferred_voice = "am_adam.onnx".to_string();
            updated = true;
            confirmation.push_str("Using Adam voice. ");
        } else if text.contains("use sarah voice") || text.contains("sarah voice") || text.contains("assistant voice") {
            self.preferred_voice = "af_sarah.onnx".to_string();
            updated = true;
            confirmation.push_str("Using Sarah voice. ");
        } else if text.contains("use emma voice") || text.contains("emma voice") || text.contains("professional voice") {
            self.preferred_voice = "bf_emma.onnx".to_string();
            updated = true;
            confirmation.push_str("Using Emma voice. ");
        }

        // 4. Save and return confirmation if updated
        if updated {
            let _ = self.save_to_db(db);
            Some(confirmation.trim().to_string())
        } else {
            None
        }
    }
}
