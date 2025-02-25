use crate::llm::openai::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::openai::model::chat_completion_response::ChatCompletionResponse;
use anyhow::Result;
use reqwest::Client;

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<ChatCompletionResponse> {
    let client = Client::new();
    let response: ChatCompletionResponse = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}
