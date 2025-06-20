use crate::llm::openai::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::openai::model::chat_completion_response::ChatCompletionResponse;
use anyhow::Result;
use reqwest::Client;

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<ChatCompletionResponse> {
    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await?;
        eprintln!("OpenAI API Error: {}", error_text);
        anyhow::bail!("OpenAI API error: {}", error_text);
    }

    let parsed_response = response.json::<ChatCompletionResponse>().await?;
    Ok(parsed_response)
}
