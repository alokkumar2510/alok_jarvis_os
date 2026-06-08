use std::io::{Result, Error, ErrorKind};
use std::time::Duration;
use std::io::Cursor;
use screenshots::Screen;
use windows::Graphics::Imaging::BitmapDecoder;
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{InMemoryRandomAccessStream, DataWriter};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[link(name = "user32")]
extern "system" {
    pub fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    pub fn SetCursorPos(x: i32, y: i32) -> i32;
    pub fn mouse_event(dwFlags: u32, dx: i32, dy: i32, dwData: u32, dwExtraInfo: usize);
    pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
    pub fn VkKeyScanW(ch: u16) -> i16;
}

const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;

pub async fn move_to(x: i32, y: i32) -> Result<()> {
    unsafe {
        let mut start_pos = POINT::default();
        if GetCursorPos(&mut start_pos) == 0 {
            return Err(Error::new(ErrorKind::Other, "Failed to get current mouse position"));
        }
        
        let dx = (x - start_pos.x) as f64;
        let dy = (y - start_pos.y) as f64;
        let distance = (dx * dx + dy * dy).sqrt();
        
        if distance < 1.0 {
            SetCursorPos(x, y);
            return Ok(());
        }

        let steps = (distance / 12.0).max(8.0).min(45.0) as i32;
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let ease = t * t * (3.0 - 2.0 * t);
            
            let noise_x = (rand_noise(i) - 0.5) * 1.5;
            let noise_y = (rand_noise(i + 17) - 0.5) * 1.5;

            let cur_x = start_pos.x as f64 + dx * ease + noise_x;
            let cur_y = start_pos.y as f64 + dy * ease + noise_y;
            
            SetCursorPos(cur_x as i32, cur_y as i32);
            tokio::time::sleep(Duration::from_millis(8)).await;
        }
        
        SetCursorPos(x, y);
    }
    Ok(())
}

fn rand_noise(seed: i32) -> f64 {
    let x = (seed as f64).sin() * 10000.0;
    x - x.floor()
}

pub async fn click() -> Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        tokio::time::sleep(Duration::from_millis(65)).await;
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
    Ok(())
}

pub async fn right_click() -> Result<()> {
    unsafe {
        mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
        tokio::time::sleep(Duration::from_millis(65)).await;
        mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
    }
    Ok(())
}

pub async fn double_click() -> Result<()> {
    click().await?;
    tokio::time::sleep(Duration::from_millis(160)).await;
    click().await?;
    Ok(())
}

pub async fn type_text(text: &str) -> Result<()> {
    unsafe {
        for ch in text.chars() {
            if ch == '\n' {
                keybd_event(0x0D, 0, 0, 0);
                tokio::time::sleep(Duration::from_millis(15)).await;
                keybd_event(0x0D, 0, 0x0002, 0);
                tokio::time::sleep(Duration::from_millis(25)).await;
                continue;
            }

            let scan = VkKeyScanW(ch as u16);
            if scan == -1 {
                continue;
            }
            let vk = (scan & 0xFF) as u8;
            let shift = ((scan >> 8) & 1) != 0;

            if shift {
                keybd_event(0x10, 0, 0, 0);
                tokio::time::sleep(Duration::from_millis(8)).await;
            }

            keybd_event(vk, 0, 0, 0);
            tokio::time::sleep(Duration::from_millis(15)).await;
            keybd_event(vk, 0, 0x0002, 0);

            if shift {
                tokio::time::sleep(Duration::from_millis(8)).await;
                keybd_event(0x10, 0, 0x0002, 0);
            }

            let delay = 35 + (rand_noise(vk as i32) * 30.0) as u64;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
    }
    Ok(())
}

pub fn find_ui_element(text: &str) -> Result<Option<(i32, i32)>> {
    let screens = Screen::all().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    if screens.is_empty() {
        return Ok(None);
    }
    let primary = screens[0];
    let image = primary.capture().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, screenshots::image::ImageFormat::Png).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let buffer = cursor.into_inner();

    let stream = InMemoryRandomAccessStream::new().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let writer = DataWriter::CreateDataWriter(&stream).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    writer.WriteBytes(&buffer).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    writer.StoreAsync().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.get().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    writer.FlushAsync().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.get().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    stream.Seek(0).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let decoder = BitmapDecoder::CreateAsync(&stream).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.get().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let bitmap = decoder.GetSoftwareBitmapAsync().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.get().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let engine = OcrEngine::TryCreateFromUserProfileLanguages().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let result = engine.RecognizeAsync(&bitmap).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.get().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    let lines = result.Lines().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    let query_lower = text.to_lowercase();

    for line in lines {
        let line_text = line.Text().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.to_string().to_lowercase();
        if line_text.contains(&query_lower) {
            let words = line.Words().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            let mut found = false;

            for word in words {
                let word_text = word.Text().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?.to_string().to_lowercase();
                if query_lower.contains(&word_text) || word_text.contains(&query_lower) {
                    let rect = word.BoundingRect().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                    if rect.X < min_x { min_x = rect.X; }
                    if rect.X + rect.Width > max_x { max_x = rect.X + rect.Width; }
                    if rect.Y < min_y { min_y = rect.Y; }
                    if rect.Y + rect.Height > max_y { max_y = rect.Y + rect.Height; }
                    found = true;
                }
            }

            if found {
                return Ok(Some(((min_x + max_x) as i32 / 2, (min_y + max_y) as i32 / 2)));
            } else {
                let words = line.Words().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                for word in words {
                    let rect = word.BoundingRect().map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
                    if rect.X < min_x { min_x = rect.X; }
                    if rect.X + rect.Width > max_x { max_x = rect.X + rect.Width; }
                    if rect.Y < min_y { min_y = rect.Y; }
                    if rect.Y + rect.Height > max_y { max_y = rect.Y + rect.Height; }
                }
                return Ok(Some(((min_x + max_x) as i32 / 2, (min_y + max_y) as i32 / 2)));
            }
        }
    }

    Ok(None)
}

pub fn manipulate_window(app_name: &str, action: &str) -> Result<()> {
    match action.to_lowercase().as_str() {
        "focus" => crate::desktop::control::focus_app(app_name),
        "minimize" => crate::desktop::control::minimize_app(app_name),
        "maximize" => crate::desktop::control::maximize_app(app_name),
        "close" => crate::desktop::control::close_app(app_name),
        _ => Err(Error::new(ErrorKind::InvalidInput, format!("Unknown window action: {}", action))),
    }
}
