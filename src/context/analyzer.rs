use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Represents a file with its relevance score and metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileScore {
    pub path: String,
    pub relevance_score: f32,
    pub size_bytes: u64,
    pub modified_time: SystemTime,
    pub file_type: FileType,
    pub importance_factors: Vec<ImportanceFactor>,
}

/// Different types of files in a project
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FileType {
    SourceCode,
    Configuration,
    Documentation,
    Test,
    Data,
    Unknown,
}

/// Factors that contribute to file importance
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ImportanceFactor {
    EntryPoint,
    RecentlyModified,
    ConfigFile,
    MainModule,
    TestFile,
    Documentation,
    SmallSize,
    FrequentlyAccessed,
    HighlyReferenced, // File is referenced by many other files
    DependencyRoot,   // File that others depend on
    RecentDependency, // File that recently changed dependencies point to
}

/// Analyzes files and assigns relevance scores
pub struct FileAnalyzer {
    /// File extension to type mapping
    extension_map: HashMap<String, FileType>,
    /// Priority patterns for important files
    priority_patterns: Vec<String>,
}

impl FileAnalyzer {
    pub fn new() -> Self {
        let mut extension_map = HashMap::new();

        // Source code extensions
        extension_map.insert("rs".to_string(), FileType::SourceCode);
        extension_map.insert("js".to_string(), FileType::SourceCode);
        extension_map.insert("ts".to_string(), FileType::SourceCode);
        extension_map.insert("py".to_string(), FileType::SourceCode);
        extension_map.insert("java".to_string(), FileType::SourceCode);
        extension_map.insert("kt".to_string(), FileType::SourceCode); // Kotlin
        extension_map.insert("go".to_string(), FileType::SourceCode);
        extension_map.insert("cpp".to_string(), FileType::SourceCode);
        extension_map.insert("c".to_string(), FileType::SourceCode);

        // Configuration files
        extension_map.insert("toml".to_string(), FileType::Configuration);
        extension_map.insert("json".to_string(), FileType::Configuration);
        extension_map.insert("yaml".to_string(), FileType::Configuration);
        extension_map.insert("yml".to_string(), FileType::Configuration);
        extension_map.insert("ini".to_string(), FileType::Configuration);
        extension_map.insert("conf".to_string(), FileType::Configuration);
        extension_map.insert("xml".to_string(), FileType::Configuration); // Maven pom.xml, Android
        extension_map.insert("gradle".to_string(), FileType::Configuration); // Gradle build files
        extension_map.insert("properties".to_string(), FileType::Configuration); // Java properties

        // Documentation
        extension_map.insert("md".to_string(), FileType::Documentation);
        extension_map.insert("rst".to_string(), FileType::Documentation);
        extension_map.insert("txt".to_string(), FileType::Documentation);

        // Test files (will be refined by naming patterns)
        // Data files
        extension_map.insert("csv".to_string(), FileType::Data);
        extension_map.insert("json".to_string(), FileType::Data); // Can be both config and data
        extension_map.insert("xml".to_string(), FileType::Data);

        let priority_patterns = vec![
            "main.rs".to_string(),
            "lib.rs".to_string(),
            "mod.rs".to_string(),
            "index.js".to_string(),
            "index.ts".to_string(),
            "main.py".to_string(),
            "__init__.py".to_string(),
            "main.go".to_string(),
            "Main.java".to_string(),
            "Application.java".to_string(),
            "Main.kt".to_string(),
            "Application.kt".to_string(),
            "Cargo.toml".to_string(),
            "package.json".to_string(),
            "pyproject.toml".to_string(),
            "go.mod".to_string(),
            "pom.xml".to_string(),
            "build.gradle".to_string(),
            "build.gradle.kts".to_string(),
            "README.md".to_string(),
        ];

        Self {
            extension_map,
            priority_patterns,
        }
    }

