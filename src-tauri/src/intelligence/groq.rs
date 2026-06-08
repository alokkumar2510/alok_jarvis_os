use std::process::Command;
use serde_json::Value;

pub struct GroqClient {
    api_key: Option<String>,
    model: String,
}

impl GroqClient {
    pub fn new(api_key: Option<String>, model: &str) -> Self {
        let key = if api_key.is_some() {
            api_key
        } else {
            // Fallback to reading from .env file or environment variable
            std::env::var("GROQ_API_KEY").ok()
        };

        Self {
            api_key: key,
            model: model.to_string(),
        }
    }

    /// Queries Groq API for general conversation using curl.exe to avoid bulky native dependencies.
    pub fn query(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let key = match &self.api_key {
            Some(k) if !k.is_empty() => k,
            _ => return Err("Groq API Key is not configured. Please add it in settings or a .env file.".into()),
        };

        println!("GroqClient: Querying cloud model '{}'...", self.model);

        // Construct JSON payload robustly using serde_json
        let payload_json = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });
        let payload = serde_json::to_string(&payload_json)?;

        // Call system curl.exe (available natively on Win 10/11)
        let output = Command::new("curl.exe")
            .args([
                "-X", "POST",
                "https://api.groq.com/openai/v1/chat/completions",
                "-H", &format!("Authorization: Bearer {}", key),
                "-H", "Content-Type: application/json",
                "-d", &payload
            ])
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("curl failed: {}", err).into());
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let response_json: Value = serde_json::from_str(&stdout_str).map_err(|e| {
            format!("Failed to parse Groq JSON response: {}. Raw: {}", e, &stdout_str[..stdout_str.len().min(500)])
        })?;

        // Check for API-level error in the response body
        if let Some(err_obj) = response_json.get("error") {
            let err_msg = err_obj["message"].as_str().unwrap_or("Unknown Groq API error");
            return Err(format!("Groq API error: {}", err_msg).into());
        }

        let text = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| format!("Failed to extract completion text. Raw response: {}", &stdout_str[..stdout_str.len().min(500)]))?;

        Ok(text.to_string())
    }
}
