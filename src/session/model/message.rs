use crate::llm::common::constants::SYSTEM_PROMPT;
use crate::llm::common::model::role::Role;
use crate::session::entity::message_entity::MessageEntity;

#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
}

impl From<&MessageEntity> for Message {
    fn from(entity: &MessageEntity) -> Self {
        Self {
            id: entity.id.to_string(),
            role: Role::from_str(&entity.role),
            content: entity.content.clone(),
        }
    }
}

impl Message {
    pub fn to_entity(&self, session_id: &str) -> MessageEntity {
        MessageEntity {
            id: self.id.to_string(),
            session_id: session_id.to_string(),
            role: self.role.to_string(),
            content: self.content.clone(),
        }
    }

    pub fn copy_with_id(&self, id: String) -> Self {
        Self { id, ..self.clone() }
    }
}

pub fn contains_system_prompt(messages: &Vec<Message>) -> bool {
    messages.iter().any(|m| m.role == Role::System)
}

pub fn messages_with_system_prompt(
    user_prompt: Option<String>,
    messages: &Vec<Message>,
) -> Vec<Message> {
    let mut new_messages = Vec::with_capacity(messages.len() + 1);
    let system_prompt = user_prompt.unwrap_or_else(|| SYSTEM_PROMPT.to_string());
    new_messages.push(Message {
        id: "".to_string(),
        role: Role::System,
        content: system_prompt,
    });
    for m in messages {
        new_messages.push(m.clone());
    }

    new_messages
}
