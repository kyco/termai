use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_completion_response::ChatCompletionResponse;
use anyhow::Result;
use reqwest::{Client, StatusCode};

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<(StatusCode, ChatCompletionResponse)> {
    // Create client without timeout restrictions
    let client = Client::builder()
        .build()?;

    // Log request info for debugging
    let input_size: usize = request.messages.iter()
        .map(|msg| msg.content.len())
        .sum();
        
    if input_size > 10000 {
        eprintln!("Claude Request: Large input detected ({} characters)", input_size);
    }

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await?;
        eprintln!("Claude API Error: {}", error_text);
        anyhow::bail!("Claude API error: {}", error_text);
    }

    let parsed_response = response.json::<ChatCompletionResponse>().await?;
    Ok((status, parsed_response))
}
