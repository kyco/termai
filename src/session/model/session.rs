use crate::config::repository::ConfigRepository;
use crate::openai::model::role::Role;
use crate::redactions::redact::redact;
use crate::redactions::revert::unredact;
use crate::session::entity::session_entity::SessionEntity;
use crate::session::model::message::Message;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub expires_at: NaiveDateTime,
    pub current: bool,
    pub messages: Vec<Message>,
}

impl From<&SessionEntity> for Session {
    fn from(value: &SessionEntity) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            expires_at: value.expires_at,
            current: value.current == 1,
            messages: Vec::new(),
        }
    }
}

impl Session {
    pub fn copy_with_messages(&self, messages: Vec<Message>) -> Self {
        Self {
            id: self.id.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at.clone(),
            current: self.current.clone(),
            messages: messages.to_vec(),
        }
    }

    pub fn add_raw_message(&mut self, message: String, role: Role) {
        self.messages.push(Message {
            id: "".to_string(),
            role,
            content: message,
            redaction_mapping: None,
        });
    }

    pub fn redact<R: ConfigRepository>(&mut self, repo: &R) {
        let mut redacted_messages = Vec::with_capacity(self.messages.len());
        for message in self.messages.iter() {
            let (redacted_input, mapped_redactions) = redact(repo, &message.content);
            redacted_messages.push(Message {
                id: message.id.to_string(),
                role: message.role.clone(),
                content: redacted_input,
                redaction_mapping: Some(mapped_redactions),
            });
        }

        self.messages = redacted_messages;
    }

    pub fn unredact(&mut self) {
        let mut unredacted = Vec::with_capacity(self.messages.len());
        for message in self.messages.iter() {
            match &message.redaction_mapping {
                Some(redaction_mapping) => {
                    let content = unredact(redaction_mapping, &message.content);
                    unredacted.push(Message {
                        id: message.id.to_string(),
                        role: message.role.clone(),
                        content,
                        redaction_mapping: message.redaction_mapping.clone(),
                    });
                }
                None => unredacted.push(message.clone()),
            }
        }

        self.messages = unredacted;
    }
}
