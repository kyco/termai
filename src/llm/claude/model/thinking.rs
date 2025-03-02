use crate::llm::claude::model::thinking_type::ThinkingType;
use serde::Serialize;

#[derive(Serialize)]
pub struct Thinking {
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    pub budget_tokens: u32,
}