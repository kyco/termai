use rusqlite::Connection;
use std::path::Path;
use anyhow::Result;

pub fn create_test_database(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    initialize_test_schema(&conn)?;
    Ok(conn)
}

fn initialize_test_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    
    Ok(())
}

pub fn create_in_memory_db() -> Result<Connection> {
    let conn = Connection::open(":memory:")?;
    initialize_test_schema(&conn)?;
    Ok(conn)
}