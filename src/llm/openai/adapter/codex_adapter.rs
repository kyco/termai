//! Adapter for the OpenAI Codex backend API
//!
//! This adapter handles requests to the Codex API endpoint
//! (chatgpt.com/backend-api/codex/responses) using OAuth tokens.

use crate::llm::openai::model::codex_api::{CodexRequest, CodexResponse};
use anyhow::{anyhow, Result};
use reqwest::Client;

/// The Codex API endpoint
const CODEX_API_ENDPOINT: &str = "https://chatgpt.com/backend-api/codex/responses";

/// Adapter for the OpenAI Codex API
pub struct CodexAdapter;

impl CodexAdapter {
    /// Make a request to the Codex API
    ///
    /// Uses OAuth access token for authentication instead of API key.
    pub async fn chat(
        request: &CodexRequest,
        access_token: &str,
    ) -> Result<CodexResponse> {
        let client = Client::builder()
            .build()?;

        // Log request info for debugging large inputs
        let input_size = match &request.input {
            Some(crate::llm::openai::model::codex_api::CodexInput::Text(text)) => text.len(),
            Some(crate::llm::openai::model::codex_api::CodexInput::Messages(msgs)) => {
                msgs.iter().map(|m| m.content.len()).sum()
            }
            None => 0,
        };

        if input_size > 10000 {
            eprintln!("Codex Request: Large input detected ({} characters)", input_size);
        }

        let response = client
            .post(CODEX_API_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .bearer_auth(access_token)
            .json(&request)
            .send()
            .await?;

        // Check if the response was successful
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Check for specific error cases
            if status.as_u16() == 401 {
                return Err(anyhow!(
                    "Authentication failed. Your Codex session may have expired. Run 'termai config login-codex' to re-authenticate."
                ));
            }

            return Err(anyhow!(
                "Codex API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        // Parse the response
        let response_text = response.text().await?;

        match serde_json::from_str::<CodexResponse>(&response_text) {
            Ok(parsed_response) => Ok(parsed_response),
            Err(e) => {
                eprintln!("Failed to parse Codex response: {}", e);
                eprintln!("Response body: {}", response_text);
                Err(anyhow!("Error decoding Codex response: {}", e))
            }
        }
    }

    /// Make a streaming request to the Codex API
    #[allow(dead_code)]
    pub async fn chat_stream(
        request: &CodexRequest,
        access_token: &str,
    ) -> Result<reqwest::Response> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);

        let client = Client::new();
        let response = client
            .post(CODEX_API_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .bearer_auth(access_token)
            .json(&streaming_request)
            .send()
            .await?;

        Ok(response)
    }
}
