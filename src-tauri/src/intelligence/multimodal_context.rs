use crate::AppState;
use crate::environment_awareness;
use crate::vision::VisionEngine;

pub struct MultimodalContextEngine;

impl MultimodalContextEngine {
    /// Captures all voice, screen, clipboard, browser, active window, and file context,
    /// and formats it into a structured prompt block for the online reasoning LLM.
    pub fn get_enriched_prompt(text: &str, state: &AppState) -> String {
        // 1. Capture OS & window context
        let env = environment_awareness::capture();
        
        // 2. Capture Screen visual text context
        let ocr_text = VisionEngine::capture_screen_text();
        
        // 3. Construct description details
        let active_window = format!("{} (Title: \"{}\")", env.active_window.process_name, env.active_window.title);
        
        let browser = if env.browser.is_browser {
            format!(
                "Browser: {}, URL: {}, Title: \"{}\"",
                env.browser.browser_name.as_deref().unwrap_or("Unknown"),
                env.browser.url.as_deref().unwrap_or("None"),
                env.browser.title.as_deref().unwrap_or("None")
            )
        } else {
            "None (Active window is not a web browser)".to_string()
        };

        let clipboard = format!(
            "Selected text: \"{}\", Clipboard contents: \"{}\"",
            env.clipboard.selected_text.as_deref().unwrap_or(""),
            env.clipboard.clipboard_text.as_deref().unwrap_or("")
        );

        let file = format!(
            "Active file: \"{}\", Active folder: \"{}\"",
            env.file.active_file.as_deref().unwrap_or("None"),
            env.file.active_folder.as_deref().unwrap_or("None")
        );

        // 4. Construct unified context block
        let context_block = format!(
            "[MULTIMODAL CONTEXT STATE]\n\
             - Active Window: {}\n\
             - Browser Context: {}\n\
             - File Context: {}\n\
             - Clipboard Context: {}\n\
             - Visible Screen Content (OCR):\n\
             \"\"\"\n\
             {}\n\
             \"\"\"\n\
             [END CONTEXT STATE]",
            active_window, browser, file, clipboard, ocr_text
        );

        // 5. Wrap prompt nicely
        format!(
            "You are ALOK OS, a premium personal assistant. Respond to the user's request. \
             Utilize the [MULTIMODAL CONTEXT STATE] below to resolve references like 'this', 'it', 'that', 'this page', 'this error', 'this code', or 'what is on my screen'. \
             Do not request clarification if the context contains the answer.\n\n\
             {}\n\n\
             User request: '{}'",
            context_block,
            text
        )
    }
}
