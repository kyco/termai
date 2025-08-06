use crate::llm::openai::model::chat_message::ChatMessage;
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub reasoning_effort: ReasoningEffort,
}
