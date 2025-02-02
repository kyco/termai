use rusqlite::{Connection, Result};

pub struct SqliteRepository {
    pub(crate) conn: Connection,
}

impl SqliteRepository {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        create_table_messages(&conn)?;
        create_table_config(&conn)?;
        create_table_sessions(&conn)?;
        sessions_add_current_column(&conn)?;
        sessions_rename_column_key_to_name(&conn)?;
        Ok(Self { conn })
    }
}

fn create_table_messages(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL
            )",
        [],
    )?;
    Ok(())
}

fn create_table_config(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
                id INTEGER PRIMARY KEY,
                key TEXT NOT NULL,
                value TEXT NOT NULL
            )",
        [],
    )?;
    Ok(())
}

fn create_table_sessions(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT NOT NULL,
                name TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                current INTEGER NOT NULL DEFAULT 0
            )",
        [],
    )?;
    Ok(())
}

fn sessions_add_current_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
    let mut has_current = false;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        if col? == "current" {
            has_current = true;
            break;
        }
    }
    if !has_current {
        conn.execute(
            "ALTER TABLE sessions ADD COLUMN current INTEGER NOT NULL
                        DEFAULT 0",
            [],
        )?;
    }
    drop(stmt);
    Ok(())
}

fn sessions_rename_column_key_to_name(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(sessions)")?;
    let mut has_key = false;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        if col? == "key" {
            has_key = true;
            break;
        }
    }
    drop(stmt);
    if has_key {
        conn.execute("ALTER TABLE sessions RENAME COLUMN key TO name", [])?;
    }
    Ok(())
}
