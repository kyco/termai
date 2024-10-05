use crate::repository::Repository;
use rusqlite::{params, Connection, Result};

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }
}

impl Repository for SqliteRepository {
    type Error = rusqlite::Error;

    fn get_messages(&self) -> Result<Vec<String>, Self::Error> {
        let mut stmt = self.conn.prepare("SELECT content FROM messages")?;
        let rows = stmt.query_map([], |row| row.get(0))?;

        let mut messages = Vec::new();
        for msg in rows {
            messages.push(msg?);
        }
        Ok(messages)
    }

    fn add_message(&self, content: String) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO messages (content) VALUES (?1)",
            params![content],
        )?;
        Ok(())
    }
}
