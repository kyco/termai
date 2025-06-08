use crate::llm::claude::model::content_block::ContentBlock;
use crate::llm::claude::model::usage::Usage;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    pub content: Vec<ContentBlock>,
    #[allow(dead_code)]
    pub id: String,
    #[allow(dead_code)]
    pub model: String,
    #[allow(dead_code)]
    pub role: String,
    pub stop_reason: String,
    #[allow(dead_code)]
    pub stop_sequence: Option<String>,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub response_type: String,
    #[allow(dead_code)]
    pub usage: Usage,
}
