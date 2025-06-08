use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CompletionTokensDetails {
    #[allow(dead_code)]
    pub reasoning_tokens: u32,
}
