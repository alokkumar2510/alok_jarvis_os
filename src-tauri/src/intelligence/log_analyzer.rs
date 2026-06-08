#[derive(Debug, Clone)]
pub struct ParsedError {
    pub language: String,
    pub error_type: String,
    pub description: String,
    pub file_location: Option<String>,
}

/// Identifies stack trace patterns or compiler diagnostic headers to categorize errors
pub fn parse_log(log_text: &str) -> Option<ParsedError> {
    let log_lower = log_text.to_lowercase();

    // 1. Rust Compiler Diagnostic Outputs
    if log_lower.contains("error[e") || log_lower.contains("cargo build") || log_lower.contains("rustc") {
        let mut error_type = "rust_compile_error".to_string();
        let mut description = "Rust compilation failure".to_string();
        let mut file_location = None;

        for line in log_text.lines() {
            if line.contains("error[E") {
                if let Some(idx) = line.find("error[E") {
                    error_type = line[idx..idx + 12].trim_end_matches(':').to_string();
                }
                description = line.to_string();
            }
            if line.contains("--> ") {
                file_location = Some(line.replace("--> ", "").trim().to_string());
            }
        }
        return Some(ParsedError {
            language: "Rust".to_string(),
            error_type,
            description,
            file_location,
        });
    }

    // 2. Python Exception Tracebacks
    if log_lower.contains("traceback (most recent call last)") || log_lower.contains("exception") || log_lower.contains("file \"") {
        let mut error_type = "python_exception".to_string();
        let mut description = "Python runtime exception".to_string();
        let mut file_location = None;

        let lines: Vec<&str> = log_text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("  File \"") {
                file_location = Some(line.trim().to_string());
            }
            if i == lines.len() - 1 && line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                error_type = parts[0].trim().to_string();
                description = parts[1].trim().to_string();
            }
        }
        return Some(ParsedError {
            language: "Python".to_string(),
            error_type,
            description,
            file_location,
        });
    }

    // 3. Node/JavaScript Runtime Stack Traces
    if log_lower.contains("node_modules") || log_lower.contains("error:") || log_lower.contains("at ") {
        let mut error_type = "js_exception".to_string();
        let mut description = "JavaScript exception".to_string();
        let mut file_location = None;

        for line in log_text.lines() {
            if line.contains("Error:") || line.contains("TypeError:") || line.contains("ReferenceError:") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                error_type = parts[0].trim().to_string();
                description = parts.get(1).unwrap_or(&"").trim().to_string();
            }
            if line.trim().starts_with("at ") && file_location.is_none() {
                file_location = Some(line.replace("at ", "").trim().to_string());
            }
        }
        return Some(ParsedError {
            language: "JavaScript/TypeScript".to_string(),
            error_type,
            description,
            file_location,
        });
    }

    // 4. Flutter/Dart Doctor and Build Failures
    if log_lower.contains("flutter doctor") || log_lower.contains("dart:") || log_lower.contains("flutter:") {
        return Some(ParsedError {
            language: "Flutter/Dart".to_string(),
            error_type: "flutter_error".to_string(),
            description: log_text.lines().next().unwrap_or("Flutter execution error").to_string(),
            file_location: None,
        });
    }

    None
}
