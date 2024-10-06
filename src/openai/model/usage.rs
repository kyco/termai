use crate::openai::model::completion_token_details::CompletionTokensDetails;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub completion_tokens_details: CompletionTokensDetails,
}
