use std::fs;
use std::path::Path;

/// Reads directory structure recursively up to a depth of 3
pub fn read_structure(root_path: &str) -> Vec<String> {
    let mut results = Vec::new();
    let root = Path::new(root_path);
    if !root.exists() {
        return vec![format!("Path '{}' does not exist.", root_path)];
    }

    let mut dirs_to_visit = vec![(root.to_path_buf(), 0)];
    while let Some((dir, depth)) = dirs_to_visit.pop() {
        if depth > 3 {
            continue;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap_or(&path);

                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name == "target" || name == "node_modules" || name == ".git" || name == ".gradle" || name == "build" {
                    continue;
                }

                if path.is_dir() {
                    results.push(format!("{}[D] {}", "  ".repeat(depth), relative.display()));
                    dirs_to_visit.push((path, depth + 1));
                } else {
                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    results.push(format!("{}[F] {} ({} bytes)", "  ".repeat(depth), relative.display(), size));
                }
            }
        }
    }

    results
}

/// Greps source files in the project for occurrences of the search term
pub fn search_codebase(root_path: &str, query: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let root = Path::new(root_path);
    if !root.exists() {
        return vec![];
    }

    let mut dirs_to_visit = vec![root.to_path_buf()];
    while let Some(dir) = dirs_to_visit.pop() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name == "target" || name == "node_modules" || name == ".git" || name == ".gradle" || name == "build" {
                    continue;
                }

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else {
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                    if ["rs", "dart", "js", "ts", "py", "json", "toml", "yaml", "xml", "html", "css"].contains(&ext) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if content.to_lowercase().contains(&query.to_lowercase()) {
                                for (i, line) in content.lines().enumerate() {
                                    if line.to_lowercase().contains(&query.to_lowercase()) {
                                        let relative = path.strip_prefix(root).unwrap_or(&path);
                                        matches.push(format!("{}:{}: {}", relative.display(), i + 1, line.trim()));
                                        if matches.len() >= 50 {
                                            return matches;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    matches
}

/// Lists all files in the project recursively, ignoring build/dependencies folders
pub fn list_project_files(root_path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut results = Vec::new();
    let root = Path::new(root_path);
    if !root.exists() {
        return Ok(vec![]);
    }

    let mut dirs_to_visit = vec![root.to_path_buf()];
    while let Some(dir) = dirs_to_visit.pop() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if name == "target" || name == "node_modules" || name == ".git" || name == ".gradle" || name == "build" {
                    continue;
                }

                if path.is_dir() {
                    dirs_to_visit.push(path);
                } else {
                    if let Some(p_str) = path.to_str() {
                        results.push(p_str.to_string());
                    }
                }
            }
        }
    }

    Ok(results)
}
