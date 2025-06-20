use crate::common;
use crate::config::repository::ConfigRepository;
use crate::config::service::redacted_config;
use crate::llm::common::model::role::Role;
use crate::redactions::common::redaction_map;
use crate::redactions::redact::redact;
use crate::redactions::revert::unredact;
use crate::session::entity::session_entity::SessionEntity;
use crate::session::model::message::Message;
use chrono::{Duration, NaiveDateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub expires_at: NaiveDateTime,
    pub current: bool,
    pub messages: Vec<Message>,
    pub temporary: bool,
    pub redaction_mapping: Option<HashMap<String, String>>,
}

impl From<&SessionEntity> for Session {
    fn from(value: &SessionEntity) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            expires_at: value.expires_at,
            current: value.current == 1,
            messages: Vec::new(),
            temporary: false,
            redaction_mapping: None,
        }
    }
}

impl Session {
    pub fn new_temporary() -> Self {
        let now = Utc::now().naive_utc();
        let expires_at: NaiveDateTime = now + Duration::hours(24);
        Self {
            id: common::unique_id::generate_uuid_v4().to_string(),
            name: "temporary".to_string(),
            expires_at,
            current: true,
            messages: Vec::new(),
            temporary: true,
            redaction_mapping: None,
        }
    }

    pub fn copy_with_messages(&self, messages: Vec<Message>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at.clone(),
            current: self.current.clone(),
            messages: messages.to_vec(),
            temporary: self.temporary,
            redaction_mapping: self.redaction_mapping.clone(),
        }
    }

    pub fn add_raw_message(&mut self, message: String, role: Role) {
        self.messages.push(Message {
            id: "".to_string(),
            role,
            content: message,
        });
    }

    pub fn redact<R: ConfigRepository>(&mut self, repo: &R) {
        let mut redacted_messages = Vec::with_capacity(self.messages.len());
        let redactions = redacted_config::fetch_redactions(repo);
        let mapped_redactions = redaction_map(redactions);
        for message in self.messages.iter() {
            let redacted_input = redact(&message.content, &mapped_redactions);
            redacted_messages.push(Message {
                id: message.id.to_string(),
                role: message.role.clone(),
                content: redacted_input,
            });
        }

        self.redaction_mapping = Some(mapped_redactions);
        self.messages = redacted_messages;
    }

    pub fn unredact(&mut self) {
        let mut unredacted = Vec::with_capacity(self.messages.len());
        for message in self.messages.iter() {
            match &self.redaction_mapping {
                Some(redaction_mapping) => {
                    let content = unredact(redaction_mapping, &message.content);
                    unredacted.push(Message {
                        id: message.id.to_string(),
                        role: message.role.clone(),
                        content,
                    });
                }
                None => {
                    println!("no redaction mapping: {:#?}", message);
                    unredacted.push(message.clone())
                }
            }
        }

        self.messages = unredacted;
    }
}
