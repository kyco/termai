use crate::context::{ProjectInfo, ProjectType};
use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Trait for detecting different project types
pub trait ProjectDetector: Send + Sync {
    /// Attempt to detect a project at the given path
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>>;

    /// Get the project type this detector handles
    fn project_type(&self) -> ProjectType;

    /// Get the confidence threshold for detection
    fn confidence_threshold(&self) -> f32 {
        0.7
    }
}

/// Detector for Rust projects
pub struct RustProjectDetector;

impl RustProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for RustProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        let cargo_toml = path.join("Cargo.toml");

        if !cargo_toml.exists() {
            return Ok(None);
        }

        let mut entry_points = Vec::new();
        let mut important_files = vec!["Cargo.toml".to_string()];

        // Check for common Rust entry points
        let src_main = path.join("src/main.rs");
        let src_lib = path.join("src/lib.rs");

        if src_main.exists() {
            entry_points.push("src/main.rs".to_string());
            important_files.push("src/main.rs".to_string());
        }

        if src_lib.exists() {
            entry_points.push("src/lib.rs".to_string());
            important_files.push("src/lib.rs".to_string());
        }

        // Add other important Rust files
        if path.join("README.md").exists() {
            important_files.push("README.md".to_string());
        }

        let confidence = if entry_points.is_empty() { 0.6 } else { 0.9 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Rust,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Rust
    }
}

/// Detector for JavaScript/TypeScript projects
pub struct JavaScriptProjectDetector;

impl JavaScriptProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for JavaScriptProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        let package_json = path.join("package.json");

        if !package_json.exists() {
            return Ok(None);
        }

        let mut entry_points = Vec::new();
        let mut important_files = vec!["package.json".to_string()];

        // Check for common JS entry points
        for entry in &[
            "index.js",
            "index.ts",
            "src/index.js",
            "src/index.ts",
            "src/main.js",
            "src/main.ts",
        ] {
            if path.join(entry).exists() {
                entry_points.push(entry.to_string());
                important_files.push(entry.to_string());
            }
        }

        // Add other important files
        for file in &[
            "README.md",
            "tsconfig.json",
            ".eslintrc.json",
            "webpack.config.js",
        ] {
            if path.join(file).exists() {
                important_files.push(file.to_string());
            }
        }

        let confidence = if entry_points.is_empty() { 0.6 } else { 0.9 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::JavaScript,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::JavaScript
    }
}

/// Detector for Python projects
pub struct PythonProjectDetector;

impl PythonProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for PythonProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        // Check for Python project indicators
        let indicators = [
            "pyproject.toml",
            "setup.py",
            "requirements.txt",
            "Pipfile",
            "poetry.lock",
        ];

        let has_indicator = indicators.iter().any(|file| path.join(file).exists());

        if !has_indicator {
            return Ok(None);
        }

        let mut entry_points = Vec::new();
        let mut important_files = Vec::new();

        // Add found indicators to important files
        for indicator in indicators {
            if path.join(indicator).exists() {
                important_files.push(indicator.to_string());
            }
        }

        // Look for common Python entry points
        for entry in &["main.py", "__init__.py", "app.py", "src/main.py"] {
            if path.join(entry).exists() {
                entry_points.push(entry.to_string());
                important_files.push(entry.to_string());
            }
        }

        if path.join("README.md").exists() {
            important_files.push("README.md".to_string());
        }

        let confidence = if entry_points.is_empty() { 0.6 } else { 0.8 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Python,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Python
    }
}

/// Detector for Go projects
pub struct GoProjectDetector;

impl GoProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for GoProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        // Check for Go project indicators
        let indicators = [
            "go.mod",
            "go.sum",
            "Gopkg.toml", // dep tool
            "glide.yaml", // glide tool
        ];

        let has_indicator = indicators.iter().any(|file| path.join(file).exists());

        if !has_indicator {
            // Also check for .go files in root or common directories
            let go_dirs = [".", "cmd", "internal", "pkg"];
            let has_go_files = go_dirs.iter().any(|dir| {
                let dir_path = if *dir == "." {
                    path.to_path_buf()
                } else {
                    path.join(dir)
                };
                if dir_path.exists() {
                    dir_path.read_dir().map_or(false, |mut entries| {
                        entries.any(|entry| {
                            entry.map_or(false, |e| {
                                e.path().extension().and_then(|ext| ext.to_str()) == Some("go")
                            })
                        })
                    })
                } else {
                    false
                }
            });

            if !has_go_files {
                return Ok(None);
            }
        }

        let mut entry_points = Vec::new();
        let mut important_files = Vec::new();

        // Add found indicators to important files
        for indicator in indicators {
            if path.join(indicator).exists() {
                important_files.push(indicator.to_string());
            }
        }

        // Look for common Go entry points
        for entry in &["main.go", "cmd/main.go", "cmd/*/main.go"] {
            if entry.contains('*') {
                // Handle wildcard patterns like cmd/*/main.go
                if let Ok(entries) = path.join("cmd").read_dir() {
                    for dir_entry in entries.flatten() {
                        if dir_entry.file_type().map_or(false, |ft| ft.is_dir()) {
                            let main_path = dir_entry.path().join("main.go");
                            if main_path.exists() {
                                let relative_path = format!(
                                    "cmd/{}/main.go",
                                    dir_entry.file_name().to_string_lossy()
                                );
                                entry_points.push(relative_path.clone());
                                important_files.push(relative_path);
                            }
                        }
                    }
                }
            } else if path.join(entry).exists() {
                entry_points.push(entry.to_string());
                important_files.push(entry.to_string());
            }
        }

        if path.join("README.md").exists() {
            important_files.push("README.md".to_string());
        }

        let confidence = if entry_points.is_empty() { 0.6 } else { 0.8 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Go,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Go
    }
}

