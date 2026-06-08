use crate::vision::ui_parser::UiParser;

pub struct ScreenAnalysis {
    pub raw_ocr: String,
    pub title: Option<String>,
    pub code_snippet: Option<String>,
    pub error_traceback: Option<String>,
    pub body_text: String,
}

pub struct ScreenAnalyzer;

impl ScreenAnalyzer {
    pub fn analyze(ocr_text: &str) -> ScreenAnalysis {
        let blocks = UiParser::parse_raw_ocr(ocr_text);

        let mut title = None;
        let mut code_blocks = Vec::new();
        let mut error_blocks = Vec::new();
        let mut text_blocks = Vec::new();

        for block in blocks {
            match block.block_type.as_str() {
                "title" => {
                    if title.is_none() {
                        title = Some(block.content);
                    } else {
                        text_blocks.push(block.content);
                    }
                }
                "code" => {
                    code_blocks.push(block.content);
                }
                "error" => {
                    error_blocks.push(block.content);
                }
                _ => {
                    text_blocks.push(block.content);
                }
            }
        }

        let code_snippet = if !code_blocks.is_empty() {
            // Pick the longest block of code (most likely the active editor code)
            code_blocks.sort_by_key(|b| b.len());
            code_blocks.last().cloned()
        } else {
            None
        };

        let error_traceback = if !error_blocks.is_empty() {
            Some(error_blocks.join("\n"))
        } else {
            // Fallback: search general text for error patterns if no error-block was classified
            let text_full = text_blocks.join("\n");
            let mut found_errors = Vec::new();
            for line in text_full.lines() {
                if line.to_lowercase().contains("error") 
                   || line.to_lowercase().contains("exception")
                   || line.to_lowercase().contains("failed") {
                    found_errors.push(line.trim().to_string());
                }
            }
            if !found_errors.is_empty() {
                Some(found_errors.join("\n"))
            } else {
                None
            }
        };

        ScreenAnalysis {
            raw_ocr: ocr_text.to_string(),
            title,
            code_snippet,
            error_traceback,
            body_text: text_blocks.join("\n"),
        }
    }
}
