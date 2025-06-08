use crate::llm::openai::model::choice::Choice;
use crate::llm::openai::model::usage::Usage;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    #[allow(dead_code)]
    pub id: Option<String>,
    #[allow(dead_code)]
    pub object: Option<String>,
    #[allow(dead_code)]
    pub created: Option<u64>,
    #[allow(dead_code)]
    pub model: Option<String>,
    #[allow(dead_code)]
    pub system_fingerprint: Option<String>,
    pub choices: Option<Vec<Choice>>,
    #[allow(dead_code)]
    pub usage: Option<Usage>,
}
