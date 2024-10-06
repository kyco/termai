use crate::openai::model::message_content::MessageContent;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: MessageContent,
    pub logprobs: Option<serde_json::Value>,
    pub finish_reason: String,
}
