use std::fs::File;
use std::io::Write;
use std::process::{Command, Child, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::path::Path;
use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;
use crate::database::Database;
use crate::voice::voice_preferences::VoicePreferences;
use crate::voice::speech_style_engine::SpeechStyleEngine;
use crate::voice::emotion_engine::Emotion;

lazy_static::lazy_static! {
    static ref TTS_STATE: Arc<Mutex<TtsState>> = Arc::new(Mutex::new(TtsState::new()));
    static ref PLAY_MUTEX: Mutex<()> = Mutex::new(());
}

static TTS_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

struct TtsState {
    queue: Vec<String>,
    current_child: Option<Child>,
    worker_running: bool,
    currently_playing: bool,
    active_voice: String,
    active_speed: f32,
    active_emotion: Emotion,
    suspended_queue: Vec<String>,
    active_wav_bytes: Option<Vec<u8>>,
}

impl TtsState {
    fn new() -> Self {
        Self {
            queue: Vec::new(),
            current_child: None,
            worker_running: false,
            currently_playing: false,
            active_voice: "en_US-lessac-medium.onnx".to_string(),
            active_speed: 1.0,
            active_emotion: Emotion::Neutral,
            suspended_queue: Vec::new(),
            active_wav_bytes: None,
        }
    }
}

pub fn speak(text: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Load active voice and speed preferences from database
    let resolved_db = crate::resolve_path("alok_jarvis_os.db", "e:\\ALOK PC\\alok_jarvis_os.db");
    let (pref, active_emotion) = if Path::new(&resolved_db).exists() {
        if let Ok(db) = Database::new(&resolved_db) {
            let p = VoicePreferences::load_from_db(&db);
            let emo_str = db.get_setting("voice_active_emotion")
                .unwrap_or_default()
                .unwrap_or_else(|| "neutral".to_string());
            let e = Emotion::from_str(&emo_str);
            (p, e)
        } else {
            (VoicePreferences::default_preferences(), Emotion::Neutral)
        }
    } else {
        (VoicePreferences::default_preferences(), Emotion::Neutral)
    };

    // 2. Segment text into sentences to stream playback
    let sentences = segment_sentences(text);
    println!("TTS (Piper): Segmented text into {} sentences for streaming", sentences.len());

    let mut lock = TTS_STATE.lock().unwrap();
    // Resolve preferred voice name
    let voice = if pref.preferred_voice != "af_bella.bin" && pref.preferred_voice != "af_sarah.bin" && pref.preferred_voice != "en_US-lessac-medium.onnx" {
        pref.preferred_voice.clone()
    } else {
        "en_US-lessac-medium.onnx".to_string()
    };
    lock.active_voice = voice;
    lock.active_speed = pref.speaking_speed;
    lock.active_emotion = active_emotion;

    lock.queue.extend(sentences);

    if !lock.worker_running {
        lock.worker_running = true;
        let state_clone = TTS_STATE.clone();
        thread::spawn(move || {
            run_worker_loop(state_clone);
        });
    }

    Ok(())
}

pub fn stop() {
    let mut lock = TTS_STATE.lock().unwrap();
    if !lock.queue.is_empty() || lock.currently_playing {
        lock.suspended_queue = lock.queue.clone();
    } else {
        lock.suspended_queue.clear();
    }
    lock.queue.clear();
    lock.currently_playing = false;

    // Kill currently executing piper subprocess if any
    if let Some(mut child) = lock.current_child.take() {
        let _ = child.kill();
    }

    // Purge Win32 PlaySound
    unsafe {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(pszSound: *const u16, hmod: isize, fdwSound: u32) -> i32;
        }
        PlaySoundW(std::ptr::null(), 0, 0x0040); // SND_PURGE
    }

    // Give Windows background thread a tiny moment to process the purge command before dropping the buffer
    std::thread::sleep(std::time::Duration::from_millis(50));
    lock.active_wav_bytes = None;
}

pub fn resume() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut lock = TTS_STATE.lock().unwrap();
    if lock.suspended_queue.is_empty() {
        return Ok(());
    }

    let to_resume = lock.suspended_queue.clone();
    lock.suspended_queue.clear();
    lock.queue.extend(to_resume);

    if !lock.worker_running {
        lock.worker_running = true;
        let state_clone = TTS_STATE.clone();
        thread::spawn(move || {
            run_worker_loop(state_clone);
        });
    }

    Ok(())
}

pub fn is_speaking() -> bool {
    let lock = TTS_STATE.lock().unwrap();
    lock.worker_running || lock.currently_playing
}