/// Detector for Kotlin projects
pub struct KotlinProjectDetector;

impl KotlinProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for KotlinProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        // Check for Kotlin project indicators
        let indicators = [
            "build.gradle.kts", // Kotlin DSL
            "settings.gradle.kts",
            "gradle.properties",
        ];

        let has_gradle_kotlin = indicators.iter().any(|file| path.join(file).exists());
        let has_regular_gradle =
            path.join("build.gradle").exists() || path.join("settings.gradle").exists();

        if !has_gradle_kotlin && !has_regular_gradle {
            // Also check for Maven with Kotlin
            if !path.join("pom.xml").exists() {
                return Ok(None);
            }
        }

        // Check for Kotlin files
        let kotlin_dirs = ["src/main/kotlin", "src/test/kotlin", "src", "."];
        let has_kotlin_files = kotlin_dirs.iter().any(|dir| {
            let dir_path = if *dir == "." {
                path.to_path_buf()
            } else {
                path.join(dir)
            };
            self.has_kotlin_files_recursive(&dir_path)
        });

        if !has_kotlin_files {
            return Ok(None);
        }

        let mut entry_points = Vec::new();
        let mut important_files = Vec::new();

        // Add build files to important files
        for file in &[
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
            "gradle.properties",
            "pom.xml",
        ] {
            if path.join(file).exists() {
                important_files.push(file.to_string());
            }
        }

        // Look for main Kotlin files
        for entry in &[
            "src/main/kotlin/Main.kt",
            "src/main/kotlin/Application.kt",
            "Main.kt",
        ] {
            if path.join(entry).exists() {
                entry_points.push(entry.to_string());
                important_files.push(entry.to_string());
            }
        }

        if path.join("README.md").exists() {
            important_files.push("README.md".to_string());
        }

        let confidence = if entry_points.is_empty() { 0.7 } else { 0.9 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Kotlin,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Kotlin
    }
}

impl KotlinProjectDetector {
    fn has_kotlin_files_recursive(&self, dir: &Path) -> bool {
        if !dir.exists() || !dir.is_dir() {
            return false;
        }

        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("kt") {
                        return true;
                    }
                } else if path.is_dir() && self.has_kotlin_files_recursive(&path) {
                    return true;
                }
            }
        }
        false
    }
}

/// Detector for Java projects
pub struct JavaProjectDetector;

impl JavaProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for JavaProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        // Check for Java project indicators
        let gradle_indicators = ["build.gradle", "settings.gradle", "gradle.properties"];
        let maven_indicators = ["pom.xml"];
        let ant_indicators = ["build.xml"];

        let has_gradle = gradle_indicators
            .iter()
            .any(|file| path.join(file).exists());
        let has_maven = maven_indicators.iter().any(|file| path.join(file).exists());
        let has_ant = ant_indicators.iter().any(|file| path.join(file).exists());

        if !has_gradle && !has_maven && !has_ant {
            // Check for Java files in standard locations
            let java_dirs = ["src/main/java", "src", "."];
            let has_java_files = java_dirs.iter().any(|dir| {
                let dir_path = if *dir == "." {
                    path.to_path_buf()
                } else {
                    path.join(dir)
                };
                self.has_java_files_recursive(&dir_path)
            });

            if !has_java_files {
                return Ok(None);
            }
        }

        let mut entry_points = Vec::new();
        let mut important_files = Vec::new();

        // Add build files to important files
        for file in &[
            "pom.xml",
            "build.gradle",
            "settings.gradle",
            "gradle.properties",
            "build.xml",
        ] {
            if path.join(file).exists() {
                important_files.push(file.to_string());
            }
        }

        // Look for common Java entry points
        for entry in &[
            "src/main/java/Main.java",
            "src/main/java/Application.java",
            "Main.java",
            "App.java",
        ] {
            if path.join(entry).exists() {
                entry_points.push(entry.to_string());
                important_files.push(entry.to_string());
            }
        }

        // Look for Spring Boot applications
        if let Ok(_entries) = path.join("src/main/java").read_dir() {
            self.find_spring_boot_apps(
                &path.join("src/main/java"),
                &mut entry_points,
                &mut important_files,
            );
        }

        if path.join("README.md").exists() {
            important_files.push("README.md".to_string());
        }

        let confidence = if entry_points.is_empty() { 0.6 } else { 0.8 };

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Java,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence,
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Java
    }
}

