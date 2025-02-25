use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_completion_response::ChatCompletionResponse;
use anyhow::Result;
use reqwest::Client;

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<ChatCompletionResponse> {
    let client = Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?
        .json::<ChatCompletionResponse>()
        .await?;
    Ok(response)
}
