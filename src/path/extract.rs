use crate::path::model::Files;
use std::fs;
use std::path::Path;

pub fn extract_content(dir: &Option<String>, dirs: &[String], exclude: &[String]) -> Option<Vec<Files>> {
    let mut all_dirs = Vec::new();
    if let Some(dir) = dir {
        all_dirs.push(dir.clone());
    }
    all_dirs.extend(dirs.iter().cloned());

    if all_dirs.is_empty() {
        return None;
    }

    let mut files = vec![];
    let mut any_valid_path = false;

    for path_str in all_dirs {
        let path = Path::new(&path_str);
        if path.exists() {
            any_valid_path = true;
            collect_files(path, &mut files, exclude);
        } else {
            println!("{} does not exist.", path_str);
        }
    }

    if any_valid_path {
        Some(files)
    } else {
        None
    }
}

fn collect_files(path: &Path, files: &mut Vec<Files>, exclude: &[String]) {
    let path_str = match path.to_str() {
        Some(s) => remove_dot_slash(s),
        None => return,
    };

    if must_exclude(exclude, path_str) {
        return;
    }

    if path.is_dir() {
        if !path_str.starts_with("./") && path_str.starts_with('.') && path_str != "." {
            return;
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                collect_files(&entry.path(), files, exclude);
            }
        }
    } else if path.is_file() {
        if must_exclude(exclude, path_str) {
            return;
        }
        if let Ok(content) = fs::read_to_string(path) {
            files.push(Files {
                path: path_str.to_string(),
                content,
            });
        }
    }
}

fn must_exclude(exclude: &[String], path: &str) -> bool {
    if exclude.contains(&path.to_string()) {
        return true;
    }
    false
}

