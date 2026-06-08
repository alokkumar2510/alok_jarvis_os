use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppCategory {
    Browser,
    Editor,
    Terminal,
    FileExplorer,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContextInfo {
    pub category: AppCategory,
    pub process_name: String,
}

pub fn capture(process_name: &str) -> AppContextInfo {
    let proc_lower = process_name.to_lowercase();
    let category = if proc_lower.contains("chrome") || proc_lower.contains("msedge") || proc_lower.contains("firefox") || proc_lower.contains("brave") {
        AppCategory::Browser
    } else if proc_lower.contains("code") || proc_lower.contains("notepad") || proc_lower.contains("sublime") || proc_lower.contains("devenv") || proc_lower.contains("cursor") {
        AppCategory::Editor
    } else if proc_lower.contains("cmd") || proc_lower.contains("powershell") || proc_lower.contains("wt") || proc_lower.contains("bash") {
        AppCategory::Terminal
    } else if proc_lower == "explorer.exe" {
        AppCategory::FileExplorer
    } else {
        AppCategory::Other
    };

    AppContextInfo {
        category,
        process_name: process_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_categorization() {
        assert_eq!(capture("chrome.exe").category, AppCategory::Browser);
        assert_eq!(capture("msedge.exe").category, AppCategory::Browser);
        assert_eq!(capture("code.exe").category, AppCategory::Editor);
        assert_eq!(capture("notepad.exe").category, AppCategory::Editor);
        assert_eq!(capture("cmd.exe").category, AppCategory::Terminal);
        assert_eq!(capture("powershell.exe").category, AppCategory::Terminal);
        assert_eq!(capture("explorer.exe").category, AppCategory::FileExplorer);
        assert_eq!(capture("calc.exe").category, AppCategory::Other);
    }
}