    /// Analyze a file and return its score
    pub fn analyze_file(&self, path: &Path) -> Result<FileScore> {
        let metadata = path.metadata()?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let file_type = self.determine_file_type(path);
        let mut importance_factors = Vec::new();
        let mut base_score = 1.0;

        // Check for priority patterns
        if self
            .priority_patterns
            .iter()
            .any(|pattern| file_name.contains(pattern))
        {
            importance_factors.push(ImportanceFactor::EntryPoint);
            base_score += 2.0;
        }

        // Check if it's a main module
        if file_name == "main.rs"
            || file_name == "main.py"
            || file_name == "index.js"
            || file_name == "main.go"
            || file_name == "Main.java"
            || file_name == "Main.kt"
            || file_name == "Application.java"
            || file_name == "Application.kt"
        {
            importance_factors.push(ImportanceFactor::MainModule);
            base_score += 1.5;
        }

        // Check if recently modified (within last 30 days)
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() < 30 * 24 * 3600 {
                    // 30 days
                    importance_factors.push(ImportanceFactor::RecentlyModified);
                    base_score += 1.0;
                }
            }
        }

        // Check if it's a test file
        if self.is_test_file(path) {
            importance_factors.push(ImportanceFactor::TestFile);
            base_score += 0.5; // Tests are important but lower priority
        }

        // Check file size (prefer smaller files for context)
        let size = metadata.len();
        if size < 10_000 {
            // Files smaller than 10KB
            importance_factors.push(ImportanceFactor::SmallSize);
            base_score += 0.5;
        } else if size > 100_000 {
            // Files larger than 100KB
            base_score -= 0.5; // Penalize very large files
        }

        // File type scoring
        base_score += match file_type {
            FileType::SourceCode => 1.0,
            FileType::Configuration => 0.8,
            FileType::Documentation => 0.6,
            FileType::Test => 0.5,
            FileType::Data => 0.2,
            FileType::Unknown => 0.1,
        };

        // Normalize score to 0-1 range
        let relevance_score = (base_score / 10.0_f32).min(1.0).max(0.0);

        Ok(FileScore {
            path: path.to_string_lossy().to_string(),
            relevance_score,
            size_bytes: size,
            modified_time: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            file_type,
            importance_factors,
        })
    }

    /// Determine the type of a file based on extension and name
    fn determine_file_type(&self, path: &Path) -> FileType {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            if let Some(&ref file_type) = self.extension_map.get(extension) {
                return file_type.clone();
            }
        }

        // Special cases based on file name
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            if file_name.starts_with('.')
                && (file_name.contains("rc") || file_name.contains("config"))
            {
                return FileType::Configuration;
            }

            if file_name.to_lowercase().contains("readme") {
                return FileType::Documentation;
            }
        }

        FileType::Unknown
    }

    /// Check if a file is a test file based on path and name
    fn is_test_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        // Check for test directories
        if path_str.contains("/test/")
            || path_str.contains("/tests/")
            || path_str.starts_with("test/")
            || path_str.starts_with("tests/")
        {
            return true;
        }

        // Check for test file naming patterns
        if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
            let name_lower = file_name.to_lowercase();
            return name_lower.starts_with("test_")
                || name_lower.ends_with("_test.rs")
                || name_lower.ends_with("_test.py")
                || name_lower.ends_with("_test.go")
                || name_lower.ends_with("test.java")
                || name_lower.ends_with("test.kt")
                || name_lower.ends_with(".test.js")
                || name_lower.ends_with(".spec.js");
        }

        false
    }

    /// Analyze multiple files and return sorted by relevance
    pub fn analyze_files(&self, paths: &[&Path]) -> Result<Vec<FileScore>> {
        let mut scores: Vec<FileScore> = Vec::new();

        for path in paths {
            if let Ok(score) = self.analyze_file(path) {
                scores.push(score);
            }
        }

        // Sort by relevance score (highest first)
        scores.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(scores)
    }

    /// Filter files by query keywords
    pub fn filter_by_query(&self, files: &[FileScore], query: Option<&str>) -> Vec<FileScore> {
        if let Some(q) = query {
            let keywords: Vec<&str> = q.split_whitespace().collect();

            files
                .iter()
                .filter(|file| {
                    let file_content = file.path.to_lowercase();
                    keywords
                        .iter()
                        .any(|keyword| file_content.contains(&keyword.to_lowercase()))
                })
                .cloned()
                .collect()
        } else {
            files.to_vec()
        }
    }

    /// Analyze file dependencies and enhance scores based on relationships
    pub fn analyze_dependencies(&self, files: &mut [FileScore]) -> Result<()> {
        let dependency_map = self.build_dependency_map(files)?;

        // Calculate reference counts for each file
        let mut reference_counts: HashMap<String, usize> = HashMap::new();
        for dependencies in dependency_map.values() {
            for dep in dependencies {
                *reference_counts.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        // Enhance scores based on dependency relationships
        for file_score in files.iter_mut() {
            let file_path = &file_score.path;
            let ref_count = reference_counts.get(file_path).unwrap_or(&0);

            // High reference count indicates an important file
            if *ref_count >= 3 {
                if !file_score
                    .importance_factors
                    .contains(&ImportanceFactor::HighlyReferenced)
                {
                    file_score
                        .importance_factors
                        .push(ImportanceFactor::HighlyReferenced);
                    file_score.relevance_score = (file_score.relevance_score + 0.3).min(1.0);
                }
            }

            // Files that others depend on but have few dependencies themselves (root files)
            if *ref_count >= 2 {
                let own_deps = dependency_map
                    .get(file_path)
                    .map(|deps| deps.len())
                    .unwrap_or(0);
                if own_deps <= 1 {
                    if !file_score
                        .importance_factors
                        .contains(&ImportanceFactor::DependencyRoot)
                    {
                        file_score
                            .importance_factors
                            .push(ImportanceFactor::DependencyRoot);
                        file_score.relevance_score = (file_score.relevance_score + 0.2).min(1.0);
                    }
                }
            }
        }

        Ok(())
    }

    /// Build a dependency map showing which files depend on which other files
    fn build_dependency_map(
        &self,
        files: &[FileScore],
    ) -> Result<HashMap<String, HashSet<String>>> {
        let mut dependency_map: HashMap<String, HashSet<String>> = HashMap::new();

        // Create a set of all file paths for quick lookup (without directory prefix)
        let file_names: HashSet<String> = files
            .iter()
            .filter_map(|f| {
                Path::new(&f.path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string())
            })
            .collect();

        for file_score in files {
            let file_path = &file_score.path;
            let mut dependencies = HashSet::new();

            // Analyze file content for imports/dependencies
            if let Ok(content) = fs::read_to_string(file_path) {
                dependencies.extend(self.extract_dependencies(&content, &file_names, file_path)?);
            }

            dependency_map.insert(file_path.clone(), dependencies);
        }

        Ok(dependency_map)
    }

    /// Extract dependencies from file content based on language-specific patterns
    fn extract_dependencies(
        &self,
        content: &str,
        available_files: &HashSet<String>,
        file_path: &str,
    ) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        let path = Path::new(file_path);

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => {
                // Rust: use, mod, extern crate
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("use ")
                        || line.starts_with("mod ")
                        || line.starts_with("extern crate ")
                    {
                        // Extract potential file references
                        self.extract_rust_dependencies(line, available_files, &mut dependencies);
                    }
                }
            }
            Some("js") | Some("ts") => {
                // JavaScript/TypeScript: import, require
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("import ") || line.contains("require(") {
                        self.extract_js_dependencies(line, available_files, &mut dependencies);
                    }
                }
            }
            Some("py") => {
                // Python: import, from ... import
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("import ") || line.starts_with("from ") {
                        self.extract_python_dependencies(line, available_files, &mut dependencies);
                    }
                }
            }
            Some("go") => {
                // Go: import
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("import ") {
                        self.extract_go_dependencies(line, available_files, &mut dependencies);
                    }
                }
            }
            Some("java") | Some("kt") => {
                // Java/Kotlin: import
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("import ") {
                        self.extract_java_dependencies(line, available_files, &mut dependencies);
                    }
                }
            }
            _ => {}
        }

        Ok(dependencies)
    }

    /// Extract Rust dependencies (simplified)
    fn extract_rust_dependencies(
        &self,
        line: &str,
        available_files: &HashSet<String>,
        dependencies: &mut HashSet<String>,
    ) {
        // Simple pattern matching for mod declarations
        if line.starts_with("mod ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mod_name = parts[1].trim_end_matches(';');
                let rust_file = format!("{}.rs", mod_name);
                if available_files.contains(&rust_file) {
                    dependencies.insert(rust_file);
                }
            }
        }
    }

    /// Extract JavaScript/TypeScript dependencies (simplified)
    fn extract_js_dependencies(
        &self,
        line: &str,
        available_files: &HashSet<String>,
        dependencies: &mut HashSet<String>,
    ) {
        // Look for relative imports like "./module" or "../utils"
        for file_name in available_files {
            let base_name = file_name.trim_end_matches(".js").trim_end_matches(".ts");
            if line.contains(&format!("./{}", base_name))
                || line.contains(&format!("../{}", base_name))
            {
                dependencies.insert(file_name.clone());
            }
        }
    }

    /// Extract Python dependencies (simplified)
    fn extract_python_dependencies(
        &self,
        line: &str,
        available_files: &HashSet<String>,
        dependencies: &mut HashSet<String>,
    ) {
        for file_name in available_files {
            let module_name = file_name.trim_end_matches(".py");
            if line.contains(module_name) && !line.contains("__") {
                dependencies.insert(file_name.clone());
            }
        }
    }

    /// Extract Go dependencies (simplified)  
    fn extract_go_dependencies(
        &self,
        _line: &str,
        _available_files: &HashSet<String>,
        _dependencies: &mut HashSet<String>,
    ) {
        // Go imports are typically packages, not individual files
        // More complex analysis would be needed for local file dependencies
    }

    /// Extract Java/Kotlin dependencies (simplified)
    fn extract_java_dependencies(
        &self,
        _line: &str,
        _available_files: &HashSet<String>,
        _dependencies: &mut HashSet<String>,
    ) {
        // Java imports are typically packages, not individual files
        // More complex analysis would be needed for local file dependencies
    }
}

