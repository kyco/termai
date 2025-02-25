use serde::Serialize;
use crate::llm::claude::model::chat_message::ChatMessage;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}
