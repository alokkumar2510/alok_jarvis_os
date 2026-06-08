pub mod recorder;
pub mod tts;
pub mod whisper;
pub mod wakeword;
pub mod emotion_engine;
pub mod personality_engine;
pub mod speech_style_engine;
pub mod voice_manager;
pub mod voice_preferences;
pub mod presence_system;

pub use whisper::WhisperEngine;
pub use wakeword::WakeWordService;
pub use emotion_engine::{Emotion, VoiceCharacteristics, EmotionEngine};
pub use personality_engine::PersonalityMode;
pub use speech_style_engine::{SpeechStyle, SpeechStyleEngine};
pub use voice_manager::VoiceManager;
pub use voice_preferences::VoicePreferences;
pub use presence_system::play_presence_signature;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    Idle,
    Listening,
    Processing,
    Speaking,
}


