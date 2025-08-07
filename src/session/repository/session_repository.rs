use super::SessionRepository;
use crate::{repository::db::SqliteRepository, session::entity::session_entity::SessionEntity};
use chrono::NaiveDateTime;
use rusqlite::{params, Result, Row};

const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl SessionRepository for SqliteRepository {
    type Error = rusqlite::Error;

    fn fetch_all_sessions(&self) -> Result<Vec<SessionEntity>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, expires_at, current FROM sessions")?;
        let rows = stmt.query_map([], row_to_session_entity())?;

        let mut sessions = Vec::new();
        for session in rows {
            sessions.push(session?);
        }
        Ok(sessions)
    }

    fn fetch_session_by_name(&self, name: &str) -> Result<SessionEntity, Self::Error> {
        let session = self.conn.query_row(
            "SELECT id, name, expires_at, current FROM sessions WHERE name = ?1",
            params![name],
            row_to_session_entity(),
        )?;

        Ok(session)
    }

    fn add_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error> {
        let expires_at_str = expires_at.format(DATE_TIME_FORMAT).to_string();
        let current_i = if current { 1 } else { 0 };
        self.conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, expires_at_str, current_i],
        )?;
        Ok(())
    }

    fn update_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error> {
        let expires_at_str = expires_at.format(DATE_TIME_FORMAT).to_string();
        let current_i = if current { 1 } else { 0 };
        self.conn.execute(
            "UPDATE sessions SET name = ?1, expires_at = ?2, current = ?3 WHERE id = ?4",
            params![name, expires_at_str, current_i, id],
        )?;
        Ok(())
    }

    fn remove_current_from_all(&self) -> Result<(), Self::Error> {
        self.conn
            .execute("UPDATE sessions SET current = 0", params![])?;
        Ok(())
    }

    fn delete_session(&self, session_id: &str) -> Result<(), Self::Error> {
        // Start a transaction to ensure both session and messages are deleted atomically
        let tx = self.conn.unchecked_transaction()?;

        // Delete all messages for this session first (foreign key constraint)
        tx.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            params![session_id],
        )?;

        // Delete the session itself
        let rows_affected =
            tx.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;

        // Commit the transaction
        tx.commit()?;

        // Check if session was actually deleted
        if rows_affected == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        Ok(())
    }
}

fn row_to_session_entity() -> fn(&Row) -> Result<SessionEntity> {
    |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let expires_at_str: String = row.get(2)?;
        let current: i32 = row.get(3)?;
        let expires_at = NaiveDateTime::parse_from_str(&expires_at_str, DATE_TIME_FORMAT)
            .expect("Invalid DateTime format");

        Ok(SessionEntity::new(id, name, expires_at, current))
    }
}
