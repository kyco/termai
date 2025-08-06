use crate::context::ProjectType;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for smart context discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    pub context: ContextSettings,
    pub project: Option<ProjectSettings>,
}

/// Context-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSettings {
    pub max_tokens: usize,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub priority_patterns: Vec<String>,
    pub enable_cache: Option<bool>,
}

/// Project-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub project_type: Option<String>,
    pub entry_points: Vec<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            context: ContextSettings {
                max_tokens: 4000,
                include: vec![
                    "src/**/*.rs".to_string(),
                    "tests/**/*.rs".to_string(),
                    "Cargo.toml".to_string(),
                    "README.md".to_string(),
                ],
                exclude: vec![
                    "target/**".to_string(),
                    "**/*.log".to_string(),
                    "**/node_modules/**".to_string(),
                    ".git/**".to_string(),
                ],
                priority_patterns: vec![
                    "main.rs".to_string(),
                    "lib.rs".to_string(),
                    "mod.rs".to_string(),
                    "index.js".to_string(),
                    "index.ts".to_string(),
                    "main.py".to_string(),
                    "__init__.py".to_string(),
                ],
                enable_cache: Some(true),
            },
            project: None,
        }
    }
}

impl ContextConfig {
    /// Load configuration from a .termai.toml file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let config: ContextConfig = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse .termai.toml: {}", e))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to a .termai.toml file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize configuration: {}", e))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Find and load configuration from project directory
    pub fn discover_config(project_path: &Path) -> Result<Self> {
        let config_file = project_path.join(".termai.toml");
        Self::load_from_file(&config_file)
    }

    /// Get project type from configuration
    pub fn get_project_type(&self) -> Option<ProjectType> {
        self.project.as_ref().and_then(|p| {
            p.project_type.as_ref().and_then(|t| match t.as_str() {
                "rust" => Some(ProjectType::Rust),
                "javascript" | "js" | "typescript" | "ts" => Some(ProjectType::JavaScript),
                "python" | "py" => Some(ProjectType::Python),
                "go" | "golang" => Some(ProjectType::Go),
                "kotlin" | "kt" => Some(ProjectType::Kotlin),
                "java" => Some(ProjectType::Java),
                "git" => Some(ProjectType::Git),
                _ => Some(ProjectType::Generic),
            })
        })
    }

    /// Check if a file should be included based on patterns
    pub fn should_include(&self, file_path: &str) -> bool {
        // Check exclude patterns first
        for pattern in &self.context.exclude {
            if self.matches_pattern(file_path, pattern) {
                return false;
            }
        }

        // Check include patterns
        if self.context.include.is_empty() {
            return true; // Include everything if no specific patterns
        }

        for pattern in &self.context.include {
            if self.matches_pattern(file_path, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a file matches a priority pattern
    pub fn is_priority_file(&self, file_path: &str) -> bool {
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        self.context
            .priority_patterns
            .iter()
            .any(|pattern| file_name.contains(pattern))
    }

    /// Simple pattern matching (will be enhanced with proper glob support)
    pub fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        if pattern.contains("**") {
            // Recursive directory matching: src/**/*.rs matches src/main.rs, src/foo/bar.rs, etc.
            let pattern_parts: Vec<&str> = pattern.split("**").collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1].trim_start_matches('/');

                if file_path.starts_with(prefix) {
                    if suffix.is_empty() {
                        return true; // Just prefix/**
                    }
                    // Handle suffix pattern like "*.rs"
                    if suffix.starts_with('*') {
                        let extension = suffix.trim_start_matches('*');
                        return file_path.ends_with(extension);
                    } else {
                        return file_path.ends_with(suffix);
                    }
                }
            }
        } else if pattern.contains('*') {
            // Simple wildcard matching
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                return file_path.starts_with(pattern_parts[0])
                    && file_path.ends_with(pattern_parts[1]);
            }
        } else {
            // Exact match
            return file_path == pattern || file_path.ends_with(pattern);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = ContextConfig::default();

        assert_eq!(config.context.max_tokens, 4000);
        assert!(!config.context.include.is_empty());
        assert!(!config.context.exclude.is_empty());
        assert!(!config.context.priority_patterns.is_empty());
    }

    #[test]
    fn test_should_include() {
        let config = ContextConfig::default();

        // Should include Rust source files
        assert!(config.should_include("src/main.rs"));
        assert!(config.should_include("tests/test_example.rs"));

        // Should exclude target directory
        assert!(!config.should_include("target/debug/deps/example"));
        assert!(!config.should_include("example.log"));
    }

    #[test]
    fn test_priority_files() {
        let config = ContextConfig::default();

        assert!(config.is_priority_file("src/main.rs"));
        assert!(config.is_priority_file("lib.rs"));
        assert!(!config.is_priority_file("src/utils.rs"));
    }

    #[test]
    fn test_discover_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = ContextConfig::discover_config(temp_dir.path()).unwrap();

        // Should return default config when no .termai.toml exists
        assert_eq!(config.context.max_tokens, 4000);
    }
}
