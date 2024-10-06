use std::fs;
use std::path::Path;
use crate::path::model::Files;

pub fn extract_content(directory: &Option<String>, exclude: &[String]) -> Option<Vec<Files>> {
    let mut files = Vec::new();

    let path = match directory {
        Some(dir) => Path::new(dir),
        None => return None,
    };

    if path.exists() && path.is_dir() {
        collect_files(path, &mut files, exclude);
        Some(files)
    } else {
        println!("{} is not a valid directory.",
                 directory.as_deref().unwrap_or("."));
        None
    }
}

fn collect_files(path: &Path, files: &mut Vec<Files>, exclude: &[String]) {
    if path.is_dir() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || exclude.contains(&name.to_lowercase()) {
                return;
            }
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                collect_files(&entry_path, files, exclude);
            }
        }
    } else if path.is_file() {
        if let Some(path_str) = path.to_str() {
            if let Ok(content) = fs::read_to_string(path) {
                files.push(Files {
                    path: path_str.to_string(),
                    content,
                });
            }
        }
    }
}