impl Default for FileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_type_detection() {
        let analyzer = FileAnalyzer::new();

        assert_eq!(
            analyzer.determine_file_type(Path::new("main.rs")),
            FileType::SourceCode
        );
        assert_eq!(
            analyzer.determine_file_type(Path::new("config.toml")),
            FileType::Configuration
        );
        assert_eq!(
            analyzer.determine_file_type(Path::new("README.md")),
            FileType::Documentation
        );
    }

    #[test]
    fn test_test_file_detection() {
        let analyzer = FileAnalyzer::new();

        assert!(analyzer.is_test_file(Path::new("tests/mod.rs")));
        assert!(analyzer.is_test_file(Path::new("test_utils.py")));
        assert!(analyzer.is_test_file(Path::new("component.test.js")));
        assert!(!analyzer.is_test_file(Path::new("main.rs")));
    }

    #[test]
    fn test_file_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.rs");
        fs::write(&file_path, "fn main() {}").unwrap();

        let analyzer = FileAnalyzer::new();
        let score = analyzer.analyze_file(&file_path).unwrap();

        assert_eq!(score.file_type, FileType::SourceCode);
        assert!(score.relevance_score > 0.0);
        assert!(score
            .importance_factors
            .contains(&ImportanceFactor::EntryPoint));
        assert!(score
            .importance_factors
            .contains(&ImportanceFactor::MainModule));
    }
}
