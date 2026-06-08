#[derive(Debug, Clone)]
pub struct UiBlock {
    pub block_type: String, // "title", "code", "text", "error"
    pub content: String,
}

pub struct UiParser;

impl UiParser {
    pub fn parse_raw_ocr(ocr_text: &str) -> Vec<UiBlock> {
        let mut blocks = Vec::new();
        let mut current_code = Vec::new();
        let mut current_text = Vec::new();

        for line in ocr_text.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.is_empty() {
                continue;
            }

            // Detect code lines by syntax keywords/characters
            let is_code_line = line_trimmed.contains(";") 
                || line_trimmed.contains("{") 
                || line_trimmed.contains("}")
                || line_trimmed.contains("=>")
                || line_trimmed.contains("import ")
                || line_trimmed.contains("pub ")
                || line_trimmed.contains("fn ")
                || line_trimmed.contains("const ")
                || line_trimmed.contains("let ")
                || line_trimmed.contains("class ")
                || line_trimmed.contains("return ")
                || line_trimmed.starts_with("#include")
                || line_trimmed.starts_with("def ")
                || line_trimmed.starts_with("package ");

            // Detect exception/error keywords
            let is_error_line = line_trimmed.to_lowercase().contains("exception")
                || line_trimmed.to_lowercase().contains("error:")
                || line_trimmed.to_lowercase().contains("failed:")
                || line_trimmed.to_lowercase().contains("traceback")
                || line_trimmed.to_lowercase().contains("stack trace")
                || line_trimmed.to_lowercase().contains("unhandled");

            if is_error_line {
                // Flush other lists first
                Self::flush_blocks(&mut blocks, &mut current_code, &mut current_text);
                blocks.push(UiBlock {
                    block_type: "error".to_string(),
                    content: line_trimmed.to_string(),
                });
            } else if is_code_line {
                // Flush general text
                if !current_text.is_empty() {
                    blocks.push(UiBlock {
                        block_type: "text".to_string(),
                        content: current_text.join("\n"),
                    });
                    current_text.clear();
                }
                current_code.push(line_trimmed.to_string());
            } else {
                // Flush code block
                if !current_code.is_empty() {
                    blocks.push(UiBlock {
                        block_type: "code".to_string(),
                        content: current_code.join("\n"),
                    });
                    current_code.clear();
                }
                
                // Check if it's a short UPPERCASE title
                if line_trimmed.len() < 50 && line_trimmed.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
                    Self::flush_blocks(&mut blocks, &mut current_code, &mut current_text);
                    blocks.push(UiBlock {
                        block_type: "title".to_string(),
                        content: line_trimmed.to_string(),
                    });
                } else {
                    current_text.push(line_trimmed.to_string());
                }
            }
        }

        Self::flush_blocks(&mut blocks, &mut current_code, &mut current_text);
        blocks
    }

    fn flush_blocks(blocks: &mut Vec<UiBlock>, code: &mut Vec<String>, text: &mut Vec<String>) {
        if !code.is_empty() {
            blocks.push(UiBlock {
                block_type: "code".to_string(),
                content: code.join("\n"),
            });
            code.clear();
        }
        if !text.is_empty() {
            blocks.push(UiBlock {
                block_type: "text".to_string(),
                content: text.join("\n"),
            });
            text.clear();
        }
    }
}
