/// Git diff parsing and analysis functionality
use anyhow::Result;
use colored::*;
use git2::{Diff, DiffDelta, DiffFormat, DiffHunk, DiffLine, DiffOptions, Repository};
use std::collections::HashMap;
use std::path::PathBuf;

/// Analyzes Git diffs and provides structured information about changes
pub struct DiffAnalyzer<'repo> {
    repository: &'repo Repository,
}

/// Type of change in a file
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    // Legacy variants for compatibility
    Addition,
    Deletion,
    Modification,
    Rename,
    Copy,
}

/// Information about changes in a single file
#[derive(Debug, Clone)]
pub struct FileChange {
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub change_type: ChangeType,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk_>,
    pub is_binary: bool,
    pub language: Option<String>,
}

/// A diff hunk with context
#[derive(Debug, Clone)]
pub struct DiffHunk_ {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine_>,
}

/// A line in a diff hunk
#[derive(Debug, Clone)]
pub struct DiffLine_ {
    pub origin: char,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

/// Summary of all changes in a diff
#[derive(Debug, Clone)]
pub struct DiffSummary {
    pub files_changed: usize,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files: Vec<FileChange>,
    pub binary_files: usize,
    pub language_breakdown: HashMap<String, (usize, usize)>, // lang -> (additions, deletions)
}

impl<'repo> DiffAnalyzer<'repo> {
    /// Create a new diff analyzer for a repository
    pub fn new(repository: &'repo Repository) -> Self {
        Self { repository }
    }

    /// Analyze staged changes (index vs HEAD)
    pub fn analyze_staged(&self) -> Result<DiffSummary> {
        let tree = if let Ok(head) = self.repository.head() {
            Some(head.peel_to_tree()?)
        } else {
            None
        };

        let diff = self.repository.diff_tree_to_index(
            tree.as_ref(),
            None, // Use current index
            Some(&mut DiffOptions::new()),
        )?;

        self.analyze_diff(&diff)
    }

    /// Analyze unstaged changes (index vs working directory)
    pub fn analyze_unstaged(&self) -> Result<DiffSummary> {
        let diff = self.repository.diff_index_to_workdir(
            None, // Use current index
            Some(&mut DiffOptions::new()),
        )?;

        self.analyze_diff(&diff)
    }

    /// Analyze changes between two commits
    pub fn analyze_between_commits(
        &self,
        old_commit: &str,
        new_commit: &str,
    ) -> Result<DiffSummary> {
        let old_tree = self
            .repository
            .revparse_single(old_commit)?
            .peel_to_tree()?;
        let new_tree = self
            .repository
            .revparse_single(new_commit)?
            .peel_to_tree()?;

        let diff = self.repository.diff_tree_to_tree(
            Some(&old_tree),
            Some(&new_tree),
            Some(&mut DiffOptions::new()),
        )?;

        self.analyze_diff(&diff)
    }

    /// Analyze a specific diff object
    fn analyze_diff(&self, diff: &Diff) -> Result<DiffSummary> {
        let mut summary = DiffSummary {
            files_changed: 0,
            total_additions: 0,
            total_deletions: 0,
            files: Vec::new(),
            binary_files: 0,
            language_breakdown: HashMap::new(),
        };

        // Collect file changes
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(file_change) = self.process_file_delta(&delta) {
                    summary.files.push(file_change);
                }
                true
            },
            Some(&mut |_delta, _binary| {
                summary.binary_files += 1;
                true
            }),
            Some(&mut |_delta, _hunk| {
                // This callback is called for each hunk, but we'll process hunks
                // separately when we analyze the detailed diff
                true
            }),
            Some(&mut |_delta, _hunk, line| {
                // Line-by-line processing happens in detailed analysis
                match line.origin() {
                    '+' => summary.total_additions += 1,
                    '-' => summary.total_deletions += 1,
                    _ => {}
                }
                true
            }),
        )?;

        // Note: Detailed hunk and line analysis would require more complex
        // processing. For now, we'll focus on file-level statistics.

