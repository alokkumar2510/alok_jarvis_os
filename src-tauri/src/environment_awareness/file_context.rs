use crate::environment_awareness::active_window::ActiveWindowInfo;
use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContextInfo {
    pub active_folder: Option<String>,
    pub active_file: Option<String>,
}

pub fn capture(active: &ActiveWindowInfo) -> FileContextInfo {
    let proc_lower = active.process_name.to_lowercase();
    
    let mut active_folder = None;
    let mut active_file = None;

    if proc_lower == "explorer.exe" {
        active_folder = get_explorer_path(active.hwnd);
    } else {
        active_file = parse_file_from_title(&active.title);
    }

    FileContextInfo {
        active_folder,
        active_file,
    }
}

fn get_explorer_path(_hwnd: isize) -> Option<String> {
    // Bypass PowerShell process invocation to prevent background threading hangs/crashes.
    None
}

fn parse_file_from_title(title: &str) -> Option<String> {
    let extensions = [
        "rs", "js", "ts", "py", "html", "css", "json", "toml", "md", "txt", "pdf", "docx", "xlsx", "cpp", "c", "h", "java", "go", "sh", "bat", "ps1"
    ];
    
    // Split title by typical separators like spaces, dashes, slashes, backslashes, pipes
    let parts: Vec<&str> = title.split(|c: char| c == ' ' || c == '-' || c == '|' || c == '/' || c == '\\' || c == '•').collect();
    
    for part in parts {
        let part_trimmed = part.trim();
        if let Some(dot_idx) = part_trimmed.rfind('.') {
            let ext = &part_trimmed[dot_idx + 1..];
            if extensions.contains(&ext.to_lowercase().as_str()) {
                if part_trimmed.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') {
                    return Some(part_trimmed.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_from_title() {
        assert_eq!(
            parse_file_from_title("active_window.rs - alok_jarvis_os - Visual Studio Code"),
            Some("active_window.rs".to_string())
        );
        assert_eq!(
            parse_file_from_title("main.js - Notepad"),
            Some("main.js".to_string())
        );
        assert_eq!(
            parse_file_from_title("E:\\ALOK PC\\alok_jarvis_os\\src-tauri\\src\\environment_awareness\\file_context.rs"),
            Some("file_context.rs".to_string())
        );
        assert_eq!(
            parse_file_from_title("https://google.com"),
            None
        );
    }
}
