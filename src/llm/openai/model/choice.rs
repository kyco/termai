use crate::llm::openai::model::message_content::MessageContent;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Choice {
    #[allow(dead_code)]
    pub index: u32,
    pub message: MessageContent,
    #[allow(dead_code)]
    pub logprobs: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub finish_reason: String,
}
