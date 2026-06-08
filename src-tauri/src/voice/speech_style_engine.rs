use crate::voice::emotion_engine::Emotion;
use crate::voice::personality_engine::PersonalityMode;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpeechStyle {
    pub speed: f32,             // Speaking speed (0.5 to 2.0)
    pub pause_duration: f32,    // Pause duration multiplier
    pub emphasis: f32,         // Pitch emphasis level
    pub warmth: f32,           // Phrasing warmth (0.0 to 1.0)
    pub energy: f32,           // Voice energy (0.0 to 1.0)
    pub formality: f32,        // Dialogue formality (0.0 to 1.0)
}

pub struct SpeechStyleEngine;

impl SpeechStyleEngine {
    /// Determines the speech style parameters based on detected emotion and active personality mode.
    pub fn determine_style(emotion: Emotion, personality: PersonalityMode) -> SpeechStyle {
        // 1. Establish base modifiers per personality mode
        let mut style = match personality {
            PersonalityMode::Assistant => SpeechStyle {
                speed: 1.0,
                pause_duration: 1.0,
                emphasis: 1.0,
                warmth: 0.5,
                energy: 0.5,
                formality: 0.8,
            },
            PersonalityMode::Friendly => SpeechStyle {
                speed: 1.05,
                pause_duration: 0.8,
                emphasis: 1.15,
                warmth: 0.8,
                energy: 0.8,
                formality: 0.3,
            },
            PersonalityMode::Professional => SpeechStyle {
                speed: 1.0,
                pause_duration: 1.1,
                emphasis: 0.95,
                warmth: 0.6,
                energy: 0.5,
                formality: 0.9,
            },
            PersonalityMode::Companion => SpeechStyle {
                speed: 0.96,
                pause_duration: 1.3,
                emphasis: 1.05,
                warmth: 0.95,
                energy: 0.4,
                formality: 0.2,
            },
            PersonalityMode::Developer => SpeechStyle {
                speed: 1.15,
                pause_duration: 0.7,
                emphasis: 0.9,
                warmth: 0.3,
                energy: 0.6,
                formality: 0.6,
            },
        };

        // 2. Adjust based on user's emotional state
        match emotion {
            Emotion::Happy => {
                style.speed *= 1.05;
                style.emphasis *= 1.10;
                style.energy = (style.energy + 0.15).min(1.0);
                style.warmth = (style.warmth + 0.10).min(1.0);
            }
            Emotion::Excited => {
                style.speed *= 1.12;
                style.emphasis *= 1.20;
                style.pause_duration *= 0.75;
                style.energy = (style.energy + 0.25).min(1.0);
            }
            Emotion::Sad => {
                style.speed *= 0.84;
                style.emphasis *= 0.90;
                style.pause_duration *= 1.4;
                style.energy = (style.energy - 0.20).max(0.1);
                style.warmth = (style.warmth + 0.20).min(1.0); // Comforting tone
            }
            Emotion::Angry => {
                style.speed *= 0.95; // Speak slower to project a calm, stabilizing tone
                style.emphasis *= 0.95;
                style.pause_duration *= 1.2;
                style.warmth = (style.warmth + 0.15).min(1.0);
            }
            Emotion::Frustrated => {
                style.speed *= 0.90;
                style.pause_duration *= 1.3;
                style.energy = (style.energy - 0.10).max(0.2);
                style.warmth = (style.warmth + 0.25).min(1.0);
            }
            Emotion::Confused => {
                style.speed *= 0.92;
                style.pause_duration *= 1.25;
            }
            Emotion::Focused => {
                style.speed *= 1.10;
                style.pause_duration *= 0.80;
                style.formality = (style.formality + 0.10).min(1.0);
            }
            Emotion::Stressed => {
                style.speed *= 0.88;
                style.pause_duration *= 1.35;
                style.energy = (style.energy - 0.15).max(0.2);
                style.warmth = (style.warmth + 0.20).min(1.0);
            }
            Emotion::Neutral => {}
        }

        // Clamp speed to safe limits for local Piper ONNX model
        style.speed = style.speed.clamp(0.5, 2.0);
        style.pause_duration = style.pause_duration.clamp(0.5, 2.0);

        style
    }

    /// Pre-processes the output speech text to add comma pauses or dashes matching the speaking style.
    pub fn format_text_style(text: &str, style: &SpeechStyle) -> String {
        let mut formatted = text.to_string();

        // If highly warm and slow (sad/stressed/companion mode), insert ellipses or dashes to force natural pauses
        if style.warmth > 0.85 && style.speed < 0.95 {
            formatted = formatted
                .replace(", ", "... ")
                .replace("; ", "... ")
                .replace(" - ", "... ");
        }

        // Clean double spaces or duplicate punctuation
        formatted = formatted.replace("  ", " ").replace("...", "... ");
        formatted
    }
}
