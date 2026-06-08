use tauri::{AppHandle, Manager, Window, PhysicalSize, PhysicalPosition};

pub fn position_bottom_right(window: &Window, width: u32, height: u32) -> Result<(), String> {
    if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
        let monitor_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        
        // Convert logical pixels to physical pixels based on monitor scaling factor
        let phys_w = (width as f64 * scale_factor) as u32;
        let phys_h = (height as f64 * scale_factor) as u32;
        
        // Add a 24 logical pixel margin from bottom right corner
        let margin = (24.0 * scale_factor) as u32;
        
        let x = monitor_size.width.saturating_sub(phys_w).saturating_sub(margin);
        // Position at bottom of screen, keeping taskbar safety margin
        let y = monitor_size.height.saturating_sub(phys_h).saturating_sub(margin);
        
        window.set_size(tauri::Size::Physical(PhysicalSize::new(phys_w, phys_h))).map_err(|e| e.to_string())?;
        window.set_position(tauri::Position::Physical(PhysicalPosition::new(x as i32, y as i32))).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_bubble(window: Window) -> Result<(), String> {
    // 120x120 window fits the 80x80 bubble + glow effects
    position_bottom_right(&window, 120, 120)?;
    window.set_ignore_cursor_events(false).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_card(window: Window) -> Result<(), String> {
    // 520x270 fits the 400x150 card + 80x80 bubble + gap & glows
    position_bottom_right(&window, 520, 270)?;
    window.set_ignore_cursor_events(false).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn show_toast_window(window: Window) -> Result<(), String> {
    // 380x450 fits the 330px wide toast + 80x80 bubble + glows
    position_bottom_right(&window, 380, 450)?;
    window.set_ignore_cursor_events(false).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn enable_click_through(window: Window) -> Result<(), String> {
    window.set_ignore_cursor_events(true).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn disable_click_through(window: Window) -> Result<(), String> {
    window.set_ignore_cursor_events(false).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_overlay(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_window_label(window: Window) -> String {
    window.label().to_string()
}

#[tauri::command]
pub fn close_window(window: Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_command_center(app: AppHandle) -> Result<(), String> {
    eprintln!("RUST: show_command_center called!");
    if let Some(cc_window) = app.get_webview_window("command_center") {
        eprintln!("RUST: Found existing command_center window!");
        let _ = cc_window.show();
        let _ = cc_window.set_focus();
    } else {
        eprintln!("RUST: Creating new command_center window with WebviewUrl::App('command_center.html')");
        // Pass ?window=command_center in the URL so JS can detect identity
        // immediately on load without waiting for async IPC
        let cc_window = tauri::WebviewWindowBuilder::new(
            &app,
            "command_center",
            tauri::WebviewUrl::App("command_center.html".into())
        )
        .title("ALOK Command Center")
        .inner_size(1000.0, 700.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .decorations(true)
        .transparent(false)
        .always_on_top(false)
        .center()
        .build()
        .map_err(|e| {
            eprintln!("RUST: Error building command_center window: {:?}", e);
            e.to_string()
        })?;

        if let Ok(url) = cc_window.url() {
            eprintln!("RUST: Command Center Window URL: {:?}", url);
        } else {
            eprintln!("RUST: Failed to get Command Center Window URL");
        }

        #[cfg(debug_assertions)]
        {
            eprintln!("RUST: Opening devtools for command_center window");
            cc_window.open_devtools();
        }
    }
    Ok(())
}
