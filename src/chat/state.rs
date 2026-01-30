use crate::completion::values::CompletionValues;
use serde::{Serialize, Deserialize};

/// Current chat session state including model and provider settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatState {
    /// Current AI provider (claude or openai)
    pub provider: String,

    /// Current model name
    pub model: String,

    /// Available models for the current provider
    pub available_models: Vec<String>,

    /// Whether tools (bash, file operations) are enabled for this session
    #[serde(default)]
    pub tools_enabled: bool,
}

impl ChatState {
    /// Create a new chat state with default provider and model
    pub fn new(provider: String, model: String) -> Self {
        let available_models = Self::get_models_for_provider(&provider);
        Self {
            provider,
            model,
            available_models,
            tools_enabled: false,
        }
    }

    /// Create default chat state (claude provider, default claude model)
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self::new(
            "claude".to_string(),
            "claude-sonnet-4-20250514".to_string(),
        )
    }

    /// Enable or disable tools for this session
    pub fn set_tools_enabled(&mut self, enabled: bool) {
        self.tools_enabled = enabled;
    }

    /// Toggle tools on/off and return the new state
    pub fn toggle_tools(&mut self) -> bool {
        self.tools_enabled = !self.tools_enabled;
        self.tools_enabled
    }

    /// Switch to a different provider
    pub fn switch_provider(&mut self, new_provider: String) -> Result<String, String> {
        let valid_providers = CompletionValues::provider_names();
        
        if !valid_providers.contains(&new_provider) {
            return Err(format!(
                "Invalid provider '{}'. Available providers: {}",
                new_provider,
                valid_providers.join(", ")
            ));
        }

        self.provider = new_provider.clone();
        self.available_models = Self::get_models_for_provider(&new_provider);
        
        // Set default model for the new provider
        self.model = self.get_default_model_for_provider(&new_provider);

        Ok(format!(
            "Switched to {} provider with model: {}",
            self.provider, self.model
        ))
    }

    /// Switch to a different model, automatically switching provider if needed
    pub fn switch_model(&mut self, new_model: String) -> Result<String, String> {
        let all_models = CompletionValues::model_names();
        
        if !all_models.contains(&new_model) {
            return Err(format!(
                "Invalid model '{}'. Available models: {}",
                new_model,
                all_models.join(", ")
            ));
        }

        let old_model = self.model.clone();
        let old_provider = self.provider.clone();

        // Check if model is compatible with current provider
        if !self.available_models.contains(&new_model) {
            // Automatically switch to the correct provider for this model
            let correct_provider = self.get_provider_for_model(&new_model);
            
            if correct_provider == "unknown" {
                return Err(format!(
                    "Cannot determine provider for model '{}'",
                    new_model
                ));
            }

            // Switch provider automatically
            self.provider = correct_provider.clone();
            self.available_models = Self::get_models_for_provider(&correct_provider);
            
            // Now set the model
            self.model = new_model.clone();

            Ok(format!(
                "Switched from {} ({}) to {} ({}) - automatically switched provider",
                old_model, old_provider, self.model, self.provider
            ))
        } else {
            // Model is compatible with current provider, just switch model
            self.model = new_model.clone();

            Ok(format!(
                "Switched from {} to {} (provider: {})",
                old_model, self.model, self.provider
            ))
        }
    }

    /// Get current status as a formatted string
    pub fn status(&self) -> String {
        let tools_status = if self.tools_enabled { "on" } else { "off" };
        format!(
            "🤖 Provider: {} | Model: {} | Tools: {} | Available models: {}",
            self.provider,
            self.model,
            tools_status,
            self.available_models.join(", ")
        )
    }

    /// Get models available for a specific provider
    fn get_models_for_provider(provider: &str) -> Vec<String> {
        let all_models = CompletionValues::model_names();

        match provider {
            "claude" => all_models
                .into_iter()
                .filter(|m| m.starts_with("claude"))
                .collect(),
            "openai" => all_models
                .into_iter()
                .filter(|m| (m.starts_with("gpt") || m.starts_with("o1") || m.starts_with("o3") || m.starts_with("o4") || m.starts_with("computer-use")) && !m.contains("codex"))
                .collect(),
            "openai-codex" | "codex" => all_models
                .into_iter()
                .filter(|m| m.contains("codex"))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Get default model for a provider
    fn get_default_model_for_provider(&self, provider: &str) -> String {
        match provider {
            "claude" => "claude-sonnet-4-20250514".to_string(),
            "openai" => "gpt-5.2".to_string(),
            "openai-codex" | "codex" => "gpt-5.2-codex".to_string(),
            _ => "claude-sonnet-4-20250514".to_string(),
        }
    }

    /// Determine which provider a model belongs to
    fn get_provider_for_model(&self, model: &str) -> String {
        if model.starts_with("claude") {
            "claude".to_string()
        } else if model.contains("codex") {
            "openai-codex".to_string()
        } else if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") || model.starts_with("o4") || model.starts_with("computer-use") {
            "openai".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// List available models with descriptions
    pub fn list_models(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("📋 Available models for {} provider:\n", self.provider));
        
        for (i, model) in self.available_models.iter().enumerate() {
            let marker = if model == &self.model { "👉" } else { "  " };
            let description = self.get_model_description(model);
            output.push_str(&format!("{}  {}. {} - {}\n", marker, i + 1, model, description));
        }
        
        output.push_str("\nUse '/model <name>' to switch to a different model");
        output
    }

    /// Get a description for a model
    fn get_model_description(&self, model: &str) -> &'static str {
        match model {
            // GPT-5.2 series
            "gpt-5.2" => "Most intelligent model, best for complex reasoning and coding",
            "gpt-5.2-pro" => "Extra compute for harder problems; higher latency/cost",
            "gpt-5.2-chat-latest" => "Chat-optimized GPT-5.2 variant (latest)",
            // GPT-5.2 Codex models (ChatGPT Plus/Pro via OAuth)
            "gpt-5.2-codex" => "Full Codex model for ChatGPT Plus/Pro subscribers",
            "gpt-5.1-codex-mini" => "Faster Codex mini model for quick responses",
            "gpt-5.1-codex-max" => "Maximum capability Codex model for complex tasks",
            // GPT-5 series
            "gpt-5.1" => "Previous GPT-5.1 flagship model (deprecated)",
            "gpt-5-mini" => "Cost-optimized reasoning, balances speed/cost/capability",
            "gpt-5-nano" => "High-throughput, simple instruction-following",
            // o3 series (deep research)
            "o3-deep-research" => "Most powerful deep research model",
            "o4-mini-deep-research" => "Faster, more affordable deep research model",
            "o3-pro" => "Version of o3 with more compute for better responses",
            "o3" => "Most powerful reasoning model",
            "o4-mini" => "Faster, more affordable reasoning model",
            "o3-mini" => "Small model alternative to o3",
            // GPT-4.1 series
            "gpt-4.1" => "Excels at function calling and instruction following",
            "gpt-4.1-mini" => "Balanced for intelligence, speed, and cost",
            "gpt-4.1-nano" => "Fastest, most cost-effective GPT-4.1 model",
            // GPT-4.5 series
            "gpt-4.5-preview" => "Preview of GPT-4.5 capabilities (deprecated)",
            // GPT-4o series
            "gpt-4o" => "High-intelligence flagship model for complex tasks",
            "gpt-4o-mini" => "Affordable and intelligent small model",
            "gpt-4o-search-preview" => "GPT model optimized for web search",
            "gpt-4o-mini-search-preview" => "Fast, affordable small model for web search",
            // o1 series (reasoning)
            "o1-pro" => "Version of o1 with more compute for better responses",
            "o1" => "Previous full o-series reasoning model",
            "o1-mini" => "Small model alternative to o1 (deprecated)",
            "o1-preview" => "Preview of first o-series reasoning model (deprecated)",
            // Specialized models
            "computer-use-preview" => "Specialized model for computer use tool",
            // Legacy models
            "gpt-4" => "Previous generation GPT-4 model",
            "gpt-4-turbo" => "Faster version of GPT-4",
            "gpt-3.5-turbo" => "Fast, cost-effective model for simple tasks",
            // Claude 4 series models
            "claude-opus-4-1-20250805" => "Latest Opus model with enhanced capabilities",
            "claude-opus-4-20250514" => "Most powerful Claude model for highly complex tasks", 
            "claude-sonnet-4-20250514" => "Best overall Claude model, excels at writing and complex tasks",
            // Claude 3.7 series models
            "claude-3-7-sonnet-20250219" => "Enhanced Sonnet model with improved performance",
            "claude-3-7-sonnet-latest" => "Latest version of Claude 3.7 Sonnet",
            // Claude 3.5 series models
            "claude-3-5-sonnet-20241022" => "Excellent model for writing and complex tasks",
            "claude-3-5-haiku-20241022" => "Fast model for quick responses",
            "claude-3-5-haiku-latest" => "Latest version of Claude 3.5 Haiku",
            // Claude 3 legacy models
            "claude-3-opus-20240229" => "Previous generation powerful model for complex tasks",
            "claude-3-sonnet-20240229" => "Balanced legacy model for general use",
            "claude-3-haiku-20240307" => "Fast and efficient legacy model for simple tasks",
            _ => "AI language model"
        }
    }

    /// Validate if a model switch is valid
    #[allow(dead_code)]
    pub fn can_switch_to_model(&self, model: &str) -> bool {
        self.available_models.contains(&model.to_string())
    }

    /// Validate if a provider switch is valid
    #[allow(dead_code)]
    pub fn can_switch_to_provider(&self, provider: &str) -> bool {
        CompletionValues::provider_names().contains(&provider.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = ChatState::default();
        assert_eq!(state.provider, "claude");
        assert_eq!(state.model, "claude-sonnet-4-20250514");
        assert!(!state.available_models.is_empty());
    }

    #[test]
    fn test_provider_switching() {
        let mut state = ChatState::default();

        // Switch to OpenAI
        let result = state.switch_provider("openai".to_string());
        assert!(result.is_ok());
        assert_eq!(state.provider, "openai");
        assert_eq!(state.model, "gpt-5.2"); // Should default to GPT-5.2

        // Switch to invalid provider
        let result = state.switch_provider("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_model_switching() {
        let mut state = ChatState::new("openai".to_string(), "gpt-5.2".to_string());

        // Switch to valid model within same provider
        if state.available_models.contains(&"gpt-5-mini".to_string()) {
            let result = state.switch_model("gpt-5-mini".to_string());
            assert!(result.is_ok());
            assert_eq!(state.model, "gpt-5-mini");
            assert_eq!(state.provider, "openai"); // Provider should remain the same
        }

        // Switch to model from different provider - should automatically switch provider
        let result = state.switch_model("claude-sonnet-4-20250514".to_string());
        assert!(result.is_ok());
        assert_eq!(state.model, "claude-sonnet-4-20250514");
        assert_eq!(state.provider, "claude"); // Provider should automatically switch
    }

    #[test]
    fn test_model_validation() {
        let state = ChatState::new("openai".to_string(), "gpt-5.2".to_string());

        assert!(state.can_switch_to_provider("claude"));
        assert!(state.can_switch_to_provider("openai"));
        assert!(!state.can_switch_to_provider("invalid"));

        // Should be able to switch to models within the same provider
        if state.available_models.contains(&"gpt-5-mini".to_string()) {
            assert!(state.can_switch_to_model("gpt-5-mini"));
        }
    }

    #[test]
    fn test_status_display() {
        let state = ChatState::new("openai".to_string(), "gpt-5.2".to_string());
        let status = state.status();

        assert!(status.contains("openai"));
        assert!(status.contains("gpt-5.2"));
        assert!(status.contains("Available models:"));
    }
}
