use super::MessageRepository;
use crate::repository::db::SqliteRepository;
use crate::session::entity::message_entity::MessageEntity;
use rusqlite::{params, Result, Row};

impl MessageRepository for SqliteRepository {
    type Error = rusqlite::Error;

    fn fetch_all_messages(&self) -> Result<Vec<MessageEntity>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, session_id, role, content FROM messages")?;
        let rows = stmt.query_map([], row_to_message_entity())?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }
        Ok(messages)
    }

    fn fetch_messages_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<MessageEntity>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, session_id, role, content FROM messages WHERE session_id = ?1")?;
        let rows = stmt.query_map([session_id], row_to_message_entity())?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }
        Ok(messages)
    }

    fn add_message_to_session(&self, message: &MessageEntity) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content) VALUES (?1, ?2, ?3, ?4)",
            params![
                message.id,
                message.session_id,
                message.role,
                message.content
            ],
        )?;
        Ok(())
    }
}

fn row_to_message_entity() -> fn(&Row) -> Result<MessageEntity> {
    |row| {
        let id: String = row.get(0)?;
        let session_id: String = row.get(1)?;
        let role: String = row.get(2)?;
        let content: String = row.get(3)?;

        Ok(MessageEntity::new(id, session_id, role, content))
    }
}
