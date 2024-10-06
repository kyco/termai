use std::fs;
use std::path::Path;
use crate::path::model::Files;

pub fn extract_content(path_str: &Option<String>, exclude: &[String]) -> Option<Vec<Files>> {
    let mut files = vec![];

    let path = match path_str {
        Some(p) => Path::new(p),
        None => return None,
    };

    if path.exists() {
        collect_files(path, &mut files, exclude);
        Some(files)
    } else {
        println!("{} does not exist.", path_str.as_deref().unwrap_or("."));
        None
    }
}

fn collect_files(path: &Path, files: &mut Vec<Files>, exclude: &[String]) {
    if path.is_dir() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || exclude.contains(&name.to_string()) {
                return;
            }
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                collect_files(&entry.path(), files, exclude);
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