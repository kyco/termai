use crate::context::{
    chunker::{ChunkingStrategy, ContextChunker},
    SmartContext,
};
use crate::path::model::Files;
use std::fs;
use std::path::Path;

pub fn extract_content(
    dir: &Option<String>,
    dirs: &[String],
    exclude: &[String],
) -> Option<Vec<Files>> {
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

/// Smart context-aware content extraction with preview option
/// Uses the SmartContext system to automatically discover relevant files
#[allow(dead_code)]
pub async fn smart_extract_content_with_preview(
    dir: &Option<String>,
    dirs: &[String],
    exclude: &[String],
    query: Option<&str>,
    show_preview: bool,
) -> Option<Vec<Files>> {
    // If manual paths are provided, use traditional extraction
    if dir.is_some() || !dirs.is_empty() {
        return extract_content(dir, dirs, exclude);
    }

    // Use smart context discovery for current directory
    let current_dir = std::env::current_dir().ok()?;
    let smart_context =
        SmartContext::from_project(&current_dir).unwrap_or_else(|_| SmartContext::new());

    // Get project info and collect files
    if let Ok(project_info) = smart_context.detect_project(&current_dir) {
        if let Ok(candidate_files) =
            smart_context.collect_candidate_files(&current_dir, &project_info)
        {
            let file_refs: Vec<&std::path::Path> =
                candidate_files.iter().map(|pb| pb.as_path()).collect();
            if let Ok(file_scores) = smart_context.analyzer.analyze_files(&file_refs) {
                let filtered_scores = smart_context.analyzer.filter_by_query(&file_scores, query);
                if let Ok(selected_scores) =
                    smart_context.optimizer.optimize_files(&filtered_scores)
                {
                    // Show preview if requested
                    if show_preview && !selected_scores.is_empty() {
                        let preview = smart_context.preview_context_selection(&selected_scores);
                        println!("{}", preview);

                        // Ask for user confirmation
                        print!("Proceed with this context selection? (y/N): ");
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();

                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            if !input.trim().to_lowercase().starts_with('y') {
                                println!("Smart context selection cancelled. Using manual extraction fallback.");
                                return extract_content(&Some(".".to_string()), &[], exclude);
                            }
                        }
                    }

                    // Convert to Files format and return
                    if let Ok(files) = smart_context.scores_to_files(&selected_scores).await {
                        return Some(files);
                    }
                }
            }
        }
    }

    // Fallback
    extract_content(&Some(".".to_string()), &[], exclude)
}

/// Smart context-aware content extraction
/// Uses the SmartContext system to automatically discover relevant files
#[allow(dead_code)]
pub async fn smart_extract_content(
    dir: &Option<String>,
    dirs: &[String],
    exclude: &[String],
    query: Option<&str>,
) -> Option<Vec<Files>> {
    smart_extract_content_with_preview(dir, dirs, exclude, query, false).await
}

/// Enhanced extract_content that can optionally use smart context discovery
#[allow(dead_code)]
pub async fn extract_content_with_smart_fallback(
    dir: &Option<String>,
    dirs: &[String],
    exclude: &[String],
    use_smart_context: bool,
    query: Option<&str>,
    show_preview: bool,
    use_chunking: bool,
    chunk_strategy: Option<&str>,
    max_tokens: Option<usize>,
) -> Option<Vec<Files>> {
    if use_smart_context {
        if use_chunking {
            smart_extract_with_chunking(
                dir,
                dirs,
                exclude,
                query,
                show_preview,
                chunk_strategy,
                max_tokens,
            )
            .await
        } else {
            smart_extract_content_with_preview(dir, dirs, exclude, query, show_preview).await
        }
    } else {
        extract_content(dir, dirs, exclude)
    }
}

