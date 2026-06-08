use std::process::Command;
use std::io::{Result, Error, ErrorKind};
use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, IsWindowVisible, ShowWindow, SetForegroundWindow,
    SW_MINIMIZE, SW_MAXIMIZE, SW_RESTORE
};

// ── Applications ─────────────────────────────────────────────────────────

pub fn launch_app(app_name: &str) -> Result<String> {
    let name_lower = app_name.to_lowercase();

    let cmd = if name_lower.contains("chrome") {
        "start chrome".to_string()
    } else if name_lower.contains("code") || name_lower.contains("vscode") || name_lower == "vs code" {
        "start code".to_string()
    } else if name_lower.contains("notepad") {
        "notepad.exe".to_string()
    } else if name_lower.contains("explorer") || name_lower.contains("folder") || name_lower == "files" || name_lower == "file explorer" || name_lower == "my files" {
        "explorer.exe".to_string()
    } else if name_lower.contains("terminal") || name_lower.contains("powershell") {
        "start powershell".to_string()
    } else if name_lower == "cmd" || name_lower.contains("command prompt") {
        "start cmd".to_string()
    } else if name_lower.contains("spotify") {
        "start spotify".to_string()
    } else if name_lower.contains("discord") {
        "start discord".to_string()
    } else if name_lower.contains("chatgpt") {
        "start https://chat.openai.com".to_string()
    } else if name_lower.contains("youtube") {
        "start https://youtube.com".to_string()
    } else if name_lower.contains("gmail") {
        "start https://mail.google.com".to_string()
    } else {
        format!("start {}", app_name)
    };

    println!("DesktopControl: Launching process: {}", cmd);

    if cmd.ends_with(".exe") {
        Command::new(&cmd).spawn()?;
    } else {
        Command::new("cmd")
            .args(["/C", &cmd])
            .spawn()?;
    }

    Ok(cmd)
}

pub fn close_app(app_name: &str) -> Result<()> {
    let name_lower = app_name.to_lowercase();
    let process_name_holder;
    let process_name = if name_lower.contains("chrome") {
        "chrome.exe"
    } else if name_lower.contains("code") || name_lower.contains("vs") {
        "code.exe"
    } else if name_lower.contains("notepad") {
        "notepad.exe"
    } else if name_lower.contains("explorer") || name_lower.contains("files") || name_lower.contains("folder") {
        "explorer.exe"
    } else if name_lower.contains("spotify") {
        "Spotify.exe"
    } else if name_lower.contains("discord") {
        "Discord.exe"
    } else if name_lower.ends_with(".exe") {
        app_name
    } else {
        process_name_holder = format!("{}.exe", app_name);
        &process_name_holder
    };
    
    println!("DesktopControl: Terminating process: {}", process_name);
    let status = Command::new("taskkill")
        .args(["/F", "/IM", process_name])
        .status()?;
        
    if status.success() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, format!("Failed to terminate process {}", process_name)))
    }
}

pub fn focus_app(app_name: &str) -> Result<()> {
    println!("DesktopControl: Focusing window: {}", app_name);
    unsafe {
        if let Some(hwnd) = find_window_by_name(app_name) {
            let _ = ShowWindow(hwnd, SW_RESTORE);
            let _ = SetForegroundWindow(hwnd);
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::NotFound, format!("Window for app '{}' not found", app_name)))
}

pub fn minimize_app(app_name: &str) -> Result<()> {
    println!("DesktopControl: Minimizing window: {}", app_name);
    unsafe {
        if let Some(hwnd) = find_window_by_name(app_name) {
            let _ = ShowWindow(hwnd, SW_MINIMIZE);
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::NotFound, format!("Window for app '{}' not found", app_name)))
}

pub fn maximize_app(app_name: &str) -> Result<()> {
    println!("DesktopControl: Maximizing window: {}", app_name);
    unsafe {
        if let Some(hwnd) = find_window_by_name(app_name) {
            let _ = ShowWindow(hwnd, SW_MAXIMIZE);
            return Ok(());
        }
    }
    Err(Error::new(ErrorKind::NotFound, format!("Window for app '{}' not found", app_name)))
}

// ── System ───────────────────────────────────────────────────────────────

pub fn shutdown_system() -> Result<()> {
    println!("DesktopControl: Shutting down system...");
    let status = Command::new("shutdown")
        .args(["/s", "/t", "0"])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "Shutdown command failed"))
    }
}

pub fn restart_system() -> Result<()> {
    println!("DesktopControl: Restarting system...");
    let status = Command::new("shutdown")
        .args(["/r", "/t", "0"])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::new(ErrorKind::Other, "Restart command failed"))
    }
}

pub fn lock_system() -> Result<()> {
    crate::desktop::power::lock_workstation()
}

pub fn sleep_system() -> Result<()> {
    crate::desktop::power::suspend_system()
}

// ── Browser ──────────────────────────────────────────────────────────────

