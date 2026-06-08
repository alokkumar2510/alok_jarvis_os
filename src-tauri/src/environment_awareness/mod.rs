pub mod active_window;
pub mod browser_context;
pub mod clipboard_context;
pub mod file_context;
pub mod app_context;
pub mod browser_context_engine;

pub use active_window::ActiveWindowInfo;
pub use browser_context::BrowserInfo;
pub use clipboard_context::ClipboardInfo;
pub use file_context::FileContextInfo as FileInfo;
pub use app_context::AppContextInfo;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub active_window: ActiveWindowInfo,
    pub browser: BrowserInfo,
    pub clipboard: ClipboardInfo,
    pub file: FileInfo,
    pub app: AppContextInfo,
}

pub fn capture() -> EnvironmentContext {
    let active_window = active_window::capture();
    let app = app_context::capture(&active_window.process_name);
    let browser = browser_context::capture(&active_window);
    let clipboard = clipboard_context::capture();
    let file = file_context::capture(&active_window);

    EnvironmentContext {
        active_window,
        browser,
        clipboard,
        file,
        app,
    }
}
