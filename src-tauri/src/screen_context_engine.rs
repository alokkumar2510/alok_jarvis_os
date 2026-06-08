use screenshots::Screen;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use windows::core::Result;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{InMemoryRandomAccessStream, DataWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenContext {
    pub window_title: String,
    pub process_name: String,
    pub browser_url: Option<String>,
    pub selected_text: Option<String>,
    pub ocr_text: Option<String>,
    pub timestamp: String,
}

lazy_static::lazy_static! {
    static ref CONTEXT_CACHE: Arc<Mutex<Option<ScreenContext>>> = Arc::new(Mutex::new(None));
}

pub struct ScreenContextEngine;

impl ScreenContextEngine {
    pub fn new() -> Self {
        Self
    }

    /// Must be called from within Tauri's setup() closure where the Tokio runtime is live.
    pub fn start(&self) {
        println!("ScreenContextEngine: Starting active window change tracker...");
        tauri::async_runtime::spawn(async move {
            let mut last_hwnd = 0isize;
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                unsafe {
                    #[link(name = "user32")]
                    extern "system" {
                        fn GetForegroundWindow() -> isize;
                    }
                    let hwnd = GetForegroundWindow();
                    if hwnd != last_hwnd && hwnd != 0 {
                        last_hwnd = hwnd;
                        let _ = Self::capture_context_internal(hwnd, false);
                    }
                }
            }
        });
    }

    pub fn capture_and_ocr(&self) -> Result<String> {
        // Keeps backwards compatibility with the original signature!
        unsafe {
            #[link(name = "user32")]
            extern "system" {
                fn GetForegroundWindow() -> isize;
            }
            let hwnd = GetForegroundWindow();
            match Self::capture_context_internal(hwnd, true) {
                Ok(ctx) => Ok(ctx.ocr_text.unwrap_or_default()),
                Err(e) => Err(windows::core::Error::new(
                    windows::core::HRESULT(-1),
                    format!("Capture failed: {}", e),
                )),
            }
        }
    }

    pub fn get_current_context() -> Option<ScreenContext> {
        if let Ok(cache) = CONTEXT_CACHE.lock() {
            cache.clone()
        } else {
            None
        }
    }

    /// Triggers screen context capture on wake word activation (with OCR)
    pub fn on_wakeword_invoked() {
        println!("ScreenContextEngine: Wake word invoked. Capturing context...");
        unsafe {
            #[link(name = "user32")]
            extern "system" {
                fn GetForegroundWindow() -> isize;
            }
            let hwnd = GetForegroundWindow();
            let _ = Self::capture_context_internal(hwnd, true);
        }
    }

    /// Triggers screen context capture on user manual request (with OCR)
    pub fn on_user_requested_analysis() {
        println!("ScreenContextEngine: User requested analysis. Capturing context...");
        unsafe {
            #[link(name = "user32")]
            extern "system" {
                fn GetForegroundWindow() -> isize;
            }
            let hwnd = GetForegroundWindow();
            let _ = Self::capture_context_internal(hwnd, true);
        }
    }

    fn capture_context_internal(hwnd: isize, perform_ocr: bool) -> std::result::Result<ScreenContext, Box<dyn std::error::Error + Send + Sync>> {
        unsafe {
            // 1. Get Window Title
            #[link(name = "user32")]
            extern "system" {
                fn GetWindowTextW(hwnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
            }
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            let window_title = if len > 0 {
                String::from_utf16_lossy(&title_buf[..len as usize])
            } else {
                "unknown".to_string()
            };

            // 2. Get Process Name
            let process_name = get_process_name_from_hwnd(hwnd);

            // 3. Extract Browser URL if process is browser
            let browser_url = extract_url_from_title(&window_title, &process_name);

            // 4. Read Selected Text (via Clipboard)
            let selected_text = if perform_ocr {
                get_clipboard_text()
            } else {
                None
            };

            // 5. Perform OCR if requested
            let ocr_text = if perform_ocr {
                run_winrt_ocr_flow().ok()
            } else {
                None
            };

            let ctx = ScreenContext {
                window_title,
                process_name,
                browser_url,
                selected_text,
                ocr_text,
                timestamp: format!("{:?}", Instant::now()),
            };

            // Update cache
            if let Ok(mut cache) = CONTEXT_CACHE.lock() {
                *cache = Some(ctx.clone());
            }

            Ok(ctx)
        }
    }
}

// ── Native OCR Flow ──────────────────────────────────────────────────────

fn run_winrt_ocr_flow() -> Result<String> {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    let screens = Screen::all().map_err(|e| {
        windows::core::Error::new(
            windows::core::HRESULT(-1),
            format!("Failed to list screens: {}", e),
        )
    })?;

    if screens.is_empty() {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
        return Err(windows::core::Error::new(
            windows::core::HRESULT(-1),
            "No screens found",
        ));
    }

    let primary_screen = screens[0];
    let image = primary_screen.capture().map_err(|e| {
        windows::core::Error::new(
            windows::core::HRESULT(-1),
            format!("Failed to capture screen: {}", e),
        )
    })?;

    let mut cursor = Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut cursor, screenshots::image::ImageFormat::Png) {
        unsafe {
            windows::Win32::System::Com::CoUninitialize();
        }
        return Err(windows::core::Error::new(
            windows::core::HRESULT(-1),
            format!("Failed to encode image to PNG: {}", e),
        ));
    }
    let buffer = cursor.into_inner();

    // Perform OCR using Windows WinRT OCR Engine
    let stream = match InMemoryRandomAccessStream::new() {
        Ok(s) => s,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    let writer = match DataWriter::CreateDataWriter(&stream) {
        Ok(w) => w,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    if let Err(e) = writer.WriteBytes(&buffer) {
        unsafe { windows::Win32::System::Com::CoUninitialize(); }
        return Err(e);
    }
    if let Err(e) = writer.StoreAsync().and_then(|a| a.get()) {
        unsafe { windows::Win32::System::Com::CoUninitialize(); }
        return Err(e);
    }
    if let Err(e) = writer.FlushAsync().and_then(|a| a.get()) {
        unsafe { windows::Win32::System::Com::CoUninitialize(); }
        return Err(e);
    }
    if let Err(e) = stream.Seek(0) {
        unsafe { windows::Win32::System::Com::CoUninitialize(); }
        return Err(e);
    }

    let decoder = match BitmapDecoder::CreateAsync(&stream).and_then(|a| a.get()) {
        Ok(d) => d,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    let bitmap = match decoder.GetSoftwareBitmapAsync().and_then(|a| a.get()) {
        Ok(b) => b,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    let engine = match OcrEngine::TryCreateFromUserProfileLanguages() {
        Ok(eng) => eng,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    let result = match engine.RecognizeAsync(&bitmap).and_then(|a| a.get()) {
        Ok(r) => r,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    
    let mut full_text = String::new();
    let lines = match result.Lines() {
        Ok(l) => l,
        Err(e) => {
            unsafe { windows::Win32::System::Com::CoUninitialize(); }
            return Err(e);
        }
    };
    for line in lines {
        if let Ok(text) = line.Text() {
            full_text.push_str(&text.to_string_lossy());
            full_text.push('\n');
        }
    }

    unsafe {
        windows::Win32::System::Com::CoUninitialize();
    }

    Ok(full_text)
}

// ── Helpers ──────────────────────────────────────────────────────────────

#[link(name = "user32")]
extern "system" {
    fn GetWindowThreadProcessId(hwnd: isize, lpdwprocessid: *mut u32) -> u32;
    fn OpenClipboard(hwnd: isize) -> i32;
    fn CloseClipboard() -> i32;
    fn GetClipboardData(format: u32) -> isize;
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dwdesiredaccess: u32, binherithandle: i32, dwprocessid: u32) -> isize;
    fn CloseHandle(hobject: isize) -> i32;
    fn QueryFullProcessImageNameW(hprocess: isize, dwflags: u32, lpexeName: *mut u16, lpdwsize: *mut u32) -> i32;
    fn GlobalLock(hmem: isize) -> *mut u8;
    fn GlobalUnlock(hmem: isize) -> i32;
    fn GlobalSize(hmem: isize) -> usize;
}

unsafe fn get_process_name_from_hwnd(hwnd: isize) -> String {
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == 0 {
        return "unknown.exe".to_string();
    }
    
    let process_handle = OpenProcess(0x1000, 0, pid); // 0x1000 = PROCESS_QUERY_LIMITED_INFORMATION
    if process_handle != 0 {
        let mut size = 260;
        let mut buffer = vec![0u16; 260];
        if QueryFullProcessImageNameW(process_handle, 0, buffer.as_mut_ptr(), &mut size) != 0 {
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            let filename = path.split('\\').last().unwrap_or("unknown.exe").to_string();
            CloseHandle(process_handle);
            return filename;
        }
        CloseHandle(process_handle);
    }
    "unknown.exe".to_string()
}

unsafe fn get_clipboard_text() -> Option<String> {
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

fn extract_url_from_title(title: &str, process_name: &str) -> Option<String> {
    let proc_lower = process_name.to_lowercase();
    if proc_lower.contains("chrome") || proc_lower.contains("edge") || proc_lower.contains("browser") {
        let title_lower = title.to_lowercase();
        if title_lower.contains("youtube") {
            Some("https://youtube.com".to_string())
        } else if title_lower.contains("github") {
            Some("https://github.com".to_string())
        } else if title_lower.contains("google") {
            Some("https://google.com".to_string())
        } else {
            Some("https://localhost".to_string())
        }
    } else {
        None
    }
}
