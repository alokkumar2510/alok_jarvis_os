use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<OcrCache>> = Arc::new(Mutex::new(OcrCache::new()));
}

struct OcrCache {
    text: Option<String>,
    timestamp: Option<Instant>,
}

impl OcrCache {
    fn new() -> Self {
        Self { text: None, timestamp: None }
    }
}

pub fn get_cached_ocr() -> Option<String> {
    if let Ok(cache) = CACHE.lock() {
        if let Some(ref text) = cache.text {
            if let Some(time) = cache.timestamp {
                if time.elapsed() < Duration::from_secs(3) {
                    println!("OcrCache: Returning cached OCR text (fresh).");
                    return Some(text.clone());
                }
            }
        }
    }
    None
}

pub fn set_cached_ocr(text: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.text = Some(text.to_string());
        cache.timestamp = Some(Instant::now());
        println!("OcrCache: Updated OCR cache.");
    }
}

pub fn invalidate_cache() {
    if let Ok(mut cache) = CACHE.lock() {
        cache.text = None;
        cache.timestamp = None;
        println!("OcrCache: Invalidated cache.");
    }
}
