use crate::openai::model::chat_message::ChatMessage;
use serde::Serialize;
use crate::openai::model::reasoning_effort::ReasoningEffort;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub reasoning_effort: ReasoningEffort
}
