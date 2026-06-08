use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};

pub struct WakeWordService {
    is_running: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    app_handle: AppHandle,
}

impl WakeWordService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            app_handle,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut running = self.is_running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;

        let is_running_clone = self.is_running.clone();
        let audio_buffer_clone = self.audio_buffer.clone();
        let app_handle_clone = self.app_handle.clone();

        println!("WakeWordService: Initializing audio device...");

        std::thread::spawn(move || {
            // OpenWakeWord via Python subprocess is disabled: spawning
            // Command::new("python.exe") from a MinGW Rust background thread causes
            // heap/stack corruption (exit 0xcfffffff). Using energy-threshold VAD instead.
            println!("WakeWordService: Using energy-threshold VAD wakeup (OpenWakeWord disabled).");


            // 2. Setup audio capture stream
            let host = cpal::default_host();
            let device = match host.default_input_device() {
                Some(d) => d,
                None => {
                    eprintln!("WakeWordService error: No input audio device found.");
                    return;
                }
            };

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("WakeWordService error: Default input config failed: {}", e);
                    return;
                }
            };

            let channels = config.channels();
            let sample_rate = config.sample_rate().0;
            let sample_format = config.sample_format();

            println!(
                "WakeWordService: Microphone connected. Sample Rate: {}, Channels: {}, Format: {:?}",
                sample_rate, channels, sample_format
            );

            let buffer_err_clone = audio_buffer_clone.clone();

            let stream_result = match sample_format {
                cpal::SampleFormat::I16 => device.build_input_stream(
                    &config.config(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = buffer_err_clone.lock() {
                            let f32_samples: Vec<f32> = data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            buffer.extend_from_slice(&f32_samples);
                        }
                    },
                    |err| eprintln!("WakeWordService stream error: {}", err),
                    None
                ),
                cpal::SampleFormat::U16 => device.build_input_stream(
                    &config.config(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = buffer_err_clone.lock() {
                            let f32_samples: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                            buffer.extend_from_slice(&f32_samples);
                        }
                    },
                    |err| eprintln!("WakeWordService stream error: {}", err),
                    None
                ),
                cpal::SampleFormat::F32 => device.build_input_stream(
                    &config.config(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut buffer) = buffer_err_clone.lock() {
                            buffer.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("WakeWordService stream error: {}", err),
                    None
                ),
                _ => {
                    eprintln!("WakeWordService error: Unsupported sample format {:?}", sample_format);
                    return;
                }
            };

            let stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("WakeWordService error: Build input stream failed: {}", e);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                eprintln!("WakeWordService error: Play stream failed: {}", e);
                return;
            }

            println!("WakeWordService: Microphone stream running.");

            let app_state = app_handle_clone.state::<crate::AppState>();
            let voice_state = app_state.voice_state.clone();

            let silence_threshold = 0.035f32; // tuned: above ambient (0.005-0.015), responsive to normal speech

            let mut command_accumulator: Vec<f32> = Vec::new();
            let mut waiting_for_tts = false;
            let mut last_voice_activity = Instant::now();
            let mut speech_detected = false;
            let mut listening_start = Instant::now();

            while *is_running_clone.lock().unwrap() {
                std::thread::sleep(Duration::from_millis(80));

                let samples = {
                    let mut buffer = audio_buffer_clone.lock().unwrap();
                    if buffer.is_empty() {
                        continue;
                    }
                    let res = buffer.clone();
                    buffer.clear();
                    res
                };

                let current_state = {
                    if let Ok(vs) = voice_state.lock() {
                        *vs
                    } else {
                        super::VoiceState::Idle
                    }
                };

                // Retrieve dynamic sensitivity threshold
                let (silence_threshold_val, _) = {
                    if let Ok(db) = app_state.db.lock() {
                        let sens_str = db.get_setting("wakeword_sensitivity").unwrap_or_default().unwrap_or_else(|| "0.75".to_string());
                        let sens = sens_str.parse::<f64>().unwrap_or(0.75);
                        let thresh = (1.0 - sens) * 0.075 + 0.005;
                        (thresh as f32, sens)
                    } else {
                        (0.03, 0.75)
                    }
                };

                match current_state {
                    super::VoiceState::Idle | super::VoiceState::Speaking | super::VoiceState::Processing => {
                        // VAD energy estimation (for fallback or general logging)
                        let sum: f32 = samples.iter().map(|&x| x * x).sum();
                        let rms = (sum / samples.len() as f32).sqrt();

                        let mut triggered = false;

                        // When speaking, we make the wakeup/interrupt threshold a bit higher
                        // to prevent speaker output from self-triggering a microphone feedback loop.
                        let trigger_threshold = if current_state == super::VoiceState::Speaking {
                            silence_threshold_val.max(0.035)
                        } else {
                            silence_threshold_val
                        };

                        if rms > trigger_threshold {
                            triggered = true;
                            println!(
                                "WakeWordService: VAD threshold wakeup triggered (RMS: {:.3}, State: {:?}, Thresh: {:.3}).",
                                rms, current_state, trigger_threshold
                            );
                        }

                        if triggered {
                            println!("WakeWordService: WAKING UP / INTERRUPTING. Activating command recording...");
                            
                            // Call unified interrupt system
                            crate::control::interrupt_engine::interrupt_on_wakeword(&app_handle_clone);

                            // Cancel active planner execution
                            app_state.planner.cancel_plan();

                            // Play wake word confirmation (Yes?)
                            let _ = super::tts::speak("Yes?");

                            crate::log_dev_event(&app_handle_clone, "Wake Word", "info", "Wake word detection triggered pipeline.");
                            crate::log_dev_event(&app_handle_clone, "Voice", "info", "Playing wake word confirmation tone (TTS Prompt 'Yes?')");

                            waiting_for_tts = true;
                            listening_start = Instant::now();
                            command_accumulator.clear();
                        }
                    }
                    super::VoiceState::Listening => {
                        if waiting_for_tts {
                            if super::tts::is_speaking() {
                                // Discard samples while TTS plays
                                continue;
                            }
                            // TTS finished speaking! Start actual recording
                            waiting_for_tts = false;
                            last_voice_activity = Instant::now();
                            speech_detected = false;
                            command_accumulator.clear();
                            println!("WakeWordService: TTS finished. Recording user command...");
                            crate::log_dev_event(&app_handle_clone, "Voice", "info", "Recording active. Listening for command...");
                        } else {
                            // Accumulate samples
                            command_accumulator.extend_from_slice(&samples);

                            // Calculate RMS on recent 200ms window
                            let window_size = ((sample_rate as usize * channels as usize) / 5).max(100);
                            let recent_samples = if command_accumulator.len() > window_size {
                                &command_accumulator[command_accumulator.len() - window_size..]
                            } else {
                                &command_accumulator[..]
                            };

                            let sum: f32 = recent_samples.iter().map(|&x| x * x).sum();
                            let rms = (sum / recent_samples.len() as f32).sqrt();

                            if rms > silence_threshold_val {
                                last_voice_activity = Instant::now();
                                if !speech_detected {
                                    speech_detected = true;
                                    println!("WakeWordService: Voice activity detected (RMS: {:.4}).", rms);
                                    crate::log_dev_event(&app_handle_clone, "Voice", "info", "Voice activity detected (RMS crossed VAD threshold).");
                                }
                            } else if speech_detected && last_voice_activity.elapsed() > Duration::from_millis(1500) {
                                println!("WakeWordService: Silence detected. Stopping recording.");
                                crate::log_dev_event(&app_handle_clone, "Voice", "info", "Silence detected. Stopping microphone recording.");
                                
                                // Save and trigger
                                let wav_path = "e:\\ALOK PC\\bin\\input.wav";
                                let processed = resample_mono(&command_accumulator, channels, sample_rate, 16000);
                                if let Err(e) = save_wav_file(wav_path, &processed) {
                                    eprintln!("WakeWordService error saving WAV: {}", e);
                                }

                                if let Ok(mut vs) = voice_state.lock() {
                                    *vs = super::VoiceState::Processing;
                                }

                                let app_clone = app_handle_clone.clone();
                                std::thread::spawn(move || {
                                    let _ = crate::jarvis_runtime::JarvisRuntime::handle_trigger(&app_clone, wav_path);
                                });
                            }

                            // Safety limit
                            if listening_start.elapsed() > Duration::from_secs(10) {
                                println!("WakeWordService: Max recording duration reached.");
                                crate::log_dev_event(&app_handle_clone, "Voice", "warn", "Max recording duration (10s) reached.");

                                // Save and trigger
                                let wav_path = "e:\\ALOK PC\\bin\\input.wav";
                                let processed = resample_mono(&command_accumulator, channels, sample_rate, 16000);
                                if let Err(e) = save_wav_file(wav_path, &processed) {
                                    eprintln!("WakeWordService error saving WAV: {}", e);
                                }

                                if let Ok(mut vs) = voice_state.lock() {
                                    *vs = super::VoiceState::Processing;
                                }

                                let app_clone = app_handle_clone.clone();
                                std::thread::spawn(move || {
                                    let _ = crate::jarvis_runtime::JarvisRuntime::handle_trigger(&app_clone, wav_path);
                                });
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
    }
}

// Resampling helper
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