fn run_worker_loop(state_ref: Arc<Mutex<TtsState>>) {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    loop {
        let (next_text, voice, speed, emotion) = {
            let mut lock = state_ref.lock().unwrap();
            if lock.queue.is_empty() {
                lock.worker_running = false;
                break;
            }
            let text = lock.queue.remove(0);
            lock.currently_playing = true;
            (text, lock.active_voice.clone(), lock.active_speed, lock.active_emotion)
        };

        if let Err(e) = process_and_play_sentence(&next_text, &voice, speed, emotion, &state_ref) {
            eprintln!("TTS (Piper): Error playing sentence: {}", e);
        }

        {
            let mut lock = state_ref.lock().unwrap();
            lock.currently_playing = false;
        }
    }

    unsafe {
        windows::Win32::System::Com::CoUninitialize();
    }
}

fn get_wav_duration_ms_bytes(bytes: &[u8]) -> u64 {
    if let Ok(reader) = hound::WavReader::new(std::io::Cursor::new(bytes)) {
        let spec = reader.spec();
        let duration = reader.duration() as u64; // total samples across all channels
        let sample_rate = spec.sample_rate as u64;
        let channels = spec.channels as u64;
        if sample_rate > 0 && channels > 0 {
            let samples_per_channel = duration / channels;
            return (samples_per_channel * 1000) / sample_rate;
        }
    }
    0
}

fn process_and_play_sentence(
    text: &str,
    voice: &str,
    speed: f32,
    emotion: Emotion,
    state_ref: &Arc<Mutex<TtsState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Determine active speech style using SpeechStyleEngine
    let preferences = {
        let resolved_db = crate::resolve_path("alok_jarvis_os.db", "e:\\ALOK PC\\alok_jarvis_os.db");
        if Path::new(&resolved_db).exists() {
            if let Ok(db) = Database::new(&resolved_db) {
                VoicePreferences::load_from_db(&db)
            } else {
                VoicePreferences::default_preferences()
            }
        } else {
            VoicePreferences::default_preferences()
        }
    };

    let style = SpeechStyleEngine::determine_style(emotion, preferences.personality_mode);
    let formatted_text = SpeechStyleEngine::format_text_style(text, &style);

    // 2. Resolve Piper binary and model paths
    let resolved_bin_path = crate::resolve_path("bin\\piper\\piper.exe", "e:\\ALOK PC\\bin\\piper\\piper.exe");
    let bin_path = Path::new(&resolved_bin_path);
    let model_dir = crate::resolve_path("models\\piper", "e:\\ALOK PC\\models\\piper");
    
    let voice_file = if voice.ends_with(".onnx") {
        voice.to_string()
    } else {
        format!("{}.onnx", voice)
    };
    let model_path = Path::new(&model_dir).join(&voice_file);
    let config_path = Path::new(&model_dir).join(format!("{}.json", voice_file));

    // Check if Piper is available locally
    let piper_available = bin_path.exists() && model_path.exists() && config_path.exists();

    // Create unique temp WAV file path
    let count = TTS_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let temp_wav = std::env::temp_dir().join(format!("alok_piper_tts_{}.wav", count));
    let temp_wav_str = temp_wav.to_string_lossy().to_string();

    if piper_available {
        // Calculate length scale: length_scale = 1.0 / (rate_modifier * speed)
        let rate_modifier = style.speed * speed;
        let length_scale = 1.0 / rate_modifier.clamp(0.5, 2.0);
        
        // Map generic parameters to Piper noise scales
        // Higher energy -> larger phoneme pronunciation variance
        let noise_scale = 0.667 + (style.energy - 0.5) * 0.2;
        // Faster speed -> smaller phoneme duration variance
        let noise_w = 0.8 - (style.speed - 1.0) * 0.1;

        println!("TTS (Piper): Executing subprocess text='{}', voice={}, length_scale={:.3}", formatted_text, voice_file, length_scale);

        let mut cmd = Command::new(&bin_path);
        cmd.args([
            "--model", &model_path.to_string_lossy().to_string(),
            "--config", &config_path.to_string_lossy().to_string(),
            "--output_file", &temp_wav_str,
            "--length_scale", &format!("{:.3}", length_scale),
            "--noise_scale", &format!("{:.3}", noise_scale),
            "--noise_w", &format!("{:.3}", noise_w),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Write formatted text to piper's stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(formatted_text.as_bytes())?;
        }

        // Store child in state to allow interruption
        {
            let mut lock = state_ref.lock().unwrap();
            if lock.currently_playing {
                lock.current_child = Some(child);
            } else {
                let _ = child.kill();
                return Ok(());
            }
        }

        // Wait for child to exit
        let status = {
            let mut lock = state_ref.lock().unwrap();
            if let Some(mut child) = lock.current_child.take() {
                drop(lock);
                child.wait()?
            } else {
                return Ok(());
            }
        };

        if !status.success() {
            return Err("Piper TTS subprocess returned failure status".into());
        }
    } else {
        // Fallback to Windows native speech synthesis
        println!("TTS (Piper): Local Piper binaries or models not found. Falling back to Windows native speech synthesis.");
        generate_speech_windows(&formatted_text, style.speed * speed, &temp_wav_str)?;
    }

    // 3. Play WAV natively using winmm PlaySoundW
    unsafe {
        #[link(name = "winmm")]
        extern "system" {
            fn PlaySoundW(pszSound: *const u16, hmod: isize, fdwSound: u32) -> i32;
        }

        // Check if we were stopped before we play
        {
            let lock = state_ref.lock().unwrap();
            if !lock.currently_playing {
                let _ = std::fs::remove_file(&temp_wav);
                return Ok(());
            }
        }

        // Load WAV into memory and delete the file immediately
        let wav_bytes = std::fs::read(&temp_wav).unwrap_or_default();
        let _ = std::fs::remove_file(&temp_wav);

        if !wav_bytes.is_empty() {
            // SND_MEMORY = 0x0004, SND_NODEFAULT = 0x0002, SND_ASYNC = 0x0001
            let flags = 0x0004 | 0x0002 | 0x0001;

            let bytes_ptr = {
                let mut lock = state_ref.lock().unwrap();
                if !lock.currently_playing {
                    return Ok(());
                }
                lock.active_wav_bytes = Some(wav_bytes);
                lock.active_wav_bytes.as_ref().unwrap().as_ptr()
            };

            PlaySoundW(bytes_ptr as *const u16, 0, flags);

            // Wait for it to finish playing or be interrupted
            let duration_ms = {
                let lock = state_ref.lock().unwrap();
                if let Some(ref b) = lock.active_wav_bytes {
                    get_wav_duration_ms_bytes(b)
                } else {
                    0
                }
            };
            let start = std::time::Instant::now();
            while start.elapsed().as_millis() < duration_ms as u128 {
                std::thread::sleep(std::time::Duration::from_millis(50));
                // Check if we were interrupted/stopped
                {
                    let lock = state_ref.lock().unwrap();
                    if !lock.currently_playing {
                        // Purge Win32 PlaySound immediately
                        PlaySoundW(std::ptr::null(), 0, 0x0040); // SND_PURGE
                        break;
                    }
                }
            }

            // Final purge to release internal references
            PlaySoundW(std::ptr::null(), 0, 0x0040);

            // Keep the bytes alive for a tiny bit longer to ensure Windows thread has fully released it
            std::thread::sleep(std::time::Duration::from_millis(50));
            {
                let mut lock = state_ref.lock().unwrap();
                lock.active_wav_bytes = None;
            }
        }
    }

    Ok(())
}

fn segment_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    
    let words = text.split_whitespace();
    for word in words {
        current.push_str(word);
        current.push(' ');
        
        let is_terminal = (word.ends_with('.') && !is_abbreviation(word))
            || word.ends_with('?')
            || word.ends_with('!');
            
        if is_terminal {
            let s = current.trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
            current.clear();
        }
    }
    let s = current.trim().to_string();
    if !s.is_empty() {
        sentences.push(s);
    }
    sentences
}

