use crate::database::Database;
use crate::intelligence::groq::GroqClient;
use crate::vision::screen_analyzer::ScreenAnalyzer;

pub struct VisionEngine;

impl VisionEngine {
    pub fn capture_screen_text() -> String {
        if let Some(cached) = crate::vision::ocr_cache::get_cached_ocr() {
            return cached;
        }
        let engine = crate::screen_context_engine::ScreenContextEngine::new();
        let text = engine.capture_and_ocr().unwrap_or_default();
        crate::vision::ocr_cache::set_cached_ocr(&text);
        text
    }

    pub fn analyze_error(db: &Database) -> String {
        let ocr = Self::capture_screen_text();
        if ocr.trim().is_empty() {
            return "I took a screenshot but couldn't detect any text on your screen. Please make sure the window containing the text is in the foreground.".to_string();
        }

        let analysis = ScreenAnalyzer::analyze(&ocr);
        
        let api_key = db.get_setting("groq_api_key").ok().flatten();
        let model = db.get_setting("groq_model").ok().flatten().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        
        if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            return format!(
                "Vision Intelligence: Detected compiler/runtime error on screen:\n\n{}\n\n(Groq API Key is not configured in settings to provide detailed solutions)",
                analysis.error_traceback.unwrap_or_else(|| "No specific error traceback isolated.".to_string())
            );
        }

        let client = GroqClient::new(api_key, &model);
        
        let prompt = match &analysis.error_traceback {
            Some(err) => format!(
                "You are ALOK OS, a premium developer presence assistant. The user's screen contains the following compilation or runtime error. Explain the root cause of this error and suggest clear, step-by-step code fixes or shell commands to resolve it. Be concise, highly professional, and developer-focused.\n\nError traceback:\n{}",
                err
            ),
            None => format!(
                "The user asked to explain an error on their screen. Inspect the following raw OCR text taken from the screen. If you find any compiler, terminal, runtime, or system error, identify it, explain it, and suggest a fix. If there are no errors, notify the user nicely. Be concise and professional.\n\nScreen OCR text:\n{}",
                analysis.raw_ocr
            )
        };

        match client.query(&prompt) {
            Ok(reply) => reply,
            Err(e) => format!("Vision Intelligence: Failed to consult Groq reasoning: {:?}", e),
        }
    }

    pub fn explain_code(db: &Database) -> String {
        let ocr = Self::capture_screen_text();
        if ocr.trim().is_empty() {
            return "I couldn't detect any text on your screen to extract code from. Please verify the code editor is in the foreground.".to_string();
        }

        let analysis = ScreenAnalyzer::analyze(&ocr);
        
        let api_key = db.get_setting("groq_api_key").ok().flatten();
        let model = db.get_setting("groq_model").ok().flatten().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        
        if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            return format!(
                "Vision Intelligence: Extracted visible code:\n\n{}\n\n(Groq API Key is not configured to provide code explanations)",
                analysis.code_snippet.unwrap_or_else(|| "No code snippet block isolated.".to_string())
            );
        }

        let client = GroqClient::new(api_key, &model);
        
        let prompt = match &analysis.code_snippet {
            Some(code) => format!(
                "You are ALOK OS. Explain the logic, design patterns, and potential optimizations of the following code snippet visible on the user's screen. Keep it concise, developer-focused, and direct.\n\nCode snippet:\n{}",
                code
            ),
            None => format!(
                "The user asked to explain code on their screen. Inspect the following raw OCR text taken from the screen. Extract any visible code logic or statements and explain them. If no code is visible, let the user know. Be concise and professional.\n\nScreen OCR text:\n{}",
                analysis.raw_ocr
            )
        };

        match client.query(&prompt) {
            Ok(reply) => reply,
            Err(e) => format!("Vision Intelligence: Failed to consult Groq reasoning: {:?}", e),
        }
    }

    pub fn summarize_screen(db: &Database) -> String {
        let ocr = Self::capture_screen_text();
        if ocr.trim().is_empty() {
            return "I took a screenshot but found no text. Your screen appears to be empty or showing non-text content.".to_string();
        }

        let api_key = db.get_setting("groq_api_key").ok().flatten();
        let model = db.get_setting("groq_model").ok().flatten().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());
        
        if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            return "Vision Intelligence: Screen text detected, but Groq API Key is not configured to generate a natural summary.".to_string();
        }

        let client = GroqClient::new(api_key, &model);
        let prompt = format!(
            "Summarize what the user is working on based on the raw text visible on their screen. Be concise, professional, and natural (e.g. 'You are working on a Rust project in VS Code...'). Do not list code line-by-line.\n\nScreen OCR text:\n{}",
            ocr
        );

        match client.query(&prompt) {
            Ok(reply) => reply,
            Err(e) => format!("Vision Intelligence: Failed to generate screen summary: {:?}", e),
        }
    }
}
