use std::path::Path;
use crate::voice::emotion_engine::{Emotion, VoiceCharacteristics};
use crate::voice::personality_engine::PersonalityMode;

pub struct VoiceManager;

impl VoiceManager {
    /// Extracts voice characteristics (pitch, energy, speaking rate) from a recorded WAV file.
    pub fn extract_voice_characteristics(wav_path: &str, transcript: &str) -> Option<VoiceCharacteristics> {
        if !Path::new(wav_path).exists() {
            return None;
        }

        let mut reader = hound::WavReader::open(wav_path).ok()?;
        let spec = reader.spec();
        
        // Only support 16-bit integer PCM for simple pitch estimation
        if spec.bits_per_sample != 16 || spec.sample_format != hound::SampleFormat::Int {
            return None;
        }

        let samples: Vec<i16> = reader.samples::<i16>().filter_map(Result::ok).collect();
        if samples.is_empty() {
            return None;
        }

        let sample_rate = spec.sample_rate;
        let duration_secs = samples.len() as f32 / sample_rate as f32;

        // 1. Calculate RMS Energy
        let mut sum_sq = 0.0;
        for &sample in &samples {
            let val = sample as f32 / 32768.0;
            sum_sq += val * val;
        }
        let energy = (sum_sq / samples.len() as f32).sqrt();

        // 2. Convert samples to f32 slice for pitch autocorrelation
        let f32_samples: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();
        let pitch = Self::calculate_pitch_autocorrelation(&f32_samples, sample_rate);

        // 3. Calculate speaking rate (words per second)
        let words = transcript.split_whitespace().count();
        let speaking_rate = if duration_secs > 0.2 {
            words as f32 / duration_secs
        } else {
            0.0
        };

        // Determine a simple tone description based on basic rules
        let mut tone_description = None;
        if pitch > 200.0 {
            if energy > 0.05 {
                tone_description = Some("excited_high_pitch".to_string());
            } else {
                tone_description = Some("stressed_high_pitch".to_string());
            }
        } else if pitch > 0.1 && pitch < 110.0 {
            if energy < 0.012 {
                tone_description = Some("low_sad_pitch".to_string());
            } else {
                tone_description = Some("calm_monotone".to_string());
            }
        }

        Some(VoiceCharacteristics {
            pitch,
            energy,
            speaking_rate,
            tone_description,
        })
    }

    /// Autocorrelation-based fundamental frequency (pitch) detection.
    /// Targeted for human speech frequencies (80Hz to 320Hz).
    fn calculate_pitch_autocorrelation(samples: &[f32], sample_rate: u32) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        // Lags corresponding to human speech frequencies (80Hz - 320Hz)
        let min_lag = (sample_rate as f32 / 320.0) as usize;
        let max_lag = (sample_rate as f32 / 80.0) as usize;

        let mut best_lag = 0;
        let mut max_correlation = -1.0;

        // Use a window of up to 4000 samples to perform autocorrelation
        let window_len = samples.len().min(4000);
        let window = &samples[..window_len];

        for lag in min_lag..=max_lag {
            let mut correlation = 0.0;
            let mut norm1 = 0.0;
            let mut norm2 = 0.0;
            let limit = window_len - lag;
            
            if limit == 0 {
                continue;
            }

            for i in 0..limit {
                correlation += window[i] * window[i + lag];
                norm1 += window[i] * window[i];
                norm2 += window[i + lag] * window[i + lag];
            }

            if norm1 > 0.0 && norm2 > 0.0 {
                let r = correlation / (norm1 * norm2).sqrt();
                if r > max_correlation {
                    max_correlation = r;
                    best_lag = lag;
                }
            }
        }

        if best_lag > 0 && max_correlation > 0.5 {
            sample_rate as f32 / best_lag as f32
        } else {
            0.0 // Pitch not detected
        }
    }

    /// Adapts the phrasing of ALOK's response based on the user's emotion and the active personality mode.
    pub fn get_adapted_phrasing(emotion: Emotion, personality: PersonalityMode, base_text: &str) -> String {
        let clean_text = base_text.trim();
        if clean_text.is_empty() {
            return String::new();
        }

        match emotion {
            Emotion::Happy => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("That's fantastic! Everything is working perfectly. I'm really glad to hear that! {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("It makes me so happy to see everything going smoothly for you. 😊 {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Build successful. No errors detected. {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("I'm pleased to report that the operation completed successfully: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("It is satisfactory that the task was completed successfully. {}", clean_text)
                    }
                }
            }
            Emotion::Excited => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("Wow, that is absolutely incredible! Woohoo! {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("This is amazing! I'm so excited for you! Let's keep this momentum going: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Execution successful. Performance optimal. {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("Excellent results. The task has completed with high efficiency: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("The results are highly promising. Execution completed efficiently: {}", clean_text)
                    }
                }
            }
            Emotion::Frustrated => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("Oh no, that's frustrating. Let's solve this together. I'm on it: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("I understand. Let's solve this together. Take a breath, we'll figure it out: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Error encountered. Executing diagnostic recovery routines: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("I apologize for the frustration. Let us analyze and resolve the issue systematically: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("Let us address this bottleneck. I will begin diagnostics immediately: {}", clean_text)
                    }
                }
            }
            Emotion::Angry => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("Hey, let's stay calm. I'm here to support you. Let's tackle it: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("I completely understand your frustration. Let's work through it step-by-step, I'm right here: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Halting execution. Analyzing logs for failure roots: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("I apologize. Let us address this concern immediately: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("I understand your concerns. I will adjust the parameters to correct this: {}", clean_text)
                    }
                }
            }
            Emotion::Stressed => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("Take it easy, don't worry! We will get this sorted out in no time: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("I know you have a lot on your plate right now. Let's take it slow, one step at a time: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Running optimizations. Standing by: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("I will manage the secondary tasks to reduce your workload. Let us focus on: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("Let us divide the tasks. I will handle the background operations: {}", clean_text)
                    }
                }
            }
            Emotion::Sad => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("I'm really sorry things are feeling down. I hope I can make this task a bit easier: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("I'm so sorry you're feeling this way. I'm right here with you, let's take a look: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Processing task requests. Standing by: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("I am here to assist you with this task. Please let me know how I can make this easier for you: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("I am at your service. Let me assist you to streamline this workflow: {}", clean_text)
                    }
                }
            }
            Emotion::Confused => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("No worries, it's a bit tricky! Let me break it down simply: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("It's totally normal to be unsure about this. Let's look at it together and clarify: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Query parameters ambiguous. Resolving context: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("Let me clarify the details of the operation to assist your understanding: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("Let us review the parameters to ensure clarity and correctness: {}", clean_text)
                    }
                }
            }
            Emotion::Focused => {
                match personality {
                    PersonalityMode::Friendly => {
                        format!("Got it. Let's focus and get this done: {}", clean_text)
                    }
                    PersonalityMode::Companion => {
                        format!("I see you're locked in. I'm here to support. Here is the info: {}", clean_text)
                    }
                    PersonalityMode::Developer => {
                        format!("Code check complete. Output: {}", clean_text)
                    }
                    PersonalityMode::Assistant => {
                        format!("Processing request. Output: {}", clean_text)
                    }
                    PersonalityMode::Professional => {
                        format!("Execution prioritized. System output: {}", clean_text)
                    }
                }
            }
            Emotion::Neutral => {
                match personality {
                    PersonalityMode::Developer => {
                        format!("Done. {}", clean_text)
                    }
                    _ => clean_text.to_string(),
                }
            }
        }
    }
}