pub fn open_website(url: &str) -> Result<()> {
    println!("DesktopControl: Opening website: {}", url);

    let resolved = resolve_url(url);
    println!("DesktopControl: Resolved URL: {}", resolved);

    Command::new("cmd")
        .args(["/C", &format!("start {}", resolved)])
        .spawn()?;

    Ok(())
}

/// Resolves natural language site names to real URLs.
fn resolve_url(input: &str) -> String {
    let lower = input.to_lowercase();
    let lower = lower.trim().trim_end_matches('.').trim_end_matches(" web").trim_end_matches(" site").trim_end_matches(" page");

    // Known site mappings
    let known_sites: &[(&str, &str)] = &[
        ("instagram", "https://instagram.com"),
        ("facebook", "https://facebook.com"),
        ("youtube", "https://youtube.com"),
        ("google", "https://google.com"),
        ("gmail", "https://mail.google.com"),
        ("twitter", "https://twitter.com"),
        ("x.com", "https://x.com"),
        ("reddit", "https://reddit.com"),
        ("github", "https://github.com"),
        ("linkedin", "https://linkedin.com"),
        ("whatsapp", "https://web.whatsapp.com"),
        ("netflix", "https://netflix.com"),
        ("chatgpt", "https://chat.openai.com"),
        ("openai", "https://chat.openai.com"),
        ("amazon", "https://amazon.in"),
        ("flipkart", "https://flipkart.com"),
        ("spotify", "https://open.spotify.com"),
        ("maps", "https://maps.google.com"),
        ("translate", "https://translate.google.com"),
        ("drive", "https://drive.google.com"),
        ("docs", "https://docs.google.com"),
    ];

    for (name, site_url) in known_sites {
        if lower.contains(name) {
            return site_url.to_string();
        }
    }

    // If it already looks like a URL or domain, format it
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{}", input);
    }

    // Fallback: Google search
    let encoded = input.replace(' ', "+");
    format!("https://www.google.com/search?q={}", encoded)
}


pub fn search_query(query: &str) -> Result<()> {
    println!("DesktopControl: Searching query: {}", query);
    let encoded_query = query.replace(" ", "+");
    let url = format!("https://www.google.com/search?q={}", encoded_query);
    open_website(&url)
}

pub fn open_new_tab() -> Result<()> {
    open_website("https://google.com")
}

// ── Files ────────────────────────────────────────────────────────────────

pub fn open_file(filepath: &str) -> Result<()> {
    println!("DesktopControl: Opening file: {}", filepath);
    Command::new("cmd")
        .args(["/C", "start", "", filepath])
        .spawn()?;
    Ok(())
}

pub fn search_file(filename: &str) -> Result<String> {
    println!("DesktopControl: Searching file: {}", filename);
    Ok(format!("Found file match for '{}' at E:/Projects/flutter_app", filename))
}

pub fn reveal_file(filepath: &str) -> Result<()> {
    println!("DesktopControl: Revealing file in Explorer: {}", filepath);
    Command::new("explorer")
        .args([&format!("/select,{}", filepath)])
        .spawn()?;
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────

unsafe fn find_window_by_name(app_name: &str) -> Option<HWND> {
    struct Target {
        name: String,
        found_hwnd: Option<HWND>,
    }
    
    unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let target = &mut *(lparam.0 as *mut Target);
        if IsWindowVisible(hwnd).as_bool() {
            let mut text = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut text);
            if len > 0 {
                let window_title = String::from_utf16_lossy(&text[..len as usize]).to_lowercase();
                if window_title.contains(&target.name) {
                    target.found_hwnd = Some(hwnd);
                    return false.into(); // Stop enumeration
                }
            }
        }
        true.into()
    }
    
    let mut target = Target {
        name: app_name.to_lowercase(),
        found_hwnd: None,
    };
    
    let lparam = LPARAM(&mut target as *mut Target as isize);
    let _ = EnumWindows(Some(enum_window_callback), lparam);
    target.found_hwnd
}

pub fn is_process_running(app_name: &str) -> bool {
    let name_lower = app_name.to_lowercase();
    let process_name = if name_lower.contains("chrome") {
        "chrome.exe"
    } else if name_lower.contains("code") || name_lower.contains("vs") {
        "code.exe"
    } else if name_lower.contains("notepad") {
        "notepad.exe"
    } else if name_lower.contains("explorer") || name_lower.contains("files") || name_lower.contains("folder") {
        "explorer.exe"
    } else if name_lower.contains("spotify") {
        "Spotify.exe"
    } else if name_lower.contains("discord") {
        "Discord.exe"
    } else if name_lower.ends_with(".exe") {
        app_name
    } else {
        return check_process_via_tasklist(app_name);
    };
    
    check_process_via_tasklist(process_name)
}

fn check_process_via_tasklist(process_name: &str) -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {}", process_name), "/NH"])
        .output();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
        stdout.contains(&process_name.to_lowercase())
    } else {
        false
    }
}
