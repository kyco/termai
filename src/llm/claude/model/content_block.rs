use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ContentBlock {
    pub text: String,
    #[serde(rename = "type")]
    pub block_type: String,
}
