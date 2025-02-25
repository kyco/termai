use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}
