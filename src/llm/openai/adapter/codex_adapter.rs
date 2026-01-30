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
    /// Make a streaming request to the Codex API and collect the full response
    ///
    /// Uses OAuth access token for authentication instead of API key.
    /// The Codex API requires streaming, so we parse SSE events and extract the final response.
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

        // Parse SSE streaming response
        let response_text = response.text().await?;
        Self::parse_sse_response(&response_text)
    }

    /// Parse SSE (Server-Sent Events) response and extract the final CodexResponse
    fn parse_sse_response(sse_text: &str) -> Result<CodexResponse> {
        let mut final_response: Option<CodexResponse> = None;

        for line in sse_text.lines() {
            // SSE format: "data: {...json...}" or "event: ..." etc.
            if let Some(data) = line.strip_prefix("data: ") {
                // Skip empty data or [DONE] marker
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }

                // Try to parse as JSON
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                    // Check if this is a response.completed event with the full response
                    if event.get("type").and_then(|t| t.as_str()) == Some("response.completed") {
                        if let Some(response_obj) = event.get("response") {
                            if let Ok(parsed) = serde_json::from_value::<CodexResponse>(response_obj.clone()) {
                                final_response = Some(parsed);
                            }
                        }
                    }
                    // Also check for response.done which might contain the response directly
                    else if event.get("type").and_then(|t| t.as_str()) == Some("response.done") {
                        if let Some(response_obj) = event.get("response") {
                            if let Ok(parsed) = serde_json::from_value::<CodexResponse>(response_obj.clone()) {
                                final_response = Some(parsed);
                            }
                        }
                    }
                    // Try parsing the event itself as a CodexResponse (some APIs return it directly)
                    else if event.get("id").is_some() && event.get("output").is_some() {
                        if let Ok(parsed) = serde_json::from_value::<CodexResponse>(event) {
                            final_response = Some(parsed);
                        }
                    }
                }
            }
        }

        final_response.ok_or_else(|| {
            anyhow!("No valid response found in SSE stream. Raw response:\n{}",
                sse_text.chars().take(1000).collect::<String>())
        })
    }
}
