use crate::session::entity::smart_context_entity::SmartContextEntity;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model representing smart context information associated with a session
/// This provides high-level operations for managing context discovery metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSmartContext {
    pub session_id: String,
    pub project_path: String,
    pub project_type: String,
    pub max_tokens: Option<usize>,
    pub chunked_analysis: bool,
    pub chunk_strategy: Option<String>,
    pub selected_files: Vec<String>,
    pub total_tokens: Option<usize>,
    pub query_hash: Option<String>,
    pub config_hash: Option<String>,
}

impl SessionSmartContext {
    /// Create a new SessionSmartContext from discovery parameters
    #[allow(dead_code)]
    pub fn new(
        session_id: String,
        project_path: String,
        project_type: String,
        max_tokens: Option<usize>,
        chunked_analysis: bool,
        chunk_strategy: Option<String>,
        selected_files: Vec<String>,
        total_tokens: Option<usize>,
        query_hash: Option<String>,
        config_hash: Option<String>,
    ) -> Self {
        Self {
            session_id,
            project_path,
            project_type,
            max_tokens,
            chunked_analysis,
            chunk_strategy,
            selected_files,
            total_tokens,
            query_hash,
            config_hash,
        }
    }

    /// Create from entity
    #[allow(dead_code)]
    pub fn from_entity(entity: &SmartContextEntity) -> Result<Self> {
        let selected_files: Vec<String> = serde_json::from_str(&entity.selected_files)?;
        
        Ok(Self {
            session_id: entity.session_id.clone(),
            project_path: entity.project_path.clone(),
            project_type: entity.project_type.clone(),
            max_tokens: entity.max_tokens,
            chunked_analysis: entity.chunked_analysis,
            chunk_strategy: entity.chunk_strategy.clone(),
            selected_files,
            total_tokens: entity.total_tokens,
            query_hash: entity.query_hash.clone(),
            config_hash: entity.config_hash.clone(),
        })
    }

    /// Check if this context is still valid for a given query and config
    #[allow(dead_code)]
    pub fn is_valid_for_query(&self, query_hash: Option<&str>, config_hash: Option<&str>) -> bool {
        // Check if query hash matches (if we have one)
        if let (Some(stored_hash), Some(current_hash)) = (self.query_hash.as_ref(), query_hash) {
            if stored_hash != current_hash {
                return false;
            }
        }

        // Check if config hash matches (if we have one)
        if let (Some(stored_hash), Some(current_hash)) = (self.config_hash.as_ref(), config_hash) {
            if stored_hash != current_hash {
                return false;
            }
        }

        true
    }

    /// Get a summary string for display purposes
    #[allow(dead_code)]
    pub fn get_summary(&self) -> String {
        let chunk_info = if self.chunked_analysis {
            let strategy = self.chunk_strategy.as_deref().unwrap_or("default");
            format!(" (chunked: {})", strategy)
        } else {
            String::new()
        };

        let token_info = match (self.max_tokens, self.total_tokens) {
            (Some(max), Some(total)) => format!(" [{}/{}]", total, max),
            (Some(max), None) => format!(" [max: {}]", max),
            (None, Some(total)) => format!(" [{}]", total),
            (None, None) => String::new(),
        };

        format!(
            "{} project: {} files{}{}", 
            self.project_type,
            self.selected_files.len(),
            token_info,
            chunk_info
        )
    }

    /// Get file list as a formatted string for display
    #[allow(dead_code)]
    pub fn get_files_display(&self, max_files: usize) -> String {
        if self.selected_files.is_empty() {
            return "No files selected".to_string();
        }

        let mut display = String::new();
        let files_to_show = std::cmp::min(self.selected_files.len(), max_files);
        
        for (i, file) in self.selected_files.iter().enumerate().take(files_to_show) {
            if i > 0 {
                display.push_str(", ");
            }
            // Show just the filename, not the full path
            let filename = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file);
            display.push_str(filename);
        }

        if self.selected_files.len() > max_files {
            display.push_str(&format!(" (+{} more)", self.selected_files.len() - max_files));
        }