        summary.files_changed = summary.files.len();

        // Calculate language breakdown
        for file in &summary.files {
            if let Some(lang) = &file.language {
                let entry = summary
                    .language_breakdown
                    .entry(lang.clone())
                    .or_insert((0, 0));
                entry.0 += file.additions;
                entry.1 += file.deletions;
            }
        }

        Ok(summary)
    }

    /// Process a file delta into a FileChange
    fn process_file_delta(&self, delta: &DiffDelta) -> Option<FileChange> {
        let old_path = delta.old_file().path().map(PathBuf::from);
        let new_path = delta.new_file().path().map(PathBuf::from);

        let change_type = match delta.status() {
            git2::Delta::Added => ChangeType::Addition,
            git2::Delta::Deleted => ChangeType::Deletion,
            git2::Delta::Modified => ChangeType::Modification,
            git2::Delta::Renamed => ChangeType::Rename,
            git2::Delta::Copied => ChangeType::Copy,
            _ => return None,
        };

        let is_binary = delta.old_file().is_binary() || delta.new_file().is_binary();

        let language = self.detect_language(&new_path.as_ref().or(old_path.as_ref()));

        Some(FileChange {
            old_path,
            new_path,
            change_type,
            additions: 0, // Will be filled in during detailed analysis
            deletions: 0,
            hunks: Vec::new(),
            is_binary,
            language,
        })
    }

    /// Convert git2::DiffHunk to our DiffHunk_
    fn convert_hunk(&self, hunk: &DiffHunk) -> DiffHunk_ {
        DiffHunk_ {
            header: String::from_utf8_lossy(hunk.header()).to_string(),
            old_start: hunk.old_start(),
            old_lines: hunk.old_lines(),
            new_start: hunk.new_start(),
            new_lines: hunk.new_lines(),
            lines: Vec::new(), // Will be filled by line callback
        }
    }

    /// Convert git2::DiffLine to our DiffLine_
    fn convert_line(&self, line: &DiffLine) -> DiffLine_ {
        DiffLine_ {
            origin: line.origin(),
            content: String::from_utf8_lossy(line.content()).to_string(),
            old_lineno: line.old_lineno(),
            new_lineno: line.new_lineno(),
        }
    }

    /// Check if a delta matches a file change
    fn delta_matches_file(&self, delta: &DiffDelta, file_change: &FileChange) -> bool {
        let delta_new_path = delta.new_file().path().map(PathBuf::from);
        let delta_old_path = delta.old_file().path().map(PathBuf::from);

        (delta_new_path == file_change.new_path) && (delta_old_path == file_change.old_path)
    }

    /// Detect programming language from file path
    fn detect_language(&self, path: &Option<&PathBuf>) -> Option<String> {
        let path = path.as_ref()?;
        
        // First try extension-based detection
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let language = match extension {
                "rs" => "Rust",
                "js" | "jsx" => "JavaScript",
                "ts" | "tsx" => "TypeScript",
                "py" => "Python",
                "go" => "Go",
                "java" => "Java",
                "kt" | "kts" => "Kotlin",
                "cpp" | "cc" | "cxx" | "c++" => "C++",
                "c" => "C",
                "h" | "hpp" | "hxx" => "C/C++ Header",
                "cs" => "C#",
                "php" => "PHP",
                "rb" => "Ruby",
                "swift" => "Swift",
                "scala" => "Scala",
                "clj" | "cljs" => "Clojure",
                "hs" => "Haskell",
                "ml" | "mli" => "OCaml",
                "fs" | "fsx" | "fsi" => "F#",
                "elm" => "Elm",
                "dart" => "Dart",
                "r" | "R" => "R",
                "m" => "Objective-C",
                "mm" => "Objective-C++",
                "sh" | "bash" | "zsh" => "Shell",
                "fish" => "Fish",
                "ps1" => "PowerShell",
                "sql" => "SQL",
                "html" | "htm" => "HTML",
                "css" => "CSS",
                "scss" | "sass" => "Sass",
                "less" => "Less",
                "xml" => "XML",
                "yaml" | "yml" => "YAML",
                "json" => "JSON",
                "toml" => "TOML",
                "ini" => "INI",
                "cfg" | "conf" => "Config",
                "md" | "markdown" => "Markdown",
                "rst" => "reStructuredText",
                "tex" => "LaTeX",
                "vim" => "Vim Script",
                "dockerfile" => "Dockerfile",
                "makefile" => "Makefile",
                _ => return None,
            };
            
            return Some(language.to_string());
        }
        
        // If no extension, try filename-based detection
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            let language = match filename.to_lowercase().as_str() {
                "dockerfile" => "Dockerfile",
                "makefile" => "Makefile",
                "cmakelists.txt" => "CMake",
                "cargo.toml" | "cargo.lock" => "Rust Config",
                "package.json" | "package-lock.json" => "Node.js Config",
                "pyproject.toml" | "setup.py" | "requirements.txt" => "Python Config",
                "go.mod" | "go.sum" => "Go Config",
                _ => return None,
            };
            
            return Some(language.to_string());
        }
        
        None
    }

    /// Get diff as formatted patch string
    pub fn get_patch_string(&self, diff: &Diff) -> Result<String> {
        let mut patch_string = String::new();

        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => patch_string.push(line.origin()),
                _ => {}
            }
            patch_string.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        })?;

        Ok(patch_string)
    }
}

