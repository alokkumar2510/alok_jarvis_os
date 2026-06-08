use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use crate::database::Database;
use crate::intelligence::groq::GroqClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub browser: String,
    pub url: String,
    pub title: String,
    pub visit_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub browser: String,
    pub title: String,
}

pub struct BrowserContextEngine;

impl BrowserContextEngine {
    /// Queries yesterday's history from Chrome, Edge, Brave, and Firefox on Windows.
    pub fn get_yesterday_history() -> Vec<HistoryEntry> {
        let mut all_history = Vec::new();

        // 1. Chrome
        if let Some(path) = Self::get_chrome_history_path() {
            let entries = Self::query_chromium_history(&path, "chrome");
            all_history.extend(entries);
        }

        // 2. Edge
        if let Some(path) = Self::get_edge_history_path() {
            let entries = Self::query_chromium_history(&path, "edge");
            all_history.extend(entries);
        }

        // 3. Brave
        if let Some(path) = Self::get_brave_history_path() {
            let entries = Self::query_chromium_history(&path, "brave");
            all_history.extend(entries);
        }

        // 4. Firefox
        if let Some(path) = Self::get_firefox_history_path() {
            let entries = Self::query_firefox_history(&path);
            all_history.extend(entries);
        }

        // Sort by visit_time descending
        all_history.sort_by(|a, b| b.visit_time.cmp(&a.visit_time));
        all_history
    }

    /// Fetches all currently open tabs and titles using UI Automation via PowerShell.
    pub fn get_open_tabs() -> Vec<TabInfo> {
        // Disabled background PowerShell scan to prevent crashes and UI freezing.
        Vec::new()
    }

    /// Closes duplicate tabs (tabs with identical titles) across all open browser windows.
    pub fn close_duplicate_tabs() -> String {
        let ps_cmd = r#"
        Add-Type -AssemblyName UIAutomationClient
        Add-Type -AssemblyName UIAutomationTypes
        Add-Type -AssemblyName System.Windows.Forms
        $roots = [System.Windows.Automation.AutomationElement]::RootElement.FindAll(
            [System.Windows.Automation.TreeScope]::Children,
            [System.Windows.Automation.Condition]::TrueCondition
        )
        $closedCount = 0
        $closedTitles = @()
        foreach ($root in $roots) {
            $procId = $root.Current.ProcessId
            $proc = Get-Process -Id $procId -ErrorAction SilentlyContinue
            if ($proc -ne $null -and ($proc.ProcessName -eq 'chrome' -or $proc.ProcessName -eq 'msedge' -or $proc.ProcessName -eq 'brave' -or $proc.ProcessName -eq 'firefox')) {
                $cond = New-Object System.Windows.Automation.PropertyCondition(
                    [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                    [System.Windows.Automation.ControlType]::TabItem
                )
                $tabs = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
                $seen = @{}
                foreach ($tab in $tabs) {
                    $name = $tab.Current.Name
                    if ($name -ne $null -and $name.Trim() -ne "") {
                        if ($seen.ContainsKey($name)) {
                            try {
                                $selPattern = $tab.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
                                if ($selPattern -ne $null) {
                                    $selPattern.Select()
                                    Start-Sleep -Milliseconds 150
                                    [System.Windows.Forms.SendKeys]::SendWait("^{w}")
                                    $closedCount++
                                    $closedTitles += $name
                                    Start-Sleep -Milliseconds 250
                                }
                            } catch {}
                        } else {
                            $seen[$name] = $true
                        }
                    }
                }
            }
        }
        @{ closedCount = $closedCount; closedTitles = $closedTitles } | ConvertTo-Json -Compress
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_cmd])
            .output();

        let stdout = match output {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
            _ => return "Failed to execute tab cleaner script.".to_string(),
        };

        #[derive(Deserialize)]
        struct CloseResult {
            #[serde(rename = "closedCount")]
            closed_count: i32,
            #[serde(rename = "closedTitles")]
            closed_titles: Vec<String>,
        }

