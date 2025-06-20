use termai::session::entity::{session_entity::SessionEntity, message_entity::MessageEntity};
use termai::session::model::{session::Session, message::Message};
use termai::llm::common::model::role::Role;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[allow(dead_code)]
pub fn create_test_session_entity() -> SessionEntity {
    SessionEntity {
        id: Uuid::new_v4().to_string(),
        name: "Test Session".to_string(),
        expires_at: DateTime::from_timestamp(Utc::now().timestamp() + 3600, 0).unwrap().naive_utc(),
        current: 1,
    }
}

#[allow(dead_code)]
pub fn create_test_session() -> Session {
    Session {
        id: Uuid::new_v4().to_string(),
        name: "Test Session".to_string(),
        expires_at: DateTime::from_timestamp(Utc::now().timestamp() + 3600, 0).unwrap().naive_utc(),
        current: true,
        messages: Vec::new(),
        temporary: false,
        redaction_mapping: None,
    }
}

#[allow(dead_code)]
pub fn create_test_message_entity(session_id: &str) -> MessageEntity {
    MessageEntity {
        id: Uuid::new_v4().to_string(),
        session_id: session_id.to_string(),
        role: "user".to_string(),
        content: "Test message content".to_string(),
    }
}

#[allow(dead_code)]
pub fn create_test_message() -> Message {
    Message {
        id: Uuid::new_v4().to_string(),
        role: Role::User,
        content: "Test message content".to_string(),
    }
}

#[allow(dead_code)]
pub fn create_test_config_data() -> Vec<(String, String)> {
    vec![
        ("claude_api_key".to_string(), "test_claude_key".to_string()),
        ("openai_api_key".to_string(), "test_openai_key".to_string()),
        ("current_provider".to_string(), "claude".to_string()),
    ]
}