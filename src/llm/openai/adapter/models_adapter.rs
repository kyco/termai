use anyhow::{anyhow, Result};
use reqwest::Client;

use crate::llm::openai::model::models_api::{filter_chat_models, ModelObject, ModelsListResponse};

/// Adapter for the OpenAI Models API
pub struct ModelsAdapter;

impl ModelsAdapter {
    /// Fetch all available models from OpenAI API
    pub async fn list_models(api_key: &str) -> Result<Vec<ModelObject>> {
        let client = Client::builder().build()?;

        let response = client
            .get("https://api.openai.com/v1/models")
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch models: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            // Provide helpful error messages for common cases
            return match status.as_u16() {
                401 => Err(anyhow!(
                    "Authentication failed: Invalid API key. Please check your OpenAI API key."
                )),
                429 => Err(anyhow!(
                    "Rate limited: Too many requests. Please try again later."
                )),
                _ => Err(anyhow!("Models API error {}: {}", status, body)),
            };
        }

        let models_response: ModelsListResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse models response: {}", e))?;

        Ok(models_response.data)
    }

    /// Fetch and filter to only chat-capable models
    pub async fn list_chat_models(api_key: &str) -> Result<Vec<ModelObject>> {
        let all_models = Self::list_models(api_key).await?;
        Ok(filter_chat_models(&all_models))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests require a valid API key
    // These unit tests verify the adapter's structure and error handling logic

    #[test]
    fn test_adapter_exists() {
        // Verify the adapter type exists
        let _ = ModelsAdapter;
    }

    #[tokio::test]
    async fn test_list_models_with_invalid_key() {
        // Test with an obviously invalid key to verify error handling
        let result = ModelsAdapter::list_models("sk-invalid-key-for-testing").await;

        // Should fail with auth error
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // Either authentication failed or network error
        assert!(err.contains("Authentication failed") || err.contains("Failed to fetch"));
    }

    #[tokio::test]
    async fn test_list_chat_models_with_invalid_key() {
        // Test with an obviously invalid key to verify error handling
        let result = ModelsAdapter::list_chat_models("sk-invalid-key-for-testing").await;

        // Should fail with auth error
        assert!(result.is_err());
    }
}
