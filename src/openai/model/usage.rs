use crate::openai::model::completion_token_details::CompletionTokensDetails;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
    completion_tokens_details: CompletionTokensDetails,
}
