use serde::Deserialize;
use crate::llm::claude::model::content_block::ContentBlock;
use crate::llm::claude::model::usage::Usage;

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub content: Vec<ContentBlock>,
    pub id: String,
    pub model: String,
    pub role: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    #[serde(rename = "type")]
    pub response_type: String,
    pub usage: Usage,
}
