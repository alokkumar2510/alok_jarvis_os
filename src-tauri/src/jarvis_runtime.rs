use tauri::{AppHandle, Manager, Emitter, State};
use crate::AppState;
use crate::voice::tts;

pub struct JarvisRuntime;

impl JarvisRuntime {
    /// Coordinates the full JARVIS pipeline:
    /// Wake Word -> Voice Recording -> Whisper STT -> Intent Parsing -> Entity Extraction -> Conversation Context -> Action Bus -> Execution -> Overlay Update -> TTS Response
    pub fn handle_trigger(app: &AppHandle, wav_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Err(e) = Self::run_pipeline(app, wav_path) {
            eprintln!("JarvisRuntime: Pipeline error: {}", e);
            crate::log_dev_event(app, "Voice", "error", &format!("Pipeline failed: {}", e));
            
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(mut vs) = state.voice_state.lock() {
                    *vs = crate::voice::VoiceState::Idle;
                }
            }

            // Notify UI of error and reset to idle
            let _ = app.emit("voice-state-change", serde_json::json!({
                "state": "idle",
                "transcript": "",
                "message": format!("Error: {}", e)
            }));

            // Also notify dialogue event
            let _ = app.emit("dialogue-event", serde_json::json!({
                "sender": "Alok",
                "message": format!("Error: {}", e)
            }));

            // Speak the error
            let _ = tts::speak(&format!("Error: {}", e));

            return Err(e);
        }
        Ok(())
    }

    fn run_pipeline(app: &AppHandle, wav_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state: State<'_, AppState> = app.state();

        // Update overlay
        let _ = app.emit("voice-state-change", serde_json::json!({
            "state": "processing",
            "transcript": "",
            "message": "Transcribing..."
        }));

        // 3. Whisper STT
        println!("JarvisRuntime: Invoking Whisper STT...");
        crate::log_dev_event(app, "Voice", "info", "Transcribing WAV file via Whisper STT...");
        let start_stt = std::time::Instant::now();
        let transcript = state.whisper.transcribe(wav_path)?;
        let stt_dur = start_stt.elapsed().as_millis();
        
        if transcript.trim().is_empty() {
            println!("JarvisRuntime: Empty transcript. Resetting and resuming suspended speech if any.");
            crate::log_dev_event(app, "Voice", "warn", "Whisper STT returned an empty transcript. Resuming suspended speech.");
            
            // Try to resume suspended speech!
            if let Err(e) = crate::voice::tts::resume() {
                eprintln!("JarvisRuntime: Failed to resume speech on empty transcript: {}", e);
            }

            if let Ok(mut vs) = state.voice_state.lock() {
                *vs = crate::voice::VoiceState::Idle;
            }
            let _ = app.emit("voice-state-change", serde_json::json!({
                "state": "idle",
                "transcript": "",
                "message": ""
            }));
            return Ok(());
        }

        crate::log_dev_event(app, "Voice", "info", &format!("Whisper STT finished in {} ms. Transcript: '{}'", stt_dur, transcript));
        println!("JarvisRuntime: Transcribed text = '{}'", transcript);

        // Extract voice characteristics from recorded WAV
        let voice_char = crate::voice::voice_manager::VoiceManager::extract_voice_characteristics(wav_path, &transcript);
        if let Some(ref vc) = voice_char {
            println!("JarvisRuntime: Extracted voice characteristics: Pitch={:.1}Hz, Energy={:.3}, Rate={:.2} words/sec", vc.pitch, vc.energy, vc.speaking_rate);
            crate::log_dev_event(app, "Voice", "info", &format!("Voice metrics - Pitch: {:.1}Hz, Energy: {:.3}, Rate: {:.1} words/s", vc.pitch, vc.energy, vc.speaking_rate));
        }

        // Log speech recognition latency
        if let Ok(db_lock) = state.db.lock() {
            crate::intelligence::performance_tracker::log_latency(&db_lock, "speech_recognition", stt_dur as f64);
        }

        // Update overlay with User transcript
        let _ = app.emit("voice-state-change", serde_json::json!({
            "state": "processing",
            "transcript": transcript.clone(),
            "message": "Executing command..."
        }));

        let _ = app.emit("dialogue-event", serde_json::json!({
            "sender": "User",
            "message": transcript.clone()
        }));

        // Log command
        if let Ok(mut ctx) = state.context.lock() {
            ctx.add_history("User", &transcript);
        }
        if let Ok(db) = state.db.lock() {
            let _ = db.add_conversation_entry("User", &transcript);
        }

        // 4. Environment Awareness Context Capture
        let env_context = crate::environment_awareness::capture();
        println!("JarvisRuntime: Captured environment: {:?}", env_context);
        if let Ok(mut ctx) = state.context.lock() {
            ctx.update_environment_context(&env_context);
        }

        // 5. Conversation Context pronoun reference resolution
        let resolved_text = {
            if let Ok(ctx) = state.context.lock() {
                let resolved = ctx.resolve_text(&transcript);
                if resolved != transcript {
                    crate::log_dev_event(app, "Intent", "info", &format!("Context reference resolved: '{}' -> '{}'", transcript, resolved));
                }
                resolved
            } else {
                transcript.clone()
            }
        };

        // 5. Execute Command (handles single and chained commands, updates context, logs dialogue, plays TTS)
        let outcome = tauri::async_runtime::block_on(async {
            crate::execute_resolved_command(resolved_text, voice_char, &state, app).await
        });

        // Log action execution latency
        if let Ok(db_lock) = state.db.lock() {
            crate::intelligence::performance_tracker::log_latency(&db_lock, "action_execution", outcome.action_latency_ms as f64);
        }

        // Asynchronously trigger autonomous memory consolidation
        let app_clone = app.clone();
        let transcript_clone = transcript.clone();
        let response_clone = outcome.output.clone();
        tauri::async_runtime::spawn(async move {
            let _ = crate::intelligence::memory_engine::consolidate_turn(app_clone, transcript_clone, response_clone).await;
        });

        // Emit a comprehensive performance metrics snapshot to the console
        let memories_count = if let Ok(db) = state.db.lock() {
            db.search_memories("").map_or(0, |v| v.len())
        } else {
            0
        };

        let _ = app.emit("dev-perf", serde_json::json!({
            "state": "idle",
            "whisper_latency": stt_dur,
            "intent_score": outcome.intent_score,
            "action_latency": outcome.action_latency_ms,
            "memories_count": memories_count
        }));

        // 9. Reset voice loop back to idle or speaking
        let has_speech = !outcome.output.is_empty() && outcome.output != "Resuming speech";
        if has_speech {
            if let Ok(mut vs) = state.voice_state.lock() {
                *vs = crate::voice::VoiceState::Speaking;
            }
            let _ = app.emit("voice-state-change", serde_json::json!({
                "state": "speaking",
                "transcript": "",
                "message": outcome.output.clone()
            }));
        } else {
            if let Ok(mut vs) = state.voice_state.lock() {
                *vs = crate::voice::VoiceState::Idle;
            }
            let _ = app.emit("voice-state-change", serde_json::json!({
                "state": "idle",
                "transcript": "",
                "message": ""
            }));
        }

        Ok(())
    }
}
