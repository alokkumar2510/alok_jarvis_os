use crate::database::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PersonalityMode {
    Assistant,
    Friendly,
    Professional,
    Companion,
    Developer,
}

impl PersonalityMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PersonalityMode::Assistant => "assistant",
            PersonalityMode::Friendly => "friendly",
            PersonalityMode::Professional => "professional",
            PersonalityMode::Companion => "companion",
            PersonalityMode::Developer => "developer",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "friendly" => PersonalityMode::Friendly,
            "professional" => PersonalityMode::Professional,
            "companion" => PersonalityMode::Companion,
            "developer" => PersonalityMode::Developer,
            _ => PersonalityMode::Assistant,
        }
    }

    /// Loads the active voice personality mode from the database.
    pub fn load_from_db(db: &Database) -> Self {
        if let Ok(Some(val)) = db.get_setting("voice_personality") {
            Self::from_str(&val)
        } else {
            PersonalityMode::Assistant
        }
    }

    /// Saves the active voice personality mode to the database.
    pub fn save_to_db(&self, db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        db.set_setting("voice_personality", self.as_str())?;
        Ok(())
    }

    /// Parses the transcript to check if the user is asking to change ALOK's personality mode.
    /// Returns the new mode if matched, otherwise None.
    pub fn detect_switch_command(transcript: &str) -> Option<PersonalityMode> {
        let text = transcript.to_lowercase();
        
        // Friendly triggers
        if text.contains("friendly mode") 
           || text.contains("switch to friendly") 
           || text.contains("change to friendly") 
           || text.contains("be friendly") 
           || text.contains("personality friendly") {
            return Some(PersonalityMode::Friendly);
        }
        
        // Professional triggers
        if text.contains("professional mode") 
           || text.contains("switch to professional") 
           || text.contains("change to professional") 
           || text.contains("be professional") 
           || text.contains("personality professional") {
            return Some(PersonalityMode::Professional);
        }

        // Companion triggers
        if text.contains("companion mode") 
           || text.contains("switch to companion") 
           || text.contains("change to companion") 
           || text.contains("be my companion") 
           || text.contains("personality companion") {
            return Some(PersonalityMode::Companion);
        }

        // Developer triggers
        if text.contains("developer mode") 
           || text.contains("switch to developer") 
           || text.contains("change to developer") 
           || text.contains("be a developer") 
           || text.contains("dev mode")
           || text.contains("personality developer") {
            return Some(PersonalityMode::Developer);
        }

        // Assistant triggers
        if text.contains("assistant mode") 
           || text.contains("switch to assistant") 
           || text.contains("change to assistant") 
           || text.contains("be an assistant") 
           || text.contains("personality assistant")
           || text.contains("standard mode") {
            return Some(PersonalityMode::Assistant);
        }

        None
    }
}