/// Smart extraction with chunking support for large projects
#[allow(dead_code)]
pub async fn smart_extract_with_chunking(
    dir: &Option<String>,
    dirs: &[String],
    exclude: &[String],
    query: Option<&str>,
    show_preview: bool,
    chunk_strategy: Option<&str>,
    max_tokens: Option<usize>,
) -> Option<Vec<Files>> {
    // If manual paths are provided, use traditional extraction
    if dir.is_some() || !dirs.is_empty() {
        return extract_content(dir, dirs, exclude);
    }

    // Use smart context discovery for current directory
    let current_dir = std::env::current_dir().ok()?;
    let smart_context =
        SmartContext::from_project(&current_dir).unwrap_or_else(|_| SmartContext::new());

    // Get project info and analyze files
    if let Ok(project_info) = smart_context.detect_project(&current_dir) {
        if let Ok(candidate_files) =
            smart_context.collect_candidate_files(&current_dir, &project_info)
        {
            let file_refs: Vec<&std::path::Path> =
                candidate_files.iter().map(|pb| pb.as_path()).collect();
            if let Ok(file_scores) = smart_context.analyzer.analyze_files(&file_refs) {
                let filtered_scores = smart_context.analyzer.filter_by_query(&file_scores, query);

                // Check if project is large enough to warrant chunking
                let total_estimated_tokens: usize = filtered_scores
                    .iter()
                    .map(|score| {
                        smart_context
                            .optimizer
                            .estimate_tokens(std::path::Path::new(&score.path))
                            .unwrap_or(100)
                    })
                    .sum();

                let token_budget =
                    max_tokens.unwrap_or_else(|| smart_context.optimizer.get_token_budget());

                if total_estimated_tokens > token_budget * 2 {
                    // More than 2x budget = definitely chunk
                    println!("🚨 Large project detected!");
                    println!("   📊 Estimated tokens: ~{}", total_estimated_tokens);
                    println!("   💾 Available budget: {}", token_budget);
                    println!(
                        "   📦 Ratio: {:.1}x over budget",
                        total_estimated_tokens as f32 / token_budget as f32
                    );
                    println!();
                    println!("🔄 Switching to chunked analysis mode...");

                    // Parse chunking strategy
                    let strategy = match chunk_strategy.unwrap_or("hierarchical") {
                        "module" => ChunkingStrategy::ModuleBased,
                        "functional" => ChunkingStrategy::FunctionalBased,
                        "token" => ChunkingStrategy::TokenBased,
                        _ => ChunkingStrategy::Hierarchical,
                    };

                    // Create chunker and initialize multi-session
                    let chunker = ContextChunker::new(token_budget, strategy);
                    if let Ok(chunks) = chunker.create_chunks(&filtered_scores).await {
                        // Show chunking preview
                        if show_preview {
                            println!("📋 **Chunked Analysis Plan**");
                            println!("═══════════════════════════");
                            println!("Created {} manageable chunks:\n", chunks.len());

                            for (i, chunk) in chunks.iter().enumerate() {
                                let chunk_icon = match chunk.chunk_type {
                                    crate::context::chunker::ChunkType::Overview => "📋",
                                    crate::context::chunker::ChunkType::Core => "🎯",
                                    crate::context::chunker::ChunkType::Utils => "🔧",
                                    crate::context::chunker::ChunkType::Tests => "🧪",
                                    crate::context::chunker::ChunkType::Config => "⚙️",
                                    crate::context::chunker::ChunkType::Docs => "📚",
                                };

                                println!(
                                    "{}. {} {} - {}",
                                    i + 1,
                                    chunk_icon,
                                    chunk.name,
                                    chunk.description
                                );
                                println!(
                                    "   📄 {} files, ~{} tokens (priority: {:.1})",
                                    chunk.files.len(),
                                    chunk.estimated_tokens,
                                    chunk.priority
                                );
                                println!();
                            }

                            println!("💡 **Recommended Workflow:**");
                            println!("   1. Start with Overview chunk for project understanding");
                            println!("   2. Focus on Core chunks for main functionality");
                            println!("   3. Analyze other chunks as needed");
                            println!("   4. Use multi-session commands to navigate between chunks");
                            println!();

                            print!("Proceed with chunked analysis? (Y/n): ");
                            std::io::Write::flush(&mut std::io::stdout()).unwrap();

                            let mut input = String::new();
                            if std::io::stdin().read_line(&mut input).is_ok() {
                                if input.trim().to_lowercase().starts_with('n') {
                                    println!("Chunked analysis cancelled. Using regular smart context with truncation.");
                                    if let Ok(selected_scores) =
                                        smart_context.optimizer.optimize_files(&filtered_scores)
                                    {
                                        if let Ok(files) =
                                            smart_context.scores_to_files(&selected_scores).await
                                        {
                                            return Some(files);
                                        }
                                    }
                                }
                            }
                        }

                        // Return the first chunk (usually Overview) to start the conversation
                        if let Some(first_chunk) = chunks.first() {
                            println!("🚀 Starting with: {}", first_chunk.name);
                            println!(
                                "   Use /chunks to see all chunks and /switch <n> to change focus"
                            );
                            println!();
                            return Some(first_chunk.files.clone());
                        }
                    }
                }

                // Fall back to regular smart context if chunking not needed/failed
                if let Ok(selected_scores) =
                    smart_context.optimizer.optimize_files(&filtered_scores)
                {
                    if show_preview && !selected_scores.is_empty() {
                        let preview = smart_context.preview_context_selection(&selected_scores);
                        println!("{}", preview);

                        print!("Proceed with this context selection? (y/N): ");
                        std::io::Write::flush(&mut std::io::stdout()).unwrap();

                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            if !input.trim().to_lowercase().starts_with('y') {
                                return extract_content(&Some(".".to_string()), &[], exclude);
                            }
                        }
                    }

                    if let Ok(files) = smart_context.scores_to_files(&selected_scores).await {
                        return Some(files);
                    }
                }
            }
        }
    }

    // Final fallback
    extract_content(&Some(".".to_string()), &[], exclude)
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
        let result = extract_content(
            &Some(temp_path.to_str().unwrap().to_owned()),
            &[],
            &excludes,
        );
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
        // Arrange unchanged
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = File::create(&file_path).expect("Failed to create test.txt");
            write!(file, "Relative file").expect("Failed to write to test.txt");
        }
        let original_dir = env::current_dir().expect("Failed to get current directory");
        env::set_current_dir(temp_dir.path()).expect("Failed to change current directory");

        // Act - add empty dirs array
        let result = extract_content(&Some("./".to_owned()), &[], &[]);
        let files = result.expect("Expected Some(files) when using a relative path");

        env::set_current_dir(original_dir).expect("Failed to restore current directory");

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
