use crate::database::db::ConversationEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emotion {
    Happy,
    Excited,
    Frustrated,
    Confused,
    Focused,
    Stressed,
    Neutral,
}

impl Emotion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Emotion::Happy => "happy",
            Emotion::Excited => "excited",
            Emotion::Frustrated => "frustrated",
            Emotion::Confused => "confused",
            Emotion::Focused => "focused",
            Emotion::Stressed => "stressed",
            Emotion::Neutral => "neutral",
        }
    }
}

pub fn detect_emotion(
    voice_tone: Option<&str>,
    word_choice: &str,
    history: &[ConversationEntry],
) -> Emotion {
    // 1. Check voice tone first (highest priority if available)
    if let Some(tone) = voice_tone {
        let tone_lower = tone.to_lowercase();
        if tone_lower.contains("laughing") || tone_lower.contains("happy") || tone_lower.contains("cheerful") {
            return Emotion::Happy;
        } else if tone_lower.contains("excited") || tone_lower.contains("loud_fast") {
            return Emotion::Excited;
        } else if tone_lower.contains("angry") || tone_lower.contains("frustrated") || tone_lower.contains("harsh") {
            return Emotion::Frustrated;
        } else if tone_lower.contains("hesitant") || tone_lower.contains("confused") {
            return Emotion::Confused;
        } else if tone_lower.contains("stressed") || tone_lower.contains("trembling") || tone_lower.contains("nervous") {
            return Emotion::Stressed;
        } else if tone_lower.contains("focused") || tone_lower.contains("monotone_fast") {
            return Emotion::Focused;
        }
    }

    // 2. Check word choice (sentiment keyword matching)
    let words = word_choice.to_lowercase();
    
    // Frustrated
    if words.contains("annoyed") || words.contains("broken") || words.contains("stupid") 
       || words.contains("hate") || words.contains("not working") || words.contains("frustrated") 
       || words.contains("useless") || words.contains("garbage") || words.contains("error again") 
       || words.contains("why doesn't this") || words.contains("damn") || words.contains("crap") {
        return Emotion::Frustrated;
    }
    
    // Excited
    if words.contains("awesome") || words.contains("excited") || words.contains("incredible") 
       || words.contains("amazing") || words.contains("wonderful") || words.contains("great success") 
       || words.contains("woohoo") || words.contains("perfect!") || words.contains("so cool") {
        return Emotion::Excited;
    }

    // Happy
    if words.contains("great") || words.contains("happy") || words.contains("thanks") 
       || words.contains("thank you") || words.contains("love this") || words.contains("perfect") 
       || words.contains("good job") || words.contains("nice") {
        return Emotion::Happy;
    }

    // Stressed
    if words.contains("worry") || words.contains("anxious") || words.contains("panic") 
       || words.contains("stressed") || words.contains("deadline") || words.contains("hurry") 
       || words.contains("late") || words.contains("scared") || words.contains("please help") {
        return Emotion::Stressed;
    }

    // Confused
    if words.contains("confused") || words.contains("don't understand") || words.contains("not sure") 
       || words.contains("what is this") || words.contains("what does it mean") || words.contains("how do i")
       || words.contains("why is it") || words.contains("makes no sense") || words.contains("stuck") {
        return Emotion::Confused;
    }

    // Focused
    if words.contains("compile") || words.contains("debug") || words.contains("analyze") 
       || words.contains("refactor") || words.contains("optimize") || words.contains("implement") 
       || words.contains("code") || words.contains("build") || words.contains("run") {
        return Emotion::Focused;
    }

    // 3. Check conversation history (repetitive negative inputs or failures)
    if !history.is_empty() {
        let user_entries: Vec<&ConversationEntry> = history.iter()
            .filter(|e| e.sender.to_lowercase() == "user")
            .collect();

        if user_entries.len() >= 3 {
            let msg1 = user_entries[0].message.to_lowercase();
            let msg2 = user_entries[1].message.to_lowercase();
            let msg3 = user_entries[2].message.to_lowercase();

            if msg1.contains("error") && msg2.contains("error") && msg3.contains("error") {
                return Emotion::Frustrated;
            }
            if msg1.len() < 10 && msg2.len() < 10 && msg3.len() < 10 && msg1 == msg2 && msg2 == msg3 {
                return Emotion::Frustrated;
            }
        }
    }

    Emotion::Neutral
}

pub fn adapt_response(emotion: Emotion, base_response: &str) -> String {
    match emotion {
        Emotion::Frustrated => {
            format!("I apologize for the frustration. Let's solve this immediately:\n\n{}", base_response)
        }
        Emotion::Stressed => {
            format!("Take a breath, I'm here to help you get this done. Let's go through it step-by-step:\n\n{}", base_response)
        }
        Emotion::Focused => {
            base_response.to_string()
        }
        Emotion::Confused => {
            format!("Let me break this down simply to clarify:\n\n{}", base_response)
        }
        Emotion::Happy | Emotion::Excited => {
            format!("Fantastic! 😊\n\n{}", base_response)
        }
        Emotion::Neutral => base_response.to_string(),
    }
}

pub fn get_speech_modifiers(emotion: Emotion) -> (f64, f64) {
    // Returns (rate, pitch)
    match emotion {
        Emotion::Frustrated => (1.0, 0.95), // Calm rate, soothing lower pitch
        Emotion::Stressed => (0.90, 0.95),   // Speak slower and reassuringly
        Emotion::Focused => (1.10, 1.00),    // Deliver code/technical data faster
        Emotion::Excited => (1.15, 1.10),    // Higher energy and faster
        Emotion::Happy => (1.05, 1.05),      // Upbeat
        Emotion::Confused => (0.95, 1.00),   // Slower for clarity
        Emotion::Neutral => (1.00, 1.00),    // Default
    }
}
