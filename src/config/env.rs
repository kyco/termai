/// Environment variable handling for TermAI configuration
use std::env;

/// Environment variable names used by TermAI
pub struct EnvVars;

impl EnvVars {
    /// OpenAI API key environment variable
    pub const OPENAI_API_KEY: &'static str = "OPENAI_API_KEY";

    /// Claude API key environment variable
    pub const CLAUDE_API_KEY: &'static str = "CLAUDE_API_KEY";

    /// Anthropic API key (alternative to CLAUDE_API_KEY)
    pub const ANTHROPIC_API_KEY: &'static str = "ANTHROPIC_API_KEY";

    /// Default provider environment variable
    pub const TERMAI_PROVIDER: &'static str = "TERMAI_PROVIDER";

    /// System prompt override
    pub const TERMAI_SYSTEM_PROMPT: &'static str = "TERMAI_SYSTEM_PROMPT";

    /// Default session name
    pub const TERMAI_SESSION: &'static str = "TERMAI_SESSION";

    /// Enable debug mode
    pub const TERMAI_DEBUG: &'static str = "TERMAI_DEBUG";

    /// Configuration directory override
    pub const TERMAI_CONFIG_DIR: &'static str = "TERMAI_CONFIG_DIR";

    /// Smart context enabled by default
    pub const TERMAI_SMART_CONTEXT: &'static str = "TERMAI_SMART_CONTEXT";

    /// Maximum context tokens
    pub const TERMAI_MAX_CONTEXT_TOKENS: &'static str = "TERMAI_MAX_CONTEXT_TOKENS";

    /// Default directories to include as context
    pub const TERMAI_CONTEXT_DIRS: &'static str = "TERMAI_CONTEXT_DIRS";

    /// Default patterns to exclude from context
    pub const TERMAI_EXCLUDE_PATTERNS: &'static str = "TERMAI_EXCLUDE_PATTERNS";

    /// All supported environment variable names
    pub const ALL: &'static [&'static str] = &[
        Self::OPENAI_API_KEY,
        Self::CLAUDE_API_KEY,
        Self::ANTHROPIC_API_KEY,
        Self::TERMAI_PROVIDER,
        Self::TERMAI_SYSTEM_PROMPT,
        Self::TERMAI_SESSION,
        Self::TERMAI_DEBUG,
        Self::TERMAI_CONFIG_DIR,
        Self::TERMAI_SMART_CONTEXT,
        Self::TERMAI_MAX_CONTEXT_TOKENS,
        Self::TERMAI_CONTEXT_DIRS,
        Self::TERMAI_EXCLUDE_PATTERNS,
    ];
}

/// Environment variable resolution with fallback support
pub struct EnvResolver;

#[allow(dead_code)]
impl EnvResolver {
    /// Get OpenAI API key from environment
    pub fn openai_api_key() -> Option<String> {
        env::var(EnvVars::OPENAI_API_KEY).ok()
    }

    /// Get Claude API key from environment (checks both CLAUDE_API_KEY and ANTHROPIC_API_KEY)
    pub fn claude_api_key() -> Option<String> {
        env::var(EnvVars::CLAUDE_API_KEY)
            .or_else(|_| env::var(EnvVars::ANTHROPIC_API_KEY))
            .ok()
    }

    /// Get provider from environment
    pub fn provider() -> Option<String> {
        env::var(EnvVars::TERMAI_PROVIDER).ok()
    }

    /// Get system prompt from environment
    pub fn system_prompt() -> Option<String> {
        env::var(EnvVars::TERMAI_SYSTEM_PROMPT).ok()
    }

    /// Get default session name from environment
    pub fn session() -> Option<String> {
        env::var(EnvVars::TERMAI_SESSION).ok()
    }

    /// Check if debug mode is enabled
    pub fn debug_enabled() -> bool {
        env::var(EnvVars::TERMAI_DEBUG)
            .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
            .unwrap_or(false)
    }

    /// Get custom config directory
    pub fn config_dir() -> Option<String> {
        env::var(EnvVars::TERMAI_CONFIG_DIR).ok()
    }

    /// Check if smart context is enabled by default
    pub fn smart_context_enabled() -> bool {
        env::var(EnvVars::TERMAI_SMART_CONTEXT)
            .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
            .unwrap_or(false)
    }

