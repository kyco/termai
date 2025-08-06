use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: u32,
}
