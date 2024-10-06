use crate::openai::model::message::Message;
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
}