    /// Get maximum context tokens from environment
    pub fn max_context_tokens() -> Option<usize> {
        env::var(EnvVars::TERMAI_MAX_CONTEXT_TOKENS)
            .ok()
            .and_then(|v| v.parse().ok())
    }

    /// Get default context directories (comma-separated)
    pub fn context_directories() -> Vec<String> {
        env::var(EnvVars::TERMAI_CONTEXT_DIRS)
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get default exclude patterns (comma-separated)
    pub fn exclude_patterns() -> Vec<String> {
        env::var(EnvVars::TERMAI_EXCLUDE_PATTERNS)
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all set environment variables for TermAI
    pub fn get_all_set() -> Vec<(String, String)> {
        EnvVars::ALL
            .iter()
            .filter_map(|&var| env::var(var).ok().map(|value| (var.to_string(), value)))
            .collect()
    }

    /// Display help about supported environment variables
    pub fn help_text() -> String {
        format!(
            r#"Environment Variables:
  {}
    OpenAI API key for GPT models
    
  {}
    Claude API key (Anthropic)
    
  {}
    Alternative Claude API key variable
    
  {}
    Default AI provider (claude, openai)
    
  {}
    Default system prompt override
    
  {}
    Default session name to use
    
  {}
    Enable debug mode (true/false)
    
  {}
    Custom configuration directory path
    
  {}
    Enable smart context by default (true/false)
    
  {}
    Maximum context tokens (number)
    
  {}
    Default directories for context (comma-separated)
    
  {}
    Default exclude patterns (comma-separated)

Examples:
  export OPENAI_API_KEY="sk-..."
  export CLAUDE_API_KEY="sk-ant-..."
  export TERMAI_PROVIDER="claude"
  export TERMAI_SMART_CONTEXT="true"
  export TERMAI_CONTEXT_DIRS="src/,tests/"
  export TERMAI_EXCLUDE_PATTERNS="*.log,target/"
"#,
            EnvVars::OPENAI_API_KEY,
            EnvVars::CLAUDE_API_KEY,
            EnvVars::ANTHROPIC_API_KEY,
            EnvVars::TERMAI_PROVIDER,
            EnvVars::TERMAI_SYSTEM_PROMPT,
            EnvVars::TERMAI_SESSION,
            EnvVars::TERMAI_DEBUG,
            EnvVars::TERMAI_CONFIG_DIR,
            EnvVars::TERMAI_SMART_CONTEXT,
            EnvVars::TERMAI_MAX_CONTEXT_TOKENS,
            EnvVars::TERMAI_CONTEXT_DIRS,
            EnvVars::TERMAI_EXCLUDE_PATTERNS,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_env_var_names() {
        assert_eq!(EnvVars::OPENAI_API_KEY, "OPENAI_API_KEY");
        assert_eq!(EnvVars::CLAUDE_API_KEY, "CLAUDE_API_KEY");
        assert_eq!(EnvVars::ANTHROPIC_API_KEY, "ANTHROPIC_API_KEY");
        assert_eq!(EnvVars::TERMAI_PROVIDER, "TERMAI_PROVIDER");
    }

    #[test]
    fn test_debug_enabled() {
        // Test with unset variable
        env::remove_var(EnvVars::TERMAI_DEBUG);
        assert!(!EnvResolver::debug_enabled());

        // Test with "true"
        env::set_var(EnvVars::TERMAI_DEBUG, "true");
        assert!(EnvResolver::debug_enabled());

        // Test with "1"
        env::set_var(EnvVars::TERMAI_DEBUG, "1");
        assert!(EnvResolver::debug_enabled());

        // Test with "false"
        env::set_var(EnvVars::TERMAI_DEBUG, "false");
        assert!(!EnvResolver::debug_enabled());

        // Test with "0"
        env::set_var(EnvVars::TERMAI_DEBUG, "0");
        assert!(!EnvResolver::debug_enabled());

        // Cleanup
        env::remove_var(EnvVars::TERMAI_DEBUG);
    }

    #[test]
    fn test_context_directories_parsing() {
        // Test with unset variable
        env::remove_var(EnvVars::TERMAI_CONTEXT_DIRS);
        assert_eq!(EnvResolver::context_directories(), Vec::<String>::new());

        // Test with single directory
        env::set_var(EnvVars::TERMAI_CONTEXT_DIRS, "src/");
        assert_eq!(EnvResolver::context_directories(), vec!["src/"]);

        // Test with multiple directories
        env::set_var(EnvVars::TERMAI_CONTEXT_DIRS, "src/, tests/ ,docs/");
        assert_eq!(
            EnvResolver::context_directories(),
            vec!["src/", "tests/", "docs/"]
        );

        // Test with empty values
        env::set_var(EnvVars::TERMAI_CONTEXT_DIRS, "src/,,tests/,");
        assert_eq!(EnvResolver::context_directories(), vec!["src/", "tests/"]);

        // Cleanup
        env::remove_var(EnvVars::TERMAI_CONTEXT_DIRS);
    }

    #[test]
    fn test_claude_api_key_fallback() {
        // Clean up any existing values
        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::remove_var(EnvVars::ANTHROPIC_API_KEY);

        // Test with no keys set
        assert!(EnvResolver::claude_api_key().is_none());

        // Test with CLAUDE_API_KEY set
        env::set_var(EnvVars::CLAUDE_API_KEY, "claude-key");
        assert_eq!(
            EnvResolver::claude_api_key(),
            Some("claude-key".to_string())
        );

        // Test fallback to ANTHROPIC_API_KEY
        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::set_var(EnvVars::ANTHROPIC_API_KEY, "anthropic-key");
        assert_eq!(
            EnvResolver::claude_api_key(),
            Some("anthropic-key".to_string())
        );

        // Test CLAUDE_API_KEY takes precedence
        env::set_var(EnvVars::CLAUDE_API_KEY, "claude-key");
        env::set_var(EnvVars::ANTHROPIC_API_KEY, "anthropic-key");
        assert_eq!(
            EnvResolver::claude_api_key(),
            Some("claude-key".to_string())
        );

        // Cleanup
        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::remove_var(EnvVars::ANTHROPIC_API_KEY);
    }

    #[test]
    fn test_max_context_tokens_parsing() {
        // Test with unset variable
        env::remove_var(EnvVars::TERMAI_MAX_CONTEXT_TOKENS);
        assert!(EnvResolver::max_context_tokens().is_none());

        // Test with valid number
        env::set_var(EnvVars::TERMAI_MAX_CONTEXT_TOKENS, "8000");
        assert_eq!(EnvResolver::max_context_tokens(), Some(8000));

        // Test with invalid number
        env::set_var(EnvVars::TERMAI_MAX_CONTEXT_TOKENS, "invalid");
        assert!(EnvResolver::max_context_tokens().is_none());

        // Cleanup
        env::remove_var(EnvVars::TERMAI_MAX_CONTEXT_TOKENS);
    }

    #[test]
    fn test_get_all_set() {
        // Store current environment state
        let initial_vars: Vec<_> = EnvVars::ALL
            .iter()
            .filter_map(|&var| env::var(var).ok().map(|val| (var, val)))
            .collect();

        // Clean up environment
        for &var in EnvVars::ALL {
            env::remove_var(var);
        }

        // Should be empty
        assert_eq!(EnvResolver::get_all_set().len(), 0);

        // Set a few variables
        env::set_var(EnvVars::OPENAI_API_KEY, "test-key");
        env::set_var(EnvVars::TERMAI_PROVIDER, "openai");

        let all_set = EnvResolver::get_all_set();
        assert_eq!(all_set.len(), 2);
        assert!(all_set.contains(&("OPENAI_API_KEY".to_string(), "test-key".to_string())));
        assert!(all_set.contains(&("TERMAI_PROVIDER".to_string(), "openai".to_string())));

        // Restore original environment
        for &var in EnvVars::ALL {
            env::remove_var(var);
        }
        for (var, val) in initial_vars {
            env::set_var(var, val);
        }
    }

    #[test]
    fn test_help_text_contains_all_vars() {
        let help = EnvResolver::help_text();
        for &var in EnvVars::ALL {
            assert!(help.contains(var), "Help text should contain {}", var);
        }
    }
}
