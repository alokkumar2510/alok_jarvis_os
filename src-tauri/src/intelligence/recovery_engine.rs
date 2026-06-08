use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

pub struct RecoveryEngine;

impl RecoveryEngine {
    /// Attempts to find an alternative file path if the requested file is missing.
    /// Scans standard folder directories (Desktop, Documents, Downloads) for matches.
    pub fn recover_file_not_found(missing_path: &str) -> Option<PathBuf> {
        println!("RecoveryEngine: Handling file not found for '{}'", missing_path);
        
        let path = Path::new(missing_path);
        let file_name = path.file_name()?.to_str()?.to_lowercase();
        
        let home_dir = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| "C:\\Users\\alokk".to_string());
            
        let folders_to_search = vec![
            Path::new(&home_dir).join("Downloads"),
            Path::new(&home_dir).join("Documents"),
            Path::new(&home_dir).join("Desktop"),
        ];

        for folder in folders_to_search {
            if let Ok(entries) = fs::read_dir(folder) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Some(name) = entry_path.file_name().and_then(|s| s.to_str()) {
                            if name.to_lowercase().contains(&file_name) || file_name.contains(&name.to_lowercase()) {
                                println!("RecoveryEngine: Found alternative match: {:?}", entry_path);
                                return Some(entry_path);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Recovers a hanging browser by terminating its processes and starting it fresh.
    pub fn recover_browser_hang(browser_name: &str) -> Result<String, String> {
        println!("RecoveryEngine: Attempting to recover hanging browser '{}'", browser_name);
        
        let proc_name = if browser_name.to_lowercase().contains("chrome") {
            "chrome.exe"
        } else if browser_name.to_lowercase().contains("edge") {
            "msedge.exe"
        } else if browser_name.to_lowercase().contains("firefox") {
            "firefox.exe"
        } else {
            "chrome.exe"
        };

        // Terminate hanging instances
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", proc_name])
            .status();

        // Spawn fresh instance
        let launch_cmd = format!("start {}", browser_name);
        let status = Command::new("cmd")
            .args(["/C", &launch_cmd])
            .status()
            .map_err(|e| format!("Failed to spawn browser: {}", e))?;

        if status.success() {
            Ok(format!("Successfully terminated hanging {} and restarted a clean browser instance.", browser_name))
        } else {
            Err(format!("Terminated browser process but failed to launch a new instance."))
        }
    }

    /// Automatically retries a failed file download using powershell Invoke-WebRequest.
    pub fn recover_download_failure(url: &str, target_path: &str) -> Result<String, String> {
        println!("RecoveryEngine: Retrying failed download of '{}' to '{}'", url, target_path);
        
        // Construct Powershell download script
        let ps_script = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             Invoke-WebRequest -Uri '{}' -OutFile '{}' -TimeoutSec 30",
            url, target_path
        );

        let mut attempts = 0;
        let max_retries = 3;

        while attempts < max_retries {
            attempts += 1;
            println!("RecoveryEngine: Download retry attempt {} of {}...", attempts, max_retries);
            
            let status = Command::new("powershell")
                .args(["-NoProfile", "-Command", &ps_script])
                .status();

            match status {
                Ok(s) if s.success() => {
                    if Path::new(target_path).exists() {
                        return Ok(format!("Successfully downloaded file after {} retry attempts.", attempts));
                    }
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }

        Err(format!("Download failed after {} retry attempts.", max_retries))
    }
}