fn is_abbreviation(word: &str) -> bool {
    let w = word.to_lowercase();
    let abbrev = ["mr.", "ms.", "dr.", "prof.", "sr.", "jr.", "e.g.", "i.e.", "vs.", "al."];
    abbrev.iter().any(|&a| w.ends_with(a))
}

fn generate_speech_windows(text: &str, speed: f32, output_wav: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let synth = SpeechSynthesizer::new()?;
    
    let escaped_text = text
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;");

    let rate = speed.clamp(0.5, 2.0);
    let pitch_str = "+0%";

    let ssml = format!(
        r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="en-US"><prosody rate="{:.2}" pitch="{}">{}</prosody></speak>"#,
        rate, pitch_str, escaped_text
    );

    let stream = synth.SynthesizeSsmlToStreamAsync(&HSTRING::from(&ssml))?.get()?;
    
    let size = stream.Size()? as usize;
    let input_stream = stream.GetInputStreamAt(0)?;
    let reader = DataReader::CreateDataReader(&input_stream)?;
    reader.LoadAsync(size as u32)?.get()?;
    
    let mut buffer = vec![0u8; size];
    reader.ReadBytes(&mut buffer)?;
    
    let mut file = File::create(output_wav)?;
    file.write_all(&buffer)?;
    Ok(())
}
