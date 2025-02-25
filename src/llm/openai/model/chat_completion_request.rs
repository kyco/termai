use crate::llm::openai::model::chat_message::ChatMessage;
use serde::Serialize;
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub reasoning_effort: ReasoningEffort
}
