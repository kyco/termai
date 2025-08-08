use crate::llm::openai::model::responses_api::{ResponsesRequest, ResponsesResponse};
use anyhow::{Result, anyhow};
use reqwest::Client;

/// Adapter for the OpenAI Responses API
pub struct ResponsesAdapter;

impl ResponsesAdapter {
    /// Make a request to the Responses API
    pub async fn chat(
        request: &ResponsesRequest,
        api_key: &str,
    ) -> Result<ResponsesResponse> {
        // Create client without timeout restrictions
        let client = Client::builder()
            .build()?;

        // Log request info for debugging
        let input_size = match &request.input {
            Some(crate::llm::openai::model::responses_api::RequestInput::Text(text)) => text.len(),
            Some(crate::llm::openai::model::responses_api::RequestInput::Messages(msgs)) => 
                msgs.iter().map(|m| m.content.len()).sum(),
            None => 0,
        };
        
        if input_size > 10000 {
            eprintln!("OpenAI Request: Large input detected ({} characters)", input_size);
        }

        let response = client
            .post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&request)
            .send()
            .await?;

        // Check if the response was successful before attempting to parse
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        // Get response text first to debug parsing issues
        let response_text = response.text().await?;
        
        // Try to parse the JSON response
        match serde_json::from_str::<ResponsesResponse>(&response_text) {
            Ok(parsed_response) => Ok(parsed_response),
            Err(e) => {
                eprintln!("Failed to parse OpenAI response: {}", e);
                eprintln!("Response body: {}", response_text);
                Err(anyhow!("Error decoding response body: {}", e))
            }
        }
    }

    /// Make a streaming request to the Responses API
    #[allow(dead_code)]
    pub async fn chat_stream(
        request: &ResponsesRequest,
        api_key: &str,
    ) -> Result<reqwest::Response> {
        let mut streaming_request = request.clone();
        streaming_request.stream = Some(true);
        
        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/responses")
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&streaming_request)
            .send()
            .await?;

        Ok(response)
    }
}