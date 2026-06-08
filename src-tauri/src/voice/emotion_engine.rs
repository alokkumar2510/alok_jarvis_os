use crate::database::db::ConversationEntry;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoiceCharacteristics {
    pub pitch: f32,
    pub energy: f32,
    pub speaking_rate: f32,
    pub tone_description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Emotion {
    Happy,
    Excited,
    Sad,
    Angry,
    Frustrated,
    Confused,
    Stressed,
    Focused,
    Neutral,
}

impl Emotion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Emotion::Happy => "happy",
            Emotion::Excited => "excited",
            Emotion::Sad => "sad",
            Emotion::Angry => "angry",
            Emotion::Frustrated => "frustrated",
            Emotion::Confused => "confused",
            Emotion::Stressed => "stressed",
            Emotion::Focused => "focused",
            Emotion::Neutral => "neutral",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "happy" => Emotion::Happy,
            "excited" => Emotion::Excited,
            "sad" => Emotion::Sad,
            "angry" => Emotion::Angry,
            "frustrated" => Emotion::Frustrated,
            "confused" => Emotion::Confused,
            "stressed" => Emotion::Stressed,
            "focused" => Emotion::Focused,
            _ => Emotion::Neutral,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmotionResult {
    pub emotion: Emotion,
    pub score: f32, // The match confidence score (0.0 to 1.0)
}

pub struct EmotionEngine;

impl EmotionEngine {
    /// Detects user's emotional state, returning the best matching emotion and its score.
    pub fn detect_emotion(
        transcript: &str,
        context_history: &[ConversationEntry],
    ) -> EmotionResult {
        let mut happy_score = 0.0;
        let mut excited_score = 0.0;
        let mut sad_score = 0.0;
        let mut angry_score = 0.0;
        let mut frustrated_score = 0.0;
        let mut confused_score = 0.0;
        let mut stressed_score = 0.0;
        let mut focused_score = 0.0;
        let mut neutral_score = 0.5; // Baseline neutral score

        let words = transcript.to_lowercase();

        // 1. Process Transcript Keyword Matches
        
        // Happy
        if words.contains("great") || words.contains("happy") || words.contains("thanks") 
           || words.contains("thank you") || words.contains("love this") || words.contains("perfect") 
           || words.contains("good job") || words.contains("nice") || words.contains("glad") {
            happy_score += 2.5;
        }

        // Excited
        if words.contains("awesome") || words.contains("excited") || words.contains("incredible") 
           || words.contains("amazing") || words.contains("wonderful") || words.contains("so cool")
           || words.contains("woohoo") || words.contains("unbelievable") || words.contains("fantastic") {
            excited_score += 3.0;
        }

        // Sad
        if words.contains("sad") || words.contains("unhappy") || words.contains("depressed") 
           || words.contains("sorry") || words.contains("unfortunately") || words.contains("bad news") 
           || words.contains("lost") || words.contains("hurt") || words.contains("disappointed") {
            sad_score += 3.0;
        }

        // Angry
        if words.contains("angry") || words.contains("furious") || words.contains("hate") 
           || words.contains("mad") || words.contains("stupid") || words.contains("annoying") 
           || words.contains("idiot") || words.contains("shut up") || words.contains("nonsense") {
            angry_score += 3.0;
        }

        // Frustrated
        if words.contains("broken") || words.contains("useless") || words.contains("garbage") 
           || words.contains("waste") || words.contains("not working") || words.contains("fails") 
           || words.contains("again") || words.contains("damn") || words.contains("crap") 
           || words.contains("frustrated") || words.contains("why doesn't this") || words.contains("error again") {
            frustrated_score += 3.0;
        }

        // Confused
        if words.contains("confused") || words.contains("don't understand") || words.contains("not sure") 
           || words.contains("what is this") || words.contains("what does it mean") || words.contains("how do i")
           || words.contains("why is it") || words.contains("makes no sense") || words.contains("stuck") 
           || words.contains("how to") || words.contains("puzzled") {
            confused_score += 2.5;
        }

        // Stressed
        if words.contains("stressed") || words.contains("tired") || words.contains("exhausted") 
           || words.contains("anxious") || words.contains("worried") || words.contains("pressure") 
           || words.contains("nervous") || words.contains("stressed out") {
            stressed_score += 2.5;
        }

        // Focused
        if words.contains("compile") || words.contains("debug") || words.contains("analyze") 
           || words.contains("refactor") || words.contains("optimize") || words.contains("implement") 
           || words.contains("code") || words.contains("build") || words.contains("run") 
           || words.contains("monitor") || words.contains("test") || words.contains("diagnose") {
            focused_score += 2.5;
        }

        // Punctuation cues
        if transcript.contains("!") {
            excited_score += 1.0;
            angry_score += 0.8;
            happy_score += 0.5;
        }
        if transcript.contains("?") {
            confused_score += 1.5;
        }

        // Caps-lock detection
        let has_caps = transcript.split_whitespace()
            .filter(|w| w.len() >= 3)
            .any(|w| w.chars().all(|c| c.is_uppercase() || c.is_ascii_punctuation()));
        if has_caps {
            angry_score += 1.5;
            excited_score += 1.2;
        }

        // 2. Process Conversation Context / Previous Interactions
        if !context_history.is_empty() {
            let user_entries: Vec<&ConversationEntry> = context_history.iter()
                .filter(|e| e.sender.to_lowercase() == "user")
                .collect();

            if user_entries.len() >= 3 {
                let msg1 = user_entries[0].message.to_lowercase();
                let msg2 = user_entries[1].message.to_lowercase();
                let msg3 = user_entries[2].message.to_lowercase();

                // Repeat errors -> high frustration
                if msg1.contains("error") && msg2.contains("error") && msg3.contains("error") {
                    frustrated_score += 2.0;
                }
                
                // Short repeating inputs -> annoyed/sad/angry
                if msg1.len() < 8 && msg2.len() < 8 && msg3.len() < 8 {
                    frustrated_score += 1.0;
                    angry_score += 0.5;
                }

                // Repeating success loops -> happy
                if msg1.contains("success") && msg2.contains("success") {
                    happy_score += 1.2;
                }
            }
        }

        // 3. Compile Scores & Select Max
        let mut scores = [
            (Emotion::Happy, happy_score),
            (Emotion::Excited, excited_score),
            (Emotion::Sad, sad_score),
            (Emotion::Angry, angry_score),
            (Emotion::Frustrated, frustrated_score),
            (Emotion::Confused, confused_score),
            (Emotion::Stressed, stressed_score),
            (Emotion::Focused, focused_score),
            (Emotion::Neutral, neutral_score),
        ];

        // Softmax-like normalization to produce a score between 0.0 and 1.0
        let total_score: f32 = scores.iter().map(|&(_, s)| s).sum();
        let mut max_emotion = Emotion::Neutral;
        let mut max_raw_score = -1.0;

        for (emotion, score) in scores {
            if score > max_raw_score {
                max_raw_score = score;
                max_emotion = emotion;
            }
        }

        let normalized_score = if total_score > 0.0 {
            max_raw_score / total_score
        } else {
            0.5
        };

        EmotionResult {
            emotion: max_emotion,
            score: normalized_score.clamp(0.0, 1.0),
        }
    }
}
