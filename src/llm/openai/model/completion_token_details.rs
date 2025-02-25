use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: u32,
}
