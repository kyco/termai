use crate::output::message;
use serde::Serialize;

use super::role::Role;

#[derive(Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    pub fn to_output_message(&self) -> message::Message {
        message::Message {
            role: Role::from_str(&self.role),
            message: self.content.to_string(),
        }
    }

    pub fn prepend_content(&self, text: &str) -> Self {
        let new_content = format!("{}\n\n{}", text, self.content);
        Self {
            role: self.role.to_string(),
            content: new_content,
        }
    }

    pub fn remove_from_content(&self, text: &str) -> Self {
        let new_content = self.content.replace(text, "");
        let new_content = new_content.trim();
        Self {
            role: self.role.to_string(),
            content: new_content.to_string(),
        }
    }
}
