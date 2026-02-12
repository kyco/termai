/// Value providers for dynamic completion
use crate::repository::db::SqliteRepository;
use crate::session::repository::SessionRepository;
use anyhow::Result;

/// Provides completion values for various argument types
pub struct CompletionValues;

#[allow(dead_code)]
impl CompletionValues {
    /// Get available session names for completion
    pub fn session_names(repo: &SqliteRepository) -> Result<Vec<String>> {
        let sessions = repo.fetch_all_sessions()?;
        Ok(sessions.into_iter().map(|s| s.name).collect())
    }

    /// Get available provider names
    pub fn provider_names() -> Vec<String> {
        vec!["claude".to_string(), "openai".to_string()]
    }

    /// Get available model names (static list of common models)
    pub fn model_names() -> Vec<String> {
        vec![
            // Claude 4 series
            "claude-opus-4-1-20250805".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            // Claude 3.7 series  
            "claude-3-7-sonnet-20250219".to_string(),
            "claude-3-7-sonnet-latest".to_string(),
            // Claude 3.5 series
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-5-haiku-latest".to_string(),
            // Claude 3 legacy series
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            // OpenAI GPT-5.2 series
            "gpt-5.2".to_string(),
            "gpt-5.2-chat-latest".to_string(),
            "gpt-5.2-pro".to_string(),
            // OpenAI Codex models (ChatGPT Plus/Pro via OAuth)
            "gpt-5.3-codex".to_string(),
            "gpt-5.2-codex".to_string(),
            "gpt-5.1-codex-mini".to_string(),
            "gpt-5.1-codex-max".to_string(),
            // OpenAI GPT-5 series
            "gpt-5.1".to_string(),
            "gpt-5-mini".to_string(),
            "gpt-5-nano".to_string(),
            // OpenAI o3 series (deep research)
            "o3-deep-research".to_string(),
            "o4-mini-deep-research".to_string(),
            "o3-pro".to_string(),
            "o3".to_string(),
            "o4-mini".to_string(),
            "o3-mini".to_string(),
            // OpenAI GPT-4.1 series
            "gpt-4.1".to_string(),
            "gpt-4.1-mini".to_string(),
            "gpt-4.1-nano".to_string(),
            // OpenAI GPT-4.5 series
            "gpt-4.5-preview".to_string(),
            // OpenAI GPT-4o series (text-capable only)
            "gpt-4o-mini-search-preview".to_string(),
            "gpt-4o-search-preview".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            // OpenAI o1 series (reasoning)
            "o1-pro".to_string(),
            "o1".to_string(),
            "o1-mini".to_string(),
            "o1-preview".to_string(),
            // OpenAI specialized models (text-capable)
            "computer-use-preview".to_string(),
            // OpenAI legacy models
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }

    /// Get available chunk strategies
    pub fn chunk_strategies() -> Vec<String> {
        vec![
            "module".to_string(),
            "functional".to_string(),
            "token".to_string(),
            "hierarchical".to_string(),
        ]
    }

    /// Get common exclude patterns
    pub fn common_exclude_patterns() -> Vec<String> {
        vec![
            "*.log".to_string(),
            "*.tmp".to_string(),
            "target/".to_string(),
            "node_modules/".to_string(),
            ".git/".to_string(),
            ".DS_Store".to_string(),
            "*.pyc".to_string(),
            "__pycache__/".to_string(),
            "build/".to_string(),
            "dist/".to_string(),
        ]
    }

    /// Get common context directories
    pub fn common_context_directories() -> Vec<String> {
        vec![
            "src/".to_string(),
            "lib/".to_string(),
            "tests/".to_string(),
            "docs/".to_string(),
            "examples/".to_string(),
            "scripts/".to_string(),
            ".".to_string(),
        ]
    }

    /// Get shell types for completion generation
    pub fn shell_types() -> Vec<String> {
        vec![
            "bash".to_string(),
            "zsh".to_string(),
            "fish".to_string(),
            "powershell".to_string(),
        ]
    }

    /// Get configuration keys that can be set
    pub fn config_keys() -> Vec<String> {
        vec![
            "provider".to_string(),
            "openai-api-key".to_string(),
            "claude-api-key".to_string(),
            "system-prompt".to_string(),
            "max-context-tokens".to_string(),
        ]
    }

    /// Get session sort orders
    pub fn session_sort_orders() -> Vec<String> {
        vec![
            "name".to_string(),
            "date".to_string(),
            "messages".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_names() {
        let providers = CompletionValues::provider_names();
        assert!(providers.contains(&"claude".to_string()));
        assert!(providers.contains(&"openai".to_string()));
    }

    #[test]
    fn test_model_names() {
        let models = CompletionValues::model_names();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("claude")));
        assert!(models.iter().any(|m| m.contains("gpt")));
    }

    #[test]
    fn test_chunk_strategies() {
        let strategies = CompletionValues::chunk_strategies();
        assert_eq!(
            strategies,
            vec!["module", "functional", "token", "hierarchical"]
        );
    }

    #[test]
    fn test_common_exclude_patterns() {
        let patterns = CompletionValues::common_exclude_patterns();
        assert!(patterns.contains(&"*.log".to_string()));
        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"node_modules/".to_string()));
    }

    #[test]
    fn test_shell_types() {
        let shells = CompletionValues::shell_types();
        assert_eq!(shells.len(), 4);
        assert!(shells.contains(&"bash".to_string()));
        assert!(shells.contains(&"zsh".to_string()));
    }
}
