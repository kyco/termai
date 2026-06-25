use super::MessageRepository;
use crate::repository::db::SqliteRepository;
use crate::session::entity::message_entity::MessageEntity;
use rusqlite::{params, Result, Row};

impl MessageRepository for SqliteRepository {
    type Error = rusqlite::Error;

    fn fetch_messages_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<MessageEntity>, Self::Error> {
        // Order by the implicit rowid so messages always come back in insertion
        // order. The primary key is a random UUID (TEXT), so without an explicit
        // ORDER BY a future index or query-planner change could scramble the
        // conversation; rowid is monotonic per insert and needs no schema change.
        let mut stmt = self
            .conn
            .prepare("SELECT id, session_id, role, content, message_type, compaction_metadata FROM messages WHERE session_id = ?1 ORDER BY rowid ASC")?;
        let rows = stmt.query_map([session_id], row_to_message_entity())?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }
        Ok(messages)
    }

    fn add_message_to_session(&self, message: &MessageEntity) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, message_type, compaction_metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                message.id,
                message.session_id,
                message.role,
                message.content,
                message.message_type,
                message.compaction_metadata
            ],
        )?;
        Ok(())
    }

    fn delete_messages_for_session(&self, session_id: &str) -> Result<(), Self::Error> {
        self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id],
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
        let message_type: String = row.get(4).unwrap_or_else(|_| "standard".to_string());
        let compaction_metadata: Option<String> = row.get(5).ok();

        Ok(MessageEntity::new_with_type(
            id,
            session_id,
            role,
            content,
            message_type,
            compaction_metadata,
        ))
    }
}
