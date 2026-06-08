use crate::environment_awareness::active_window::ActiveWindowInfo;
use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInfo {
    pub is_browser: bool,
    pub browser_name: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
}

pub fn capture(active: &ActiveWindowInfo) -> BrowserInfo {
    let proc_lower = active.process_name.to_lowercase();
    let is_chrome = proc_lower.contains("chrome");
    let is_edge = proc_lower.contains("msedge");
    let is_firefox = proc_lower.contains("firefox");
    let is_brave = proc_lower.contains("brave");
    
    let is_browser = is_chrome || is_edge || is_firefox || is_brave;
    
    if !is_browser {
        return BrowserInfo {
            is_browser: false,
            browser_name: None,
            url: None,
            title: None,
        };
    }

    let browser_name = Some(active.process_name.clone());
    let title = Some(active.title.clone());

    // Extract URL via UI Automation (via PowerShell script helper)
    let url = get_url_via_powershell(active.hwnd, &active.process_name);

    BrowserInfo {
        is_browser: true,
        browser_name,
        url,
        title,
    }
}

fn get_url_via_powershell(_hwnd: isize, _process_name: &str) -> Option<String> {
    // Avoid spawning PowerShell in background loop to prevent MinGW thread/process corruption and lag.
    None
}
