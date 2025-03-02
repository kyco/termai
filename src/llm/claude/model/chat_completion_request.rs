use crate::llm::claude::model::chat_message::ChatMessage;
use crate::llm::claude::model::thinking::Thinking;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Thinking>,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}
