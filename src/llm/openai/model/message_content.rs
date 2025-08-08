use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
}
