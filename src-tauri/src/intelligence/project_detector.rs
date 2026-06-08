use std::path::{Path, PathBuf};

/// Climbs the directory tree from the active file or folder to find project markers
/// and determines the project name and type.
pub fn detect_project(active_file: Option<&str>, active_folder: Option<&str>) -> Option<(String, String)> {
    let folder_path = if let Some(folder) = active_folder {
        if folder.trim().is_empty() { None } else { Some(PathBuf::from(folder)) }
    } else if let Some(file) = active_file {
        if file.trim().is_empty() { None } else { Path::new(file).parent().map(|p| p.to_path_buf()) }
    } else {
        None
    };

    let folder = folder_path?;
    let mut current = folder.as_path();

    loop {
        if current.join("Cargo.toml").exists() {
            let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("Rust Project");
            return Some((name.to_string(), "rust".to_string()));
        }
        if current.join("pubspec.yaml").exists() {
            let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("Flutter Project");
            return Some((name.to_string(), "flutter".to_string()));
        }
        if current.join("package.json").exists() {
            let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("JS/TS Project");
            return Some((name.to_string(), "javascript".to_string()));
        }
        if current.join("requirements.txt").exists() || current.join(".venv").exists() {
            let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("Python Project");
            return Some((name.to_string(), "python".to_string()));
        }
        
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Fallback detection using active file extension
    if let Some(file) = active_file {
        let path = Path::new(file);
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => return Some(("Rust Project".to_string(), "rust".to_string())),
                "dart" => return Some(("Flutter Project".to_string(), "flutter".to_string())),
                "js" | "ts" | "jsx" | "tsx" => return Some(("JS/TS Project".to_string(), "javascript".to_string())),
                "py" => return Some(("Python Project".to_string(), "python".to_string())),
                _ => {}
            }
        }
    }

    None
}
