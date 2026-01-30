use crate::llm::openai::model::compact_api::{CompactRequest, CompactResponse};
use anyhow::{Result, anyhow};
use reqwest::Client;

/// Adapter for the OpenAI Responses Compact API
pub struct CompactAdapter;

impl CompactAdapter {
    /// Make a request to the Responses Compact API
    pub async fn compact(
        request: &CompactRequest,
        api_key: &str,
    ) -> Result<CompactResponse> {
        let client = Client::builder()
            .build()?;

        let response = client
            .post("https://api.openai.com/v1/responses/compact")
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI Compact API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let response_text = response.text().await?;

        match serde_json::from_str::<CompactResponse>(&response_text) {
            Ok(parsed_response) => Ok(parsed_response),
            Err(e) => {
                eprintln!("Failed to parse OpenAI Compact response: {}", e);
                eprintln!("Response body: {}", response_text);
                Err(anyhow!("Error decoding compact response body: {}", e))
            }
        }
    }
}
