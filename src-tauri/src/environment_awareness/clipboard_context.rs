use serde::{Serialize, Deserialize};
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardInfo {
    pub clipboard_text: Option<String>,
    pub selected_text: Option<String>,
}

pub fn capture() -> ClipboardInfo {
    let clipboard_text = get_clipboard_text_native();
    
    // Attempt to copy selected text via sending Ctrl+C
    let selected_text = capture_selected_text(&clipboard_text);

    ClipboardInfo {
        clipboard_text,
        selected_text,
    }
}

pub fn get_clipboard_text_native() -> Option<String> {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn OpenClipboard(hwnd: isize) -> i32;
            fn CloseClipboard() -> i32;
            fn GetClipboardData(format: u32) -> isize;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalLock(hmem: isize) -> *mut u8;
            fn GlobalUnlock(hmem: isize) -> i32;
            fn GlobalSize(hmem: isize) -> usize;
        }

        if OpenClipboard(0) != 0 {
            let hmem = GetClipboardData(13); // CF_UNICODETEXT = 13
            if hmem != 0 {
                let ptr = GlobalLock(hmem);
                if !ptr.is_null() {
                    let size = GlobalSize(hmem);
                    let slice = std::slice::from_raw_parts(ptr as *const u16, size / 2);
                    let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    let text = String::from_utf16_lossy(&slice[..len]);
                    GlobalUnlock(hmem);
                    CloseClipboard();
                    return Some(text);
                }
            }
            CloseClipboard();
        }
        None
    }
}

pub fn set_clipboard_text_native(text: &str) {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn OpenClipboard(hwnd: isize) -> i32;
            fn CloseClipboard() -> i32;
            fn EmptyClipboard() -> i32;
            fn SetClipboardData(format: u32, hmem: isize) -> isize;
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalAlloc(flags: u32, size: usize) -> isize;
            fn GlobalLock(hmem: isize) -> *mut u8;
            fn GlobalUnlock(hmem: isize) -> i32;
        }

        if OpenClipboard(0) != 0 {
            EmptyClipboard();
            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = utf16.len() * 2;
            let hmem = GlobalAlloc(0x0002, size); // GMEM_MOVEABLE = 0x0002
            if hmem != 0 {
                let ptr = GlobalLock(hmem) as *mut u16;
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                    GlobalUnlock(hmem);
                    SetClipboardData(13, hmem); // CF_UNICODETEXT = 13
                }
            }
            CloseClipboard();
        }
    }
}

fn capture_selected_text(original_clipboard: &Option<String>) -> Option<String> {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
        }
        // Press Ctrl (VK_CONTROL = 0x11)
        keybd_event(0x11, 0, 0, 0);
        // Press C (0x43)
        keybd_event(0x43, 0, 0, 0);
        thread::sleep(Duration::from_millis(10));
        // Release C (KEYEVENTF_KEYUP = 2)
        keybd_event(0x43, 0, 2, 0);
        // Release Ctrl
        keybd_event(0x11, 0, 2, 0);
    }

    // Sleep briefly to let the copy operation complete
    thread::sleep(Duration::from_millis(50));

    let new_clip = get_clipboard_text_native();
    
    // Restore original clipboard
    if let Some(ref orig) = original_clipboard {
        set_clipboard_text_native(orig);
    } else {
        unsafe {
            #[link(name = "user32")]
            extern "system" {
                fn OpenClipboard(hwnd: isize) -> i32;
                fn CloseClipboard() -> i32;
                fn EmptyClipboard() -> i32;
            }
            if OpenClipboard(0) != 0 {
                EmptyClipboard();
                CloseClipboard();
            }
        }
    }

    // If clipboard content changed and is not empty, that is our selected text!
    if new_clip != *original_clipboard {
        new_clip
    } else {
        None
    }
}
