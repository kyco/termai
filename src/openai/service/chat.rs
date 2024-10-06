use crate::openai::{
    adapter::open_ai_adapter,
    model::{
        chat_completion_request::ChatCompletionRequest,
        chat_completion_response::ChatCompletionResponse, message::Message, model::Model,
    },
};
use anyhow::Result;

const SYSTEM_PROMPT: &str = "
You're an assistant in the terminal.
You will keep your answers brief as the user is chatting to you from the command line.
You will never output markdown, only ASCII text.
The user also loves seeing ASCII art where appropriate.
You will limit your line length to 80 characters.";

pub async fn chat(api_key: &str, data: &str) -> Result<ChatCompletionResponse> {
    let request = ChatCompletionRequest {
        model: Model::Gpt4o.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: SYSTEM_PROMPT.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: data.to_string(),
            },
        ],
    };
    open_ai_adapter::chat(&request, api_key).await
}
