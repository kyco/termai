use crate::llm::openai::model::message_content::MessageContent;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Choice {
    pub index: u32,
    pub message: MessageContent,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: Option<String>,
}
