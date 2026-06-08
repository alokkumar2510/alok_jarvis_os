use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use crate::AppState;

pub struct AudioRecorder {
    is_recording: Arc<Mutex<bool>>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
        }
    }

    /// Captures audio from default input device, uses VAD (energy threshold),
    /// and saves a 16kHz mono WAV file when silence is detected.
    pub fn record_to_file(&self, target_path: &str, app: &AppHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut recording = self.is_recording.lock().unwrap();
        if *recording {
            return Err("Already recording".into());
        }
        *recording = true;

        let is_recording_clone = self.is_recording.clone();
        let audio_buffer = Arc::new(Mutex::new(Vec::new()));
        let audio_buffer_clone = audio_buffer.clone();

        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input audio device found")?;
        let config = device.default_input_config()?;
        let channels = config.channels();
        let sample_rate = config.sample_rate().0;
        let sample_format = config.sample_format();

        let stream_result = match sample_format {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.config(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buffer) = audio_buffer_clone.lock() {
                        let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                        buffer.extend_from_slice(&f32_samples);
                    }
                },
                |err| eprintln!("AudioRecorder stream error: {}", err),
                None
            ),
            cpal::SampleFormat::U16 => device.build_input_stream(
                &config.config(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buffer) = audio_buffer_clone.lock() {
                        let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                        buffer.extend_from_slice(&f32_samples);
                    }
                },
                |err| eprintln!("AudioRecorder stream error: {}", err),
                None
            ),
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.config(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buffer) = audio_buffer_clone.lock() {
                        buffer.extend_from_slice(data);
                    }
                },
                |err| eprintln!("AudioRecorder stream error: {}", err),
                None
            ),
            _ => return Err(format!("Unsupported sample format {:?}", sample_format).into()),
        };

        let stream = match stream_result {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to build input stream: {}", e).into()),
        };

        stream.play()?;

        // Retrieve dynamic sensitivity threshold
        let (silence_threshold, sens) = {
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(db) = state.db.lock() {
                    let sens_str = db.get_setting("wakeword_sensitivity").unwrap_or_default().unwrap_or_else(|| "0.75".to_string());
                    let sens = sens_str.parse::<f64>().unwrap_or(0.75);
                    let thresh = (1.0 - sens) * 0.075 + 0.005;
                    (thresh as f32, sens)
                } else {
                    (0.03, 0.75)
                }
            } else {
                (0.03, 0.75)
            }
        };

        println!("AudioRecorder: Recording command with VAD. Dynamic threshold: {:.4} (sensitivity: {:.2})", silence_threshold, sens);
        crate::log_dev_event(app, "Voice", "info", &format!("Recording active. Dynamic VAD threshold: {:.4} (sensitivity: {:.2})", silence_threshold, sens));

        let _ = app.emit("voice-state-change", serde_json::json!({
            "state": "listening",
            "transcript": "",
            "message": "Listening for command..."
        }));

        let mut last_voice_activity = Instant::now();
        let mut speech_detected = false;
        let timeout_duration = Duration::from_secs(10); // max recording time
        let start_time = Instant::now();

        while *is_recording_clone.lock().unwrap() {
            std::thread::sleep(Duration::from_millis(100));

            let buffer = {
                let buf = audio_buffer.lock().unwrap();
                buf.clone()
            };

            if buffer.is_empty() {
                if start_time.elapsed() > timeout_duration {
                    break;
                }
                continue;
            }

            // Calculate RMS on recent 200ms window of audio to detect silence accurately
            let window_size = ((sample_rate as usize * channels as usize) / 5).max(100);
            let recent_samples = if buffer.len() > window_size {
                &buffer[buffer.len() - window_size..]
            } else {
                &buffer[..]
            };

            let sum: f32 = recent_samples.iter().map(|&x| x * x).sum();
            let rms = (sum / recent_samples.len() as f32).sqrt();

            if rms > silence_threshold {
                last_voice_activity = Instant::now();
                if !speech_detected {
                    speech_detected = true;
                    println!("AudioRecorder: Voice activity detected.");
                    crate::log_dev_event(app, "Voice", "info", "Voice activity detected (RMS crossed VAD threshold).");
                }
            } else if speech_detected && last_voice_activity.elapsed() > Duration::from_millis(1500) {
                // Silence for 1.5s after speech
                println!("AudioRecorder: Silence detected. Stopping recording.");
                crate::log_dev_event(app, "Voice", "info", "Silence detected. Stopping microphone recording.");
                break;
            }

            // Max recording safety limit
            if start_time.elapsed() > timeout_duration {
                println!("AudioRecorder: Max recording duration reached.");
                crate::log_dev_event(app, "Voice", "warn", "Max recording duration (10s) reached.");
                break;
            }
        }

        // Stop stream
        drop(stream);
        *is_recording_clone.lock().unwrap() = false;

        // Process audio and save WAV
        let final_buffer = audio_buffer.lock().unwrap();
        if final_buffer.is_empty() {
            return Err("No audio captured".into());
        }

        let processed = resample_mono(&final_buffer, channels, sample_rate, 16000);
        save_wav_file(target_path, &processed)?;

        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut recording) = self.is_recording.lock() {
            *recording = false;
        }
    }
}

// Helpers for resampling and WAV saving
fn resample_mono(input: &[f32], channels: u16, from_rate: u32, to_rate: u32) -> Vec<f32> {
    if channels == 1 && from_rate == to_rate {
        return input.to_vec();
    }
    let mono = if channels > 1 {
        let mut m = Vec::with_capacity(input.len() / channels as usize);
        let ch = channels as usize;
        for chunk in input.chunks_exact(ch) {
            let sum: f32 = chunk.iter().sum();
            m.push(sum / channels as f32);
        }
        m
    } else {
        input.to_vec()
    };
    if from_rate == to_rate {
        return mono;
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let target_len = (mono.len() as f64 / ratio).round() as usize;
    let mut output = Vec::with_capacity(target_len);
    for i in 0..target_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = pos - idx as f64;
        if idx + 1 < mono.len() {
            let val = mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32;
            output.push(val);
        } else if idx < mono.len() {
            output.push(mono[idx]);
        }
    }
    output
}

fn save_wav_file(path_str: &str, samples: &[f32]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path_str, spec)?;
    for &sample in samples {
        let amplitude = (sample * 32767.0) as i16;
        writer.write_sample(amplitude)?;
    }
    writer.finalize()?;
    Ok(())
}
