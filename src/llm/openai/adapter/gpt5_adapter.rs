// Note: These types are temporarily maintained for compatibility during migration
// They will be removed once all code migrates to Responses API
use crate::llm::openai::model::responses_api::{ResponsesRequest, ResponsesResponse};
use crate::llm::openai::model::verbosity::Verbosity;
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use anyhow::{anyhow, Result};
use reqwest::Client;

/// Enhanced OpenAI adapter with GPT-5 features
/// Supports both Chat Completions and Responses API
pub struct Gpt5Adapter {
    client: Client,
    base_url: String,
}

impl Gpt5Adapter {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    // Chat Completions API removed - migrated to Responses API
    /// Call the new Responses API (preferred for GPT-5)
    /// Optimized for reasoning models with better caching and performance
    pub async fn responses(
        &self,
        request: &ResponsesRequest,
        api_key: &str,
    ) -> Result<ResponsesResponse> {
        let url = format!("{}/responses", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let responses_response: ResponsesResponse = response.json().await?;
        Ok(responses_response)
    }

    /// Intelligent API selection based on model and features
    /// TODO: Remove when fully migrated - we now always use Responses API
    #[allow(dead_code)]
    pub fn should_use_responses_api(
        _model: &str,
        _reasoning_effort: Option<&ReasoningEffort>,
        _verbosity: Option<&Verbosity>,
        _has_custom_tools: bool,
    ) -> bool {
        // We now always use Responses API
        true
    }

    /// Create a simple Responses API request
    pub fn create_simple_request(
        model: String,
        input: String,
        reasoning_effort: Option<ReasoningEffort>,
        verbosity: Option<Verbosity>,
    ) -> ResponsesRequest {
        let mut request = ResponsesRequest::simple(model, input);
        
        if let Some(effort) = reasoning_effort {
            if let Some(ref mut reasoning) = request.reasoning {
                reasoning.effort = effort;
            }
        }
        
        if let Some(verb) = verbosity {
            if let Some(ref mut text) = request.text {
                text.verbosity = verb;
            }
        }
        
        request
    }

    /// Convert Chat Completions request to Responses API format
    /// TODO: Remove when fully migrated to Responses API
    #[allow(dead_code)]
    pub fn convert_to_responses_request(
        _chat_request: String, // Simplified for migration
    ) -> ResponsesRequest {
        // Deprecated during migration
        ResponsesRequest::simple("gpt-5.2".to_string(), "Migration in progress".to_string())
    }

    /// Extract text content from Responses API response
    pub fn extract_response_text(response: &ResponsesResponse) -> Option<String> {
        // TODO: Update to use new response format
        response.output.iter()
            .find_map(|output| match output {
                crate::llm::openai::model::responses_api::ResponseOutput::Message { content, .. } => {
                    content.iter().map(|item| match item {
                        crate::llm::openai::model::responses_api::ContentItem::OutputText { text, .. } => text.clone(),
                    }).next()
                }
                _ => None,
            })
    }

    /// Convert Responses API response to Chat Completions format (for compatibility)
    /// TODO: Remove this when fully migrated to Responses API
    #[allow(dead_code)]
    pub fn convert_to_chat_response(
        _responses_response: ResponsesResponse,
    ) -> String {
        // Deprecated - use Responses API directly
        "Migration in progress - use Responses API directly".to_string()
    }
}

impl Default for Gpt5Adapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_responses_api() {
        // GPT-5.2 models should use Responses API
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5.2", None, None, false));
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5-mini", None, None, false));
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5-nano", None, None, false));

        // All models now use Responses API (migration complete)
        assert!(Gpt5Adapter::should_use_responses_api("gpt-4o", None, None, false));
        
        // But use Responses API if new features are requested
        assert!(Gpt5Adapter::should_use_responses_api(
            "gpt-4o", 
            Some(&ReasoningEffort::Medium), 
            None, 
            false
        ));
        assert!(Gpt5Adapter::should_use_responses_api(
            "gpt-4o", 
            None, 
            Some(&Verbosity::Low), 
            false
        ));
        assert!(Gpt5Adapter::should_use_responses_api("gpt-4o", None, None, true));
    }

    #[test]
    fn test_create_simple_request() {
        let request = Gpt5Adapter::create_simple_request(
            "gpt-5.2".to_string(),
            "Hello world".to_string(),
            Some(ReasoningEffort::Medium),
            Some(Verbosity::Low),
        );

        assert_eq!(request.model, "gpt-5.2");
        // Check input content (it's now wrapped in RequestInput enum)
        if let Some(crate::llm::openai::model::responses_api::RequestInput::Text(text)) = &request.input {
            assert_eq!(text, "Hello world");
        } else {
            panic!("Expected text input");
        }
        assert!(request.reasoning.is_some());
        assert!(request.text.is_some());
        
        if let Some(reasoning) = request.reasoning {
            assert_eq!(reasoning.effort, ReasoningEffort::Medium);
        }
        
        if let Some(text) = request.text {
            assert_eq!(text.verbosity, Verbosity::Low);
        }
    }
}
