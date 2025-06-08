use super::entity::session_entity::SessionEntity;
use crate::session::entity::message_entity::MessageEntity;
use chrono::NaiveDateTime;
use std::fmt::Debug;

pub(crate) mod session_repository;
pub(crate) mod message_repository;

pub trait SessionRepository
where
    Self::Error: Debug,
{
    type Error;

    fn fetch_all_sessions(&self) -> Result<Vec<SessionEntity>, Self::Error>;
    #[allow(dead_code)]
    fn fetch_current_session(&self) -> Result<SessionEntity, Self::Error>;
    fn fetch_session_by_name(&self, name: &str) -> Result<SessionEntity, Self::Error>;
    fn fetch_session_by_id(&self, id: &str) -> Result<SessionEntity, Self::Error>;
    fn add_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error>;
    fn update_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error>;
    fn remove_current_from_all(&self) -> Result<(), Self::Error>;
}

pub trait MessageRepository
where
    Self::Error: Debug,
{
    type Error;

    #[allow(dead_code)]
    fn fetch_all_messages(&self) -> Result<Vec<MessageEntity>, Self::Error>;
    fn fetch_messages_for_session(&self, session_id: &str) -> Result<Vec<MessageEntity>, Self::Error>;
    fn add_message_to_session(&self, message: &MessageEntity) -> Result<(), Self::Error>;
}
