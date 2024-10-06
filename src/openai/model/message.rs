use serde::Serialize;

#[derive(Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}
