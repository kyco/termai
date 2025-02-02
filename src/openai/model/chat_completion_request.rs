use crate::openai::model::chat_message::ChatMessage;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
}
