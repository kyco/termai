use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

#[async_trait::async_trait]
pub trait ApiKeyValidator {
    async fn validate(&self, api_key: &str) -> Result<()>;
}

pub struct ClaudeValidator {
    client: Client,
}

impl ClaudeValidator {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl ApiKeyValidator for ClaudeValidator {
    async fn validate(&self, api_key: &str) -> Result<()> {
        // Test the Claude API key with a minimal request
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1,
                "messages": [
                    {
                        "role": "user",
                        "content": "test"
                    }
                ]
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            match status.as_u16() {
                401 => Err(anyhow!(
                    "Invalid API key. Please check your Claude API key."
                )),
                403 => Err(anyhow!(
                    "API key not authorized. Please check your Claude account permissions."
                )),
                429 => Err(anyhow!(
                    "Rate limit exceeded. Please try again in a moment."
                )),
                _ => Err(anyhow!(
                    "API validation failed: {} - {}",
                    status,
                    error_text
                )),
            }
        }
    }
}

pub struct OpenAIValidator {
    client: Client,
}

impl OpenAIValidator {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }
}

#[async_trait::async_trait]
impl ApiKeyValidator for OpenAIValidator {
    async fn validate(&self, api_key: &str) -> Result<()> {
        // Test the OpenAI API key with a minimal request
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&json!({
                "model": "gpt-3.5-turbo",
                "messages": [
                    {
                        "role": "user",
                        "content": "test"
                    }
                ],
                "max_tokens": 1
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            match status.as_u16() {
                401 => Err(anyhow!(
                    "Invalid API key. Please check your OpenAI API key."
                )),
                403 => Err(anyhow!(
                    "API key not authorized. Please check your OpenAI account permissions."
                )),
                429 => Err(anyhow!(
                    "Rate limit exceeded. Please try again in a moment."
                )),
                _ => Err(anyhow!(
                    "API validation failed: {} - {}",
                    status,
                    error_text
                )),
            }
        }
    }
}
