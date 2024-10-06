use crate::output::message;
use serde::Serialize;

use super::role::Role;

#[derive(Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
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
}