impl DiffSummary {
    /// Display a formatted summary of the diff
    pub fn display_summary(&self) {
        if self.files_changed == 0 {
            println!("{}", "No changes found".dimmed());
            return;
        }

        println!(
            "\n{}",
            format!("📊 Diff Summary: {} files changed", self.files_changed)
                .bright_blue()
                .bold()
        );

        if self.total_additions > 0 || self.total_deletions > 0 {
            println!(
                "   {} {}, {} {}",
                format!("+{}", self.total_additions).green(),
                if self.total_additions == 1 {
                    "insertion"
                } else {
                    "insertions"
                },
                format!("-{}", self.total_deletions).red(),
                if self.total_deletions == 1 {
                    "deletion"
                } else {
                    "deletions"
                }
            );
        }

        if self.binary_files > 0 {
            println!("   {} binary files", self.binary_files);
        }

        // Show language breakdown
        if !self.language_breakdown.is_empty() {
            println!("\n{}", "📝 Changes by language:".bright_yellow().bold());
            let mut languages: Vec<_> = self.language_breakdown.iter().collect();
            languages.sort_by(|a, b| (a.1 .0 + a.1 .1).cmp(&(b.1 .0 + b.1 .1)).reverse());

            for (lang, (additions, deletions)) in languages.iter().take(5) {
                println!(
                    "   {}: {} {}, {} {}",
                    lang.cyan(),
                    format!("+{}", additions).green(),
                    if *additions == 1 {
                        "insertion"
                    } else {
                        "insertions"
                    },
                    format!("-{}", deletions).red(),
                    if *deletions == 1 {
                        "deletion"
                    } else {
                        "deletions"
                    }
                );
            }
        }

        // Show file changes
        if self.files.len() <= 10 {
            println!("\n{}", "📁 Changed files:".bright_green().bold());
            for file in &self.files {
                self.display_file_change(file);
            }
        } else {
            println!(
                "\n{}",
                format!("📁 {} files changed (showing first 10):", self.files.len())
                    .bright_green()
                    .bold()
            );
            for file in self.files.iter().take(10) {
                self.display_file_change(file);
            }
        }
    }