        display
    }

    /// Create context metadata that can be stored with messages
    /// This allows tracking what context was used for specific AI responses
    #[allow(dead_code)]
    pub fn to_context_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        metadata.insert("smart_context_enabled".to_string(), "true".to_string());
        metadata.insert("project_type".to_string(), self.project_type.clone());
        metadata.insert("project_path".to_string(), self.project_path.clone());
        metadata.insert("selected_files_count".to_string(), self.selected_files.len().to_string());
        
        if let Some(tokens) = self.total_tokens {
            metadata.insert("total_tokens".to_string(), tokens.to_string());
        }
        
        if self.chunked_analysis {
            metadata.insert("chunked_analysis".to_string(), "true".to_string());
            if let Some(strategy) = &self.chunk_strategy {
                metadata.insert("chunk_strategy".to_string(), strategy.clone());
            }
        }

        // Store a subset of files for reference (avoid making this too large)
        let key_files: Vec<&String> = self.selected_files.iter().take(10).collect();
        if !key_files.is_empty() {
            let files_json = serde_json::to_string(&key_files).unwrap_or_default();
            metadata.insert("key_files".to_string(), files_json);
        }

        metadata
    }

    /// Update the context with new results (used when context is re-run)
    #[allow(dead_code)]
    pub fn update_results(
        &mut self,
        selected_files: Vec<String>,
        total_tokens: Option<usize>,
        query_hash: Option<String>,
    ) {
        self.selected_files = selected_files;
        self.total_tokens = total_tokens;
        self.query_hash = query_hash;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_context_creation() {
        let context = SessionSmartContext::new(
            "session123".to_string(),
            "/path/to/rust/project".to_string(),
            "rust".to_string(),
            Some(4000),
            false,
            None,
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            Some(1500),
            Some("query_hash_abc".to_string()),
            Some("config_hash_def".to_string()),
        );

        assert_eq!(context.session_id, "session123");
        assert_eq!(context.project_type, "rust");
        assert_eq!(context.selected_files.len(), 2);
        assert_eq!(context.max_tokens, Some(4000));
        assert_eq!(context.total_tokens, Some(1500));
        assert!(!context.chunked_analysis);
    }

    #[test]
    fn test_context_validity_check() {
        let context = SessionSmartContext::new(
            "session123".to_string(),
            "/path/to/project".to_string(),
            "javascript".to_string(),
            Some(3000),
            false,
            None,
            vec!["index.js".to_string()],
            Some(800),
            Some("query_abc".to_string()),
            Some("config_def".to_string()),
        );

        // Should be valid with matching hashes
        assert!(context.is_valid_for_query(Some("query_abc"), Some("config_def")));

        // Should be invalid with mismatched query hash
        assert!(!context.is_valid_for_query(Some("different_query"), Some("config_def")));

        // Should be invalid with mismatched config hash
        assert!(!context.is_valid_for_query(Some("query_abc"), Some("different_config")));

        // Should be valid with None hashes (no hash to compare)
        assert!(context.is_valid_for_query(None, None));
    }

    #[test]
    fn test_summary_generation() {
        let context_simple = SessionSmartContext::new(
            "session1".to_string(),
            "/project".to_string(),
            "rust".to_string(),
            Some(4000),
            false,
            None,
            vec!["main.rs".to_string(), "lib.rs".to_string()],
            Some(1500),
            None,
            None,
        );

        let summary = context_simple.get_summary();
        assert_eq!(summary, "rust project: 2 files [1500/4000]");

        let context_chunked = SessionSmartContext::new(
            "session2".to_string(),
            "/project".to_string(),
            "javascript".to_string(),
            Some(3000),
            true,
            Some("hierarchical".to_string()),
            vec!["index.js".to_string()],
            Some(2800),
            None,
            None,
        );

        let summary_chunked = context_chunked.get_summary();
        assert_eq!(summary_chunked, "javascript project: 1 files [2800/3000] (chunked: hierarchical)");
    }

    #[test]
    fn test_files_display() {
        let files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/utils/helper.rs".to_string(),
            "tests/integration_test.rs".to_string(),
            "Cargo.toml".to_string(),
        ];

        let context = SessionSmartContext::new(
            "session1".to_string(),
            "/project".to_string(),
            "rust".to_string(),
            None,
            false,
            None,
            files,
            None,
            None,
            None,
        );

        // Show first 3 files
        let display = context.get_files_display(3);
        assert_eq!(display, "main.rs, lib.rs, helper.rs (+2 more)");

        // Show all files
        let display_all = context.get_files_display(10);
        assert_eq!(display_all, "main.rs, lib.rs, helper.rs, integration_test.rs, Cargo.toml");
    }

    #[test]
    fn test_context_metadata() {
        let context = SessionSmartContext::new(
            "session123".to_string(),
            "/rust/project".to_string(),
            "rust".to_string(),
            Some(4000),
            true,
            Some("functional".to_string()),
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            Some(2000),
            Some("query_hash".to_string()),
            Some("config_hash".to_string()),
        );

        let metadata = context.to_context_metadata();

        assert_eq!(metadata.get("smart_context_enabled"), Some(&"true".to_string()));
        assert_eq!(metadata.get("project_type"), Some(&"rust".to_string()));
        assert_eq!(metadata.get("selected_files_count"), Some(&"2".to_string()));
        assert_eq!(metadata.get("total_tokens"), Some(&"2000".to_string()));
        assert_eq!(metadata.get("chunked_analysis"), Some(&"true".to_string()));
        assert_eq!(metadata.get("chunk_strategy"), Some(&"functional".to_string()));
        
        // Should have key files stored as JSON
        assert!(metadata.contains_key("key_files"));
    }

    #[test]
    fn test_update_results() {
        let mut context = SessionSmartContext::new(
            "session1".to_string(),
            "/project".to_string(),
            "python".to_string(),
            Some(3000),
            false,
            None,
            vec!["main.py".to_string()],
            Some(500),
            Some("old_query".to_string()),
            None,
        );

        // Update with new results
        context.update_results(
            vec!["main.py".to_string(), "utils.py".to_string()],
            Some(800),
            Some("new_query".to_string()),
        );

        assert_eq!(context.selected_files.len(), 2);
        assert_eq!(context.total_tokens, Some(800));
        assert_eq!(context.query_hash, Some("new_query".to_string()));
    }
}