        if let Ok(res) = serde_json::from_str::<CloseResult>(&stdout) {
            if res.closed_count > 0 {
                format!("Successfully closed {} duplicate tab(s): {}", res.closed_count, res.closed_titles.join(", "))
            } else {
                "No duplicate tabs were found.".to_string()
            }
        } else {
            "No duplicate tabs were found or unable to clean tabs.".to_string()
        }
    }

    /// Scrapes active tab URL text or uses screen OCR text to summarize a web page.
    pub fn summarize_current_page(db: &Database, active_url: Option<&str>) -> Result<String, String> {
        let mut page_text = None;

        if let Some(url) = active_url {
            if url.starts_with("http") {
                page_text = Self::fetch_url_text(url);
            }
        }

        let content_to_summarize = match page_text {
            Some(text) if !text.trim().is_empty() => text,
            _ => {
                // Fallback to screen OCR
                let ocr = crate::vision::VisionEngine::capture_screen_text();
                if ocr.trim().is_empty() {
                    return Err("I was unable to retrieve the page content or detect text on the screen to summarize.".to_string());
                }
                ocr
            }
        };

        // Call Groq to summarize
        let api_key = db.get_setting("groq_api_key").ok().flatten();
        let model = db.get_setting("groq_model").ok().flatten().unwrap_or_else(|| "llama-3.1-8b-instant".to_string());

        if api_key.is_none() || api_key.as_ref().map_or(true, |k| k.is_empty()) {
            return Ok(format!(
                "Browser Intelligence: Extracted page content (first 300 characters):\n\n{}\n\n(Configure Groq API key in settings for a natural summary)",
                content_to_summarize.chars().take(300).collect::<String>()
            ));
        }

        let client = GroqClient::new(api_key, &model);
        let prompt = format!(
            "You are ALOK OS, a premium personal assistant. Summarize the following web page content. Keep it structured, highly informative, and concise.\n\nContent:\n{}",
            content_to_summarize
        );

        match client.query(&prompt) {
            Ok(reply) => Ok(reply),
            Err(e) => Err(format!("Failed to query Groq reasoning model: {:?}", e)),
        }
    }

    // ── Helper paths and SQLite queries ──────────────────────────────────────

    fn get_chrome_history_path() -> Option<PathBuf> {
        let local = std::env::var("LOCALAPPDATA").ok()?;
        let path = PathBuf::from(local).join("Google/Chrome/User Data/Default/History");
        if path.exists() { Some(path) } else { None }
    }

    fn get_edge_history_path() -> Option<PathBuf> {
        let local = std::env::var("LOCALAPPDATA").ok()?;
        let path = PathBuf::from(local).join("Microsoft/Edge/User Data/Default/History");
        if path.exists() { Some(path) } else { None }
    }

    fn get_brave_history_path() -> Option<PathBuf> {
        let local = std::env::var("LOCALAPPDATA").ok()?;
        let path = PathBuf::from(local).join("BraveSoftware/Brave-Browser/User Data/Default/History");
        if path.exists() { Some(path) } else { None }
    }

    fn get_firefox_history_path() -> Option<PathBuf> {
        let appdata = std::env::var("APPDATA").ok()?;
        let profiles_dir = PathBuf::from(appdata).join("Mozilla/Firefox/Profiles");
        if profiles_dir.exists() {
            if let Ok(entries) = fs::read_dir(profiles_dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.contains(".default") {
                            let places = entry.path().join("places.sqlite");
                            if places.exists() {
                                return Some(places);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn query_chromium_history(db_path: &Path, browser_name: &str) -> Vec<HistoryEntry> {
        let temp_dir = std::env::temp_dir();
        let temp_db_path = temp_dir.join(format!("{}_History.tmp", browser_name));

        if fs::copy(db_path, &temp_db_path).is_err() {
            return Vec::new();
        }

        let mut entries = Vec::new();
        if let Ok(conn) = Connection::open(&temp_db_path) {
            // Yesterday timestamp limit in Chromium (microseconds since Jan 1, 1601)
            let one_day_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(86400 * 2);
            let limit_secs = one_day_ago.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            let limit_time = (limit_secs + 11644473600) * 1000000;

            let query = "SELECT url, title, datetime(last_visit_time/1000000 - 11644473600, 'unixepoch') FROM urls WHERE last_visit_time > ? ORDER BY last_visit_time DESC LIMIT 100";
            if let Ok(mut stmt) = conn.prepare(query) {
                if let Ok(rows) = stmt.query_map([limit_time], |row| {
                    let url: String = row.get(0)?;
                    let title: String = row.get(1)?;
                    let visit_time: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());

                    Ok(HistoryEntry {
                        browser: browser_name.to_string(),
                        url,
                        title,
                        visit_time,
                    })
                }) {
                    for entry in rows.flatten() {
                        entries.push(entry);
                    }
                }
            }
        }

        let _ = fs::remove_file(temp_db_path);
        entries
    }

    fn query_firefox_history(db_path: &Path) -> Vec<HistoryEntry> {
        let temp_dir = std::env::temp_dir();
        let temp_db_path = temp_dir.join("firefox_places.tmp");

        if fs::copy(db_path, &temp_db_path).is_err() {
            return Vec::new();
        }

        let mut entries = Vec::new();
        if let Ok(conn) = Connection::open(&temp_db_path) {
            // Yesterday timestamp limit (microseconds since Jan 1, 1970)
            let one_day_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(86400 * 2);
            let limit_time = one_day_ago.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_micros() as u64;

            let query = "SELECT h.url, h.title, datetime(v.visit_date/1000000, 'unixepoch') FROM moz_places h JOIN moz_historyvisits v ON h.id = v.place_id WHERE v.visit_date > ? ORDER BY v.visit_date DESC LIMIT 100";
            if let Ok(mut stmt) = conn.prepare(query) {
                if let Ok(rows) = stmt.query_map([limit_time], |row| {
                    let url: String = row.get(0)?;
                    let title: String = row.get(1)?;
                    let visit_time: String = row.get(2).unwrap_or_else(|_| "Unknown".to_string());

                    Ok(HistoryEntry {
                        browser: "firefox".to_string(),
                        url,
                        title,
                        visit_time,
                    })
                }) {
                    for entry in rows.flatten() {
                        entries.push(entry);
                    }
                }
            }
        }

        let _ = fs::remove_file(temp_db_path);
        entries
    }

    fn fetch_url_text(url: &str) -> Option<String> {
        let ps_cmd = format!(
            "$ProgressPreference = 'SilentlyContinue'; \
             try {{ \
                 $html = Invoke-RestMethod -Uri '{}' -UserAgent 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'; \
                 $clean = $html -replace '<[^>]+>', ''; \
                 $clean = $clean -replace '\\s+', ' '; \
                 $clean.Substring(0, [Math]::Min(4000, $clean.Length)) \
             }} catch {{ \
                 write-host '' \
             }}",
            url
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        None
    }
}
