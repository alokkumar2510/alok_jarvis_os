use std::process::Command;
use std::path::Path;

pub struct WhisperEngine {
    bin_path: String,
}

impl WhisperEngine {
    pub fn new() -> Self {
        let bin_path = "e:\\ALOK PC\\bin\\whisper-cli.exe".to_string();
        Self { bin_path }
    }

    /// Transcribes a 16kHz mono WAV file using the local Whisper subprocess.
    pub fn transcribe(&self, wav_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let base_model = "e:\\ALOK PC\\models\\ggml-base.bin";
        let tiny_model = "e:\\ALOK PC\\models\\ggml-tiny.bin";
        
        let resolved_model = if Path::new(base_model).exists() {
            base_model.to_string()
        } else {
            tiny_model.to_string()
        };

        if !Path::new(&self.bin_path).exists() {
            return Err(format!("Whisper executable not found at {}", self.bin_path).into());
        }
        if !Path::new(&resolved_model).exists() {
            return Err(format!("Whisper model file not found at {}", resolved_model).into());
        }

        println!(
            "WhisperEngine: Transcribing {} using model {}...",
            wav_path, resolved_model
        );

        // Run whisper-cli subprocess
        let output = Command::new(&self.bin_path)
            .args([
                "-m", &resolved_model,
                "-f", wav_path,
                "--no-timestamps",
                "-nt", // no text tokens/timestamps
                "-np"  // do not print anything other than the results
            ])
            .output()?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Whisper execution failed: {}", err_msg).into());
        }

        let transcribed_text = String::from_utf8_lossy(&output.stdout);
        let cleaned_text = self.clean_transcription(&transcribed_text);
        
        println!("WhisperEngine: Transcription result = '{}'", cleaned_text);
        Ok(cleaned_text)
    }

    fn clean_transcription(&self, text: &str) -> String {
        // Remove trailing newlines and standard whisper CLI metadata headers
        text.lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with("whisper_") && !line.contains("system_info"))
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string()
    }
}
