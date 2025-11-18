use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use crate::llm::openai::model::verbosity::Verbosity;
use crate::llm::openai::model::custom_tools::PreambleConfig;
use serde::{Serialize, Deserialize};

/// Configuration for GPT-5 specific features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gpt5Config {
    /// Preferred reasoning effort level
    pub reasoning_effort: ReasoningEffort,
    
    /// Preferred verbosity level
    pub verbosity: Verbosity,
    
    /// Whether to use the new Responses API when available
    pub prefer_responses_api: bool,
    
    /// Enable preambles for tool calls
    pub preambles: PreambleConfig,
    
    /// Store conversations for multi-turn context
    pub store_conversations: bool,
    
    /// Zero Data Retention mode (uses encrypted reasoning items)
    pub zero_data_retention: bool,
}

impl Default for Gpt5Config {
    fn default() -> Self {
        Self {
            reasoning_effort: ReasoningEffort::Medium,
            verbosity: Verbosity::Medium,
            prefer_responses_api: true,
            preambles: PreambleConfig::default(),
            store_conversations: false,
            zero_data_retention: false,
        }
    }
}

#[allow(dead_code)]
impl Gpt5Config {
    /// Create config optimized for coding tasks
    pub fn for_coding() -> Self {
        Self {
            reasoning_effort: ReasoningEffort::High,
            verbosity: Verbosity::Medium,
            prefer_responses_api: true,
            preambles: PreambleConfig::default(),
            store_conversations: false,
            zero_data_retention: false,
        }
    }

    /// Create config optimized for complex reasoning tasks
    pub fn for_reasoning() -> Self {
        Self {
            reasoning_effort: ReasoningEffort::High,
            verbosity: Verbosity::High,
            prefer_responses_api: true,
            preambles: PreambleConfig::enabled(),
            store_conversations: true,
            zero_data_retention: false,
        }
    }

    /// Create config optimized for quick responses
    pub fn for_speed() -> Self {
        Self {
            reasoning_effort: ReasoningEffort::None,
            verbosity: Verbosity::Low,
            prefer_responses_api: true,
            preambles: PreambleConfig::default(),
            store_conversations: false,
            zero_data_retention: false,
        }
    }

    /// Create config for privacy-conscious usage (ZDR mode)
    pub fn for_privacy() -> Self {
        Self {
            reasoning_effort: ReasoningEffort::Medium,
            verbosity: Verbosity::Medium,
            prefer_responses_api: true,
            preambles: PreambleConfig::default(),
            store_conversations: false,
            zero_data_retention: true,
        }
    }

    /// Update reasoning effort
    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = effort;
        self
    }

    /// Update verbosity
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Enable/disable preambles
    pub fn with_preambles(mut self, enabled: bool) -> Self {
        self.preambles.enabled = enabled;
        if enabled && self.preambles.instruction.is_none() {
            self.preambles.instruction = Some("Before you call a tool, explain why you are calling it.".to_string());
        }
        self
    }

    /// Enable/disable conversation storage
    pub fn with_storage(mut self, store: bool) -> Self {
        self.store_conversations = store;
        self
    }

    /// Enable/disable zero data retention mode
    pub fn with_zdr(mut self, zdr: bool) -> Self {
        self.zero_data_retention = zdr;
        if zdr {
            // ZDR mode enforces no storage
            self.store_conversations = false;
        }
        self
    }

    /// Get description of current configuration
    pub fn describe(&self) -> String {
        format!(
            "GPT-5 Config: reasoning={}, verbosity={}, api={}, preambles={}, storage={}, zdr={}",
            self.reasoning_effort.to_string(),
            self.verbosity.to_string(),
            if self.prefer_responses_api { "responses" } else { "chat" },
            self.preambles.enabled,
            self.store_conversations,
            self.zero_data_retention,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Gpt5Config::default();
        assert_eq!(config.reasoning_effort, ReasoningEffort::Medium);
        assert_eq!(config.verbosity, Verbosity::Medium);
        assert!(config.prefer_responses_api);
        assert!(!config.preambles.enabled);
        assert!(!config.store_conversations);
        assert!(!config.zero_data_retention);
    }

    #[test]
    fn test_coding_config() {
        let config = Gpt5Config::for_coding();
        assert_eq!(config.reasoning_effort, ReasoningEffort::High);
        assert_eq!(config.verbosity, Verbosity::Medium);
    }

    #[test]
    fn test_reasoning_config() {
        let config = Gpt5Config::for_reasoning();
        assert_eq!(config.reasoning_effort, ReasoningEffort::High);
        assert_eq!(config.verbosity, Verbosity::High);
        assert!(config.preambles.enabled);
        assert!(config.store_conversations);
    }

    #[test]
    fn test_speed_config() {
        let config = Gpt5Config::for_speed();
        assert_eq!(config.reasoning_effort, ReasoningEffort::None);
        assert_eq!(config.verbosity, Verbosity::Low);
    }

    #[test]
    fn test_privacy_config() {
        let config = Gpt5Config::for_privacy();
        assert!(config.zero_data_retention);
        assert!(!config.store_conversations); // ZDR enforces no storage
    }

    #[test]
    fn test_builder_methods() {
        let config = Gpt5Config::default()
            .with_reasoning_effort(ReasoningEffort::Medium)
            .with_verbosity(Verbosity::Low)
            .with_preambles(true)
            .with_storage(true)
            .with_zdr(true);

        assert_eq!(config.reasoning_effort, ReasoningEffort::Medium);
        assert_eq!(config.verbosity, Verbosity::Low);
        assert!(config.preambles.enabled);
        assert!(!config.store_conversations); // ZDR overrides storage
        assert!(config.zero_data_retention);
    }
}