    fn display_file_change(&self, file: &FileChange) {
        let path_display = file
            .new_path
            .as_ref()
            .or(file.old_path.as_ref())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let change_symbol = match file.change_type {
            ChangeType::Addition | ChangeType::Added => "+".green(),
            ChangeType::Deletion | ChangeType::Deleted => "-".red(),
            ChangeType::Modification | ChangeType::Modified => "~".yellow(),
            ChangeType::Rename | ChangeType::Renamed => "→".blue(),
            ChangeType::Copy | ChangeType::Copied => "C".cyan(),
        };

        let binary_indicator = if file.is_binary { " (binary)" } else { "" };
        let lang_indicator = if let Some(ref lang) = file.language {
            format!(" [{}]", lang).dimmed()
        } else {
            "".normal()
        };

        println!(
            "   {} {}{}{}",
            change_symbol,
            path_display.bright_white(),
            binary_indicator.dimmed(),
            lang_indicator
        );

        if file.additions > 0 || file.deletions > 0 {
            println!(
                "     {} {}, {} {}",
                format!("+{}", file.additions).green(),
                if file.additions == 1 {
                    "insertion"
                } else {
                    "insertions"
                },
                format!("-{}", file.deletions).red(),
                if file.deletions == 1 {
                    "deletion"
                } else {
                    "deletions"
                }
            );
        }
    }

    /// Get a concise summary string for commit messages
    pub fn get_commit_summary(&self) -> String {
        if self.files_changed == 0 {
            return "No changes".to_string();
        }

        let mut parts = Vec::new();

        if self.files_changed == 1 {
            if let Some(file) = self.files.first() {
                let path = file
                    .new_path
                    .as_ref()
                    .or(file.old_path.as_ref())
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");

                match file.change_type {
                    ChangeType::Addition | ChangeType::Added => parts.push(format!("add {}", path)),
                    ChangeType::Deletion | ChangeType::Deleted => {
                        parts.push(format!("remove {}", path))
                    }
                    ChangeType::Modification | ChangeType::Modified => {
                        parts.push(format!("update {}", path))
                    }
                    ChangeType::Rename | ChangeType::Renamed => {
                        parts.push(format!("rename {}", path))
                    }
                    ChangeType::Copy | ChangeType::Copied => parts.push(format!("copy {}", path)),
                }
            }
        } else {
            parts.push(format!("update {} files", self.files_changed));
        }

        if self.total_additions > 0 && self.total_deletions > 0 {
            parts.push(format!(
                "+{} -{}",
                self.total_additions, self.total_deletions
            ));
        } else if self.total_additions > 0 {
            parts.push(format!("+{}", self.total_additions));
        } else if self.total_deletions > 0 {
            parts.push(format!("-{}", self.total_deletions));
        }

        parts.join(", ")
    }

