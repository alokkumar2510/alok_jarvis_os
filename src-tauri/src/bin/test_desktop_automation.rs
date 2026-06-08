use std::time::Duration;
use alok_jarvis_os_lib::desktop::automation_runtime::{move_to, click, double_click, right_click, type_text, find_ui_element};

#[tokio::main]
async fn main() {
    println!("=== Testing Desktop Automation Subsystem ===");

    // 1. Mouse movement test (smooth cubic eased coordinates step)
    println!("Moving cursor to (500, 500)...");
    match move_to(500, 500).await {
        Ok(_) => println!("Cursor moved successfully."),
        Err(e) => eprintln!("Failed to move cursor: {}", e),
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 2. Click test
    println!("Performing left click...");
    match click().await {
        Ok(_) => println!("Left click completed."),
        Err(e) => eprintln!("Failed left click: {}", e),
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 3. Double click test
    println!("Performing double click...");
    match double_click().await {
        Ok(_) => println!("Double click completed."),
        Err(e) => eprintln!("Failed double click: {}", e),
    }

    tokio::time::sleep(Duration::from_millis(500)).await;

    // 4. Right click test
    println!("Performing right click...");
    match right_click().await {
        Ok(_) => println!("Right click completed."),
        Err(e) => eprintln!("Failed right click: {}", e),
    }

    // 5. Text typing test
    println!("Typing text test (simulating keyboard)...");
    match type_text("Hello World!").await {
        Ok(_) => println!("Text typing finished."),
        Err(e) => eprintln!("Failed typing: {}", e),
    }

    // 6. UI Element search test
    println!("Searching for a visible UI element containing 'health'...");
    match find_ui_element("health") {
        Ok(Some((x, y))) => println!("Found element at coordinates: ({}, {})", x, y),
        Ok(None) => println!("Element not found on screen."),
        Err(e) => eprintln!("OCR screen capture failed: {}", e),
    }

    println!("Desktop Automation tests completed.");
}