fn remove_dot_slash(path: &str) -> &str {
    path.strip_prefix("./").unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // This helper function remains unchanged                                                                                                                                                                      
    fn files_to_map(files: &[Files]) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        for file in files {
            if let Some(fname) = Path::new(&file.path).file_name().and_then(|s| s.to_str()) {
                map.insert(fname.to_string(), file.content.clone());
            }
        }
        map
    }

    // Test 1: When the input Option is None, extract_content returns None.                                                                                                                                        
    #[test]
    fn test_extract_content_returns_none_for_none_path() {
        // Arrange                                                                                                                                                                                                 
        let input: Option<String> = None;
        let exclude: [String; 0] = [];

        // Act - add empty dirs array as second parameter                                                                                                                                                          
        let result = extract_content(&input, &[], &exclude);

        // Assert                                                                                                                                                                                                  
        assert!(result.is_none(), "Expected None when no path is provided.");
    }

    // Test 2: When the provided path does not exist, extract_content returns None.                                                                                                                                
    #[test]
    fn test_extract_content_returns_none_for_nonexistent_path() {
        // Arrange                                                                                                                                                                                                 
        let nonexistent = Some("nonexistent_directory_or_file".to_owned());
        let exclude: [String; 0] = [];

        // Act - add empty dirs array                                                                                                                                                                              
        let result = extract_content(&nonexistent, &[], &exclude);

        // Assert                                                                                                                                                                                                  
        assert!(result.is_none(), "Expected None for a nonexistent path.");
    }

    // Test 3: Extract files from an existing directory (with a nested subdirectory).                                                                                                                              
    #[test]
    fn test_extract_content_with_directory_includes_all_files() {
        // Arrange setup unchanged                                                                                                                                                                                 
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Create file1.txt at root.                                                                                                                                                                               
        let file1_path = temp_path.join("file1.txt");
        {
            let mut file1 = File::create(&file1_path).expect("Failed to create file1.txt");
            write!(file1, "Hello file1!").expect("Failed to write to file1.txt");
        }

        // Create a subdirectory "subdir" and file2.txt inside it.                                                                                                                                                 
        let subdir_path = temp_path.join("subdir");
        fs::create_dir(&subdir_path).expect("Failed to create subdir");
        let file2_path = subdir_path.join("file2.txt");
        {
            let mut file2 = File::create(&file2_path).expect("Failed to create file2.txt");
            write!(file2, "Hello file2!").expect("Failed to write to file2.txt");
        }

        // Act - add empty dirs array                                                                                                                                                                              
        let result = extract_content(&Some(temp_path.to_str().unwrap().to_owned()), &[], &[]);
        let files = result.expect("Expected Some(files) for valid directory");

        // Assert unchanged                                                                                                                                                                                        
        let files_map = files_to_map(&files);
        assert_eq!(
            files_map.len(),
            2,
            "Expected 2 files, found {}",
            files_map.len()
        );
        assert_eq!(files_map.get("file1.txt").unwrap(), "Hello file1!");
        assert_eq!(files_map.get("file2.txt").unwrap(), "Hello file2!");
    }

    // Fix remaining tests similarly by adding the empty dirs array                                                                                                                                                
    #[test]
    fn test_extract_content_with_directory_excludes_specified_file() {
        // Arrange unchanged                                                                                                                                                                                       
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();

        // Create include.txt.                                                                                                                                                                                     
        let include_path = temp_path.join("include.txt");
        {
            let mut include_file =
                File::create(&include_path).expect("Failed to create include.txt");
            write!(include_file, "Keep me").expect("Failed to write to include.txt");
        }

        // Create exclude.txt.                                                                                                                                                                                     
        let exclude_path_buf: PathBuf = temp_path.join("exclude.txt");
        {
            let mut exclude_file =
                File::create(&exclude_path_buf).expect("Failed to create exclude.txt");
            write!(exclude_file, "Remove me").expect("Failed to write to exclude.txt");
        }
        // Convert exclude file path to string (as used in extract_content).                                                                                                                                       
        let exclude_path = exclude_path_buf
            .to_str()
            .expect("Failed to convert exclude path to string")
            .to_owned();
        let excludes = vec![exclude_path];

        // Act - add empty dirs array                                                                                                                                                                              
        let result = extract_content(&Some(temp_path.to_str().unwrap().to_owned()), &[], &excludes);
        let files = result.expect("Expected Some(files) for valid directory");

        // Assert unchanged                                                                                                                                                                                        
        let files_map = files_to_map(&files);
        assert_eq!(
            files_map.len(),
            1,
            "Expected only 1 file after exclusion, found {}",
            files_map.len()
        );
        assert!(
            files_map.contains_key("include.txt"),
            "Expected include.txt to be present."
        );
        assert!(
            !files_map.contains_key("exclude.txt"),
            "Expected exclude.txt to be excluded."
        );
    }

    #[test]
    fn test_extract_content_with_file_input() {
        // Arrange unchanged                                                                                                                                                                                       
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("single_file.txt");
        {
            let mut file = File::create(&file_path).expect("Failed to create single_file.txt");
            write!(file, "File content").expect("Failed to write to single_file.txt");
        }

        // Act - add empty dirs array                                                                                                                                                                              
        let result = extract_content(&Some(file_path.to_str().unwrap().to_owned()), &[], &[]);
        let files = result.expect("Expected Some(files) for a valid file");

        // Assert unchanged                                                                                                                                                                                        
        assert_eq!(
            files.len(),
            1,
            "Expected exactly 1 file to be returned for file input"
        );
        let file_entry = &files[0];
        assert!(
            file_entry.path.ends_with("single_file.txt"),
            "Expected file path to end with 'single_file.txt'"
        );
        assert_eq!(file_entry.content, "File content");
    }

    // Fix the last two tests similarly                                                                                                                                                                            
    #[test]
    fn test_extract_content_with_relative_path_removes_dot_slash() {
        let original_dir = env::current_dir().expect("Failed to get current directory");
        
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = File::create(&file_path).expect("Failed to create test.txt");
            write!(file, "Relative file").expect("Failed to write to test.txt");
        }
        
        // Change to temp directory, run test, restore immediately
        env::set_current_dir(temp_dir.path()).expect("Failed to change current directory");
        let result = extract_content(&Some("./".to_owned()), &[], &[]);
        
        // Restore directory before temp_dir is dropped
        if original_dir.exists() {
            env::set_current_dir(&original_dir).ok();
        } else {
            // Fallback to a safe directory if original doesn't exist
            env::set_current_dir("/tmp").ok();
        }
        
        let files = result.expect("Expected Some(files) when using a relative path");

        // Assert unchanged                                                                                                                                                                                        
        let files_map = files_to_map(&files);
        assert!(
            files_map.contains_key("test.txt"),
            "Expected 'test.txt' to be present in the extracted files."
        );
    }

    #[test]
    fn test_extract_content_excludes_hidden_directory() {
        // Arrange unchanged                                                                                                                                                                                       
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let temp_path = temp_dir.path();
        let original_dir = env::current_dir().expect("Failed to get current directory");
        env::set_current_dir(temp_path).expect("Failed to change current directory");

        // Create hidden directory and files                                                                                                                                                                       
        let hidden_dir = temp_path.join(".hidden");
        fs::create_dir(&hidden_dir).expect("Failed to create hidden dir");
        let hidden_file_path = hidden_dir.join("secret.txt");
        {
            let mut hidden_file =
                File::create(&hidden_file_path).expect("Failed to create secret.txt");
            write!(hidden_file, "Hidden content").expect("Failed to write to secret.txt");
        }

        let public_file_path = temp_path.join("public.txt");
        {
            let mut public_file =
                File::create(&public_file_path).expect("Failed to create public.txt");
            write!(public_file, "Visible content").expect("Failed to write to public.txt");
        }

        // Act - add empty dirs array                                                                                                                                                                              
        let result = extract_content(&Some("./".to_owned()), &[], &[]);
        let files = result.expect("Expected Some(files) for relative extraction");

        env::set_current_dir(original_dir).expect("Failed to restore current directory");

        // Assert unchanged                                                                                                                                                                                        
        let files_map = files_to_map(&files);
        assert!(
            files_map.contains_key("public.txt"),
            "Expected 'public.txt' to be present in the extracted files."
        );
        assert!(
            !files_map.contains_key("secret.txt"),
            "Expected 'secret.txt' from the hidden directory to be excluded."
        );
    }
}   
