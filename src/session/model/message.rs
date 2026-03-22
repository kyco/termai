use serde::{Serialize, Deserialize};
use crate::llm::common::constants::SYSTEM_PROMPT;
use crate::llm::common::model::role::Role;
use crate::session::entity::message_entity::MessageEntity;

/// Type of message - standard text or compacted history
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[derive(Default)]
pub enum MessageType {
    /// Standard text message
    #[default]
    Standard,
    /// Compacted conversation history from OpenAI
    Compaction {
        /// Compaction ID from OpenAI
        compaction_id: String,
        /// Encrypted content blob
        encrypted_content: String,
    },
}


#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub message_type: MessageType,
}

impl From<&MessageEntity> for Message {
    fn from(entity: &MessageEntity) -> Self {
        let message_type = match &entity.message_type[..] {
            "compaction" => {
                // Parse compaction metadata from JSON
                if let Some(ref metadata) = entity.compaction_metadata {
                    if let Ok(parsed) = serde_json::from_str::<CompactionMetadata>(metadata) {
                        MessageType::Compaction {
                            compaction_id: parsed.compaction_id,
                            encrypted_content: parsed.encrypted_content,
                        }
                    } else {
                        MessageType::Standard
                    }
                } else {
                    MessageType::Standard
                }
            }
            _ => MessageType::Standard,
        };

        Self {
            id: entity.id.to_string(),
            role: Role::from_str(&entity.role),
            content: entity.content.clone(),
            message_type,
        }
    }
}

/// Helper struct for serializing compaction metadata
#[derive(Serialize, Deserialize)]
struct CompactionMetadata {
    compaction_id: String,
    encrypted_content: String,
}

impl Message {
    pub fn to_entity(&self, session_id: &str) -> MessageEntity {
        let (message_type, compaction_metadata) = match &self.message_type {
            MessageType::Standard => ("standard".to_string(), None),
            MessageType::Compaction { compaction_id, encrypted_content } => {
                let metadata = CompactionMetadata {
                    compaction_id: compaction_id.clone(),
                    encrypted_content: encrypted_content.clone(),
                };
                let metadata_json = serde_json::to_string(&metadata).ok();
                ("compaction".to_string(), metadata_json)
            }
        };

        MessageEntity {
            id: self.id.to_string(),
            session_id: session_id.to_string(),
            role: self.role.to_string(),
            content: self.content.clone(),
            message_type,
            compaction_metadata,
        }
    }

    pub fn copy_with_id(&self, id: String) -> Self {
        Self { id, ..self.clone() }
    }

    /// Create a new standard message
    pub fn new(id: String, role: Role, content: String) -> Self {
        Self {
            id,
            role,
            content,
            message_type: MessageType::Standard,
        }
    }

    /// Create a new compaction message
    #[allow(dead_code)]
    pub fn new_compaction(id: String, role: Role, content: String, compaction_id: String, encrypted_content: String) -> Self {
        Self {
            id,
            role,
            content,
            message_type: MessageType::Compaction {
                compaction_id,
                encrypted_content,
            },
        }
    }

    /// Check if this is a compaction message
    #[allow(dead_code)]
    pub fn is_compaction(&self) -> bool {
        matches!(self.message_type, MessageType::Compaction { .. })
    }
}

#[allow(dead_code)]
pub fn contains_system_prompt(messages: &[Message]) -> bool {
    messages.iter().any(|m| m.role == Role::System)
}

#[allow(dead_code)]
pub fn messages_with_system_prompt(
    user_prompt: Option<String>,
    messages: &[Message],
) -> Vec<Message> {
    let mut new_messages = Vec::with_capacity(messages.len() + 1);
    let system_prompt = user_prompt.unwrap_or_else(|| SYSTEM_PROMPT.to_string());
    new_messages.push(Message::new(
        "".to_string(),
        Role::System,
        system_prompt,
    ));
    for m in messages {
        new_messages.push(m.clone());
    }

    new_messages
}
