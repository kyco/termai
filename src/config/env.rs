/// Environment variable handling for auth-only configuration.
use std::env;

/// Environment variable names supported by TermAI.
pub struct EnvVars;

impl EnvVars {
    /// OpenAI API key environment variable.
    pub const OPENAI_API_KEY: &'static str = "OPENAI_API_KEY";

    /// Claude API key environment variable.
    pub const CLAUDE_API_KEY: &'static str = "CLAUDE_API_KEY";

    /// Anthropic API key (alternative to CLAUDE_API_KEY).
    pub const ANTHROPIC_API_KEY: &'static str = "ANTHROPIC_API_KEY";

    /// All supported environment variable names.
    pub const ALL: &'static [&'static str] = &[
        Self::OPENAI_API_KEY,
        Self::CLAUDE_API_KEY,
        Self::ANTHROPIC_API_KEY,
    ];
}

/// Environment variable resolution with auth-only fallback support.
pub struct EnvResolver;

#[allow(dead_code)]
impl EnvResolver {
    /// Get OpenAI API key from environment.
    pub fn openai_api_key() -> Option<String> {
        env::var(EnvVars::OPENAI_API_KEY).ok()
    }

    /// Get Claude API key from environment.
    pub fn claude_api_key() -> Option<String> {
        env::var(EnvVars::CLAUDE_API_KEY)
            .or_else(|_| env::var(EnvVars::ANTHROPIC_API_KEY))
            .ok()
    }

    /// Get all supported environment variables that are currently set.
    pub fn get_all_set() -> Vec<(String, String)> {
        EnvVars::ALL
            .iter()
            .filter_map(|&var| env::var(var).ok().map(|value| (var.to_string(), value)))
            .collect()
    }

    /// Display help about supported environment variables.
    pub fn help_text() -> String {
        format!(
            r#"Environment Variables:
  {}
    OpenAI API key for OpenAI and Codex model listings

  {}
    Claude API key (Anthropic)

  {}
    Alternative Claude API key variable

Examples:
  export OPENAI_API_KEY="sk-..."
  export CLAUDE_API_KEY="sk-ant-..."
"#,
            EnvVars::OPENAI_API_KEY,
            EnvVars::CLAUDE_API_KEY,
            EnvVars::ANTHROPIC_API_KEY,
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
    }

    #[test]
    fn test_claude_api_key_prefers_primary_name() {
        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::remove_var(EnvVars::ANTHROPIC_API_KEY);

        assert!(EnvResolver::claude_api_key().is_none());

        env::set_var(EnvVars::ANTHROPIC_API_KEY, "anthropic-key");
        assert_eq!(
            EnvResolver::claude_api_key(),
            Some("anthropic-key".to_string())
        );

        env::set_var(EnvVars::CLAUDE_API_KEY, "claude-key");
        assert_eq!(
            EnvResolver::claude_api_key(),
            Some("claude-key".to_string())
        );

        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::remove_var(EnvVars::ANTHROPIC_API_KEY);
    }

    #[test]
    fn test_get_all_set_returns_auth_env_vars_only() {
        env::remove_var(EnvVars::OPENAI_API_KEY);
        env::remove_var(EnvVars::CLAUDE_API_KEY);
        env::remove_var(EnvVars::ANTHROPIC_API_KEY);

        let baseline = EnvResolver::get_all_set();

        env::set_var(EnvVars::OPENAI_API_KEY, "openai-key");
        let all_set = EnvResolver::get_all_set();
        assert_eq!(all_set.len(), baseline.len() + 1);
        assert!(all_set.contains(&("OPENAI_API_KEY".to_string(), "openai-key".to_string())));

        env::remove_var(EnvVars::OPENAI_API_KEY);
    }

    #[test]
    fn test_help_text_only_mentions_supported_env_vars() {
        let help = EnvResolver::help_text();
        assert!(help.contains(EnvVars::OPENAI_API_KEY));
        assert!(help.contains(EnvVars::CLAUDE_API_KEY));
        assert!(help.contains(EnvVars::ANTHROPIC_API_KEY));
        assert!(!help.contains("TERMAI_PROVIDER"));
    }
}