    /// Check if changes look like they might be breaking changes
    pub fn has_potential_breaking_changes(&self) -> bool {
        for file in &self.files {
            // Check for deleted files (potential breaking change)
            if file.change_type == ChangeType::Deletion {
                return true;
            }

            // Check for renamed files in public APIs
            if file.change_type == ChangeType::Rename {
                if let Some(path) = &file.new_path {
                    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                        // Public API files that might indicate breaking changes
                        if matches!(extension, "rs" | "ts" | "js" | "py" | "go" | "java")
                            && (path.to_string_lossy().contains("lib")
                                || path.to_string_lossy().contains("api")
                                || path.to_string_lossy().contains("public"))
                            {
                                return true;
                            }
                    }
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_repo_with_commits() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let repo = Repository::init(temp_dir.path()).expect("Failed to init repo");

        // Set up user config
        let mut config = repo.config().expect("Failed to get config");
        config
            .set_str("user.name", "Test User")
            .expect("Failed to set user name");
        config
            .set_str("user.email", "test@example.com")
            .expect("Failed to set user email");

        // Create initial commit
        let test_file = temp_dir.path().join("test.rs");
        fs::write(
            &test_file,
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        )
        .expect("Failed to write test file");

        let mut index = repo.index().expect("Failed to get index");
        index
            .add_path(Path::new("test.rs"))
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        let signature =
            git2::Signature::new("Test User", "test@example.com", &git2::Time::new(0, 0))
                .expect("Failed to create signature");
        let tree_id = index.write_tree().expect("Failed to write tree");
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");

        let _commit = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .expect("Failed to create initial commit");

        drop(tree); // Explicitly drop the tree to release the borrow
        (temp_dir, repo)
    }

    #[test]
    fn test_diff_analyzer_creation() {
        let (_temp_dir, repo) = create_test_repo_with_commits();
        let _analyzer = DiffAnalyzer::new(&repo);
        // If we get here without panicking, the creation was successful
    }

    #[test]
    fn test_language_detection() {
        let (_temp_dir, repo) = create_test_repo_with_commits();
        let analyzer = DiffAnalyzer::new(&repo);

        let test_cases = vec![
            (PathBuf::from("test.rs"), Some("Rust".to_string())),
            (PathBuf::from("test.js"), Some("JavaScript".to_string())),
            (PathBuf::from("test.py"), Some("Python".to_string())),
            (PathBuf::from("Dockerfile"), Some("Dockerfile".to_string())),
            (PathBuf::from("test.unknown"), None),
        ];

        for (path, expected) in test_cases {
            let result = analyzer.detect_language(&Some(&path));
            assert_eq!(result, expected, "Failed for path: {:?}", path);
        }
    }

    #[test]
    fn test_staged_diff_analysis() {
        let (temp_dir, repo) = create_test_repo_with_commits();
        let analyzer = DiffAnalyzer::new(&repo);

        // Modify the existing file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(
            &test_file,
            "fn main() {\n    println!(\"Hello, Rust!\");\n    println!(\"Modified!\");\n}",
        )
        .expect("Failed to write test file");

        // Stage the changes
        let repo_wrapper = git2::Repository::open(temp_dir.path()).expect("Failed to open repo");
        let mut index = repo_wrapper.index().expect("Failed to get index");
        index
            .add_path(Path::new("test.rs"))
            .expect("Failed to add file to index");
        index.write().expect("Failed to write index");

        // Analyze staged changes
        let diff_summary = analyzer
            .analyze_staged()
            .expect("Failed to analyze staged changes");

        assert_eq!(diff_summary.files_changed, 1);
        assert!(diff_summary.total_additions > 0);

        let file_change = &diff_summary.files[0];
        assert_eq!(file_change.change_type, ChangeType::Modification);
        assert_eq!(file_change.language, Some("Rust".to_string()));
        assert!(!file_change.is_binary);
    }

    #[test]
    fn test_diff_summary_display() {
        // This test mainly ensures the display methods don't panic
        let summary = DiffSummary {
            files_changed: 2,
            total_additions: 10,
            total_deletions: 5,
            files: vec![
                FileChange {
                    old_path: Some(PathBuf::from("src/lib.rs")),
                    new_path: Some(PathBuf::from("src/api.rs")),
                    change_type: ChangeType::Rename,
                    additions: 5,
                    deletions: 2,
                    hunks: Vec::new(),
                    is_binary: false,
                    language: Some("Rust".to_string()),
                },
                FileChange {
                    old_path: None,
                    new_path: Some(PathBuf::from("added.js")),
                    change_type: ChangeType::Addition,
                    additions: 5,
                    deletions: 3,
                    hunks: Vec::new(),
                    is_binary: false,
                    language: Some("JavaScript".to_string()),
                },
            ],
            binary_files: 0,
            language_breakdown: {
                let mut map = HashMap::new();
                map.insert("Rust".to_string(), (5, 2));
                map.insert("JavaScript".to_string(), (5, 3));
                map
            },
        };

        // These methods shouldn't panic
        summary.display_summary();
        let commit_summary = summary.get_commit_summary();
        assert!(!commit_summary.is_empty());

        let has_breaking = summary.has_potential_breaking_changes();
        // Should detect rename as potential breaking change
        assert!(has_breaking);
    }
}

/// Simple file change struct for branch comparison
#[derive(Debug, Clone)]
pub struct SimpleFileChange {
    pub path: String,
    pub status: ChangeType,
}
