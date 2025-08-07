use crate::llm::openai::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::openai::model::chat_completion_response::ChatCompletionResponse;
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

    /// Call the traditional Chat Completions API
    /// Good for compatibility with older models
    pub async fn chat_completions(
        &self,
        request: &ChatCompletionRequest,
        api_key: &str,
    ) -> Result<ChatCompletionResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
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

        let chat_response: ChatCompletionResponse = response.json().await?;
        Ok(chat_response)
    }

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
    /// Returns true if should use Responses API, false for Chat Completions
    pub fn should_use_responses_api(
        model: &str,
        reasoning_effort: Option<&ReasoningEffort>,
        verbosity: Option<&Verbosity>,
        has_custom_tools: bool,
    ) -> bool {
        // Use Responses API for GPT-5 models
        if model.starts_with("gpt-5") {
            return true;
        }

        // Use Responses API if using new features
        if reasoning_effort.is_some() || verbosity.is_some() || has_custom_tools {
            return true;
        }

        // Default to Chat Completions for older models
        false
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
    pub fn convert_to_responses_request(
        chat_request: &ChatCompletionRequest,
    ) -> ResponsesRequest {
        // Extract the user message content as input
        let input = chat_request
            .messages
            .iter()
            .filter(|msg| msg.role == "user")
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        ResponsesRequest::with_reasoning(
            chat_request.model.clone(),
            input,
            chat_request.reasoning_effort.clone(),
        )
    }

    /// Extract text content from Responses API response
    pub fn extract_response_text(response: &ResponsesResponse) -> Option<String> {
        response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
    }

    /// Convert Responses API response to Chat Completions format (for compatibility)
    pub fn convert_to_chat_response(
        responses_response: ResponsesResponse,
    ) -> ChatCompletionResponse {
        use crate::llm::openai::model::choice::Choice;
        use crate::llm::openai::model::message_content::MessageContent;
        use crate::llm::openai::model::usage::Usage;

        let choices = responses_response
            .choices
            .into_iter()
            .map(|resp_choice| Choice {
                index: resp_choice.index as u32,
                message: MessageContent {
                    role: resp_choice.message.role,
                    content: resp_choice.message.content.unwrap_or_default(),
                },
                logprobs: None,
                finish_reason: resp_choice.finish_reason,
            })
            .collect();

        let usage = responses_response.usage.map(|resp_usage| Usage {
            prompt_tokens: resp_usage.prompt_tokens,
            completion_tokens: resp_usage.completion_tokens,
            total_tokens: resp_usage.total_tokens,
            completion_tokens_details: resp_usage.completion_tokens_details.map(|details| {
                crate::llm::openai::model::completion_token_details::CompletionTokensDetails {
                    reasoning_tokens: details.reasoning_tokens.unwrap_or(0),
                }
            }),
        });

        ChatCompletionResponse {
            id: Some(responses_response.id),
            object: Some(responses_response.object),
            created: None, // Not provided in Responses API
            model: Some(responses_response.model),
            system_fingerprint: None, // Not provided in Responses API
            choices: Some(choices),
            usage,
        }
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
        // GPT-5 models should use Responses API
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5", None, None, false));
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5-mini", None, None, false));
        assert!(Gpt5Adapter::should_use_responses_api("gpt-5-nano", None, None, false));

        // Older models should use Chat Completions by default
        assert!(!Gpt5Adapter::should_use_responses_api("gpt-4o", None, None, false));
        
        // But use Responses API if new features are requested
        assert!(Gpt5Adapter::should_use_responses_api(
            "gpt-4o", 
            Some(&ReasoningEffort::High), 
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
            "gpt-5".to_string(),
            "Hello world".to_string(),
            Some(ReasoningEffort::High),
            Some(Verbosity::Low),
        );

        assert_eq!(request.model, "gpt-5");
        assert_eq!(request.input, "Hello world");
        assert!(request.reasoning.is_some());
        assert!(request.text.is_some());
        
        if let Some(reasoning) = request.reasoning {
            assert_eq!(reasoning.effort, ReasoningEffort::High);
        }
        
        if let Some(text) = request.text {
            assert_eq!(text.verbosity, Verbosity::Low);
        }
    }
}