impl JavaProjectDetector {
    fn has_java_files_recursive(&self, dir: &Path) -> bool {
        if !dir.exists() || !dir.is_dir() {
            return false;
        }

        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
                        return true;
                    }
                } else if path.is_dir() && self.has_java_files_recursive(&path) {
                    return true;
                }
            }
        }
        false
    }

    fn find_spring_boot_apps(
        &self,
        dir: &Path,
        entry_points: &mut Vec<String>,
        important_files: &mut Vec<String>,
    ) {
        if let Ok(entries) = dir.read_dir() {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("java") {
                    // Simple heuristic: look for files containing "Application" in the name
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if file_name.contains("Application") && file_name.ends_with(".java") {
                            if let Ok(relative) =
                                path.strip_prefix(&std::env::current_dir().unwrap_or_default())
                            {
                                let relative_str = relative.to_string_lossy().to_string();
                                entry_points.push(relative_str.clone());
                                important_files.push(relative_str);
                            }
                        }
                    }
                } else if path.is_dir() {
                    self.find_spring_boot_apps(&path, entry_points, important_files);
                }
            }
        }
    }
}

/// Detector for Git repositories
pub struct GitProjectDetector;

impl GitProjectDetector {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDetector for GitProjectDetector {
    fn detect(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        let git_dir = path.join(".git");

        if !git_dir.exists() {
            return Ok(None);
        }

        let mut important_files = Vec::new();
        let mut entry_points = Vec::new();

        // Common Git-related important files
        for file in &[".gitignore", "README.md", "LICENSE", "CHANGELOG.md", ".github/workflows"] {
            if path.join(file).exists() {
                important_files.push(file.to_string());
            }
        }

        // Get recently modified files using git
        if let Ok(recent_files) = self.get_recently_modified_files(path) {
            for file in recent_files.into_iter().take(10) { // Limit to top 10 recent files
                if !important_files.contains(&file) {
                    important_files.push(file);
                }
            }
        }

        // Get staged/modified files (current working state)
        if let Ok(staged_files) = self.get_staged_files(path) {
            for file in staged_files {
                if !important_files.contains(&file) {
                    entry_points.push(file.clone()); // Staged files are likely entry points
                    important_files.push(file);
                }
            }
        }

        // Parse .gitignore for exclusion patterns (this will be used by the analyzer)
        if path.join(".gitignore").exists() {
            // The .gitignore parsing will be handled by the ignore crate integration
        }

        Ok(Some(ProjectInfo {
            project_type: ProjectType::Git,
            root_path: path.to_string_lossy().to_string(),
            entry_points,
            important_files,
            confidence: 0.8, // Higher confidence since we have git information
        }))
    }

    fn project_type(&self) -> ProjectType {
        ProjectType::Git
    }
}

impl GitProjectDetector {
    /// Get recently modified files using git log
    fn get_recently_modified_files(&self, path: &Path) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("log")
            .arg("--pretty=format:")
            .arg("--name-only")
            .arg("--since=1 week ago")
            .arg("-n")
            .arg("50") // Limit to 50 commits
            .current_dir(path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut files: Vec<String> = stdout
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|s| s.trim().to_string())
                    .collect();

                // Remove duplicates while preserving order (most recent first)
                files.dedup();

                // Filter out deleted files
                files.retain(|file| path.join(file).exists());

                Ok(files)
            }
            _ => Ok(Vec::new()), // Fail silently if git command fails
        }
    }

    /// Get currently staged/modified files
    fn get_staged_files(&self, path: &Path) -> Result<Vec<String>> {
        let output = Command::new("git")
            .arg("diff")
            .arg("--name-only")
            .arg("--cached") // Staged files
            .current_dir(path)
            .output();

        let mut files = Vec::new();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                files.extend(
                    stdout
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|s| s.trim().to_string()),
                );
            }
        }

        // Also get modified but not staged files
        let output = Command::new("git")
            .arg("diff")
            .arg("--name-only")
            .current_dir(path)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                files.extend(
                    stdout
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .map(|s| s.trim().to_string()),
                );
            }
        }

        // Remove duplicates
        files.dedup();

        // Filter out deleted files
        files.retain(|file| path.join(file).exists());

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rust_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create Cargo.toml
        fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let detector = RustProjectDetector::new();
        let result = detector.detect(path).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.project_type, ProjectType::Rust);
        assert!(info.important_files.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_javascript_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // Create package.json
        fs::write(path.join("package.json"), r#"{"name": "test"}"#).unwrap();

        let detector = JavaScriptProjectDetector::new();
        let result = detector.detect(path).unwrap();

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.project_type, ProjectType::JavaScript);
        assert!(info.important_files.contains(&"package.json".to_string()));
    }
}
