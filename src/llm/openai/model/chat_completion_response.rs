use crate::llm::openai::model::choice::Choice;
use crate::llm::openai::model::usage::Usage;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ChatCompletionResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<u64>,
    pub model: Option<String>,
    pub system_fingerprint: Option<String>,
    pub choices: Option<Vec<Choice>>,
    pub usage: Option<Usage>,
}
