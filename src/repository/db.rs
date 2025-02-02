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
        migrate_messages_id_column(&conn)?;
        messages_add_session_id_column(&conn)?;
        messages_add_role_column(&conn)?;
        sessions_add_current_column(&conn)?;
        sessions_rename_column_key_to_name(&conn)?;
        Ok(Self { conn })
    }
}

fn create_table_messages(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
                id TEXT NOT NULL PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
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

fn migrate_messages_id_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
    let mut old_id_schema = false;
    let rows = stmt.query_map([], |row| {
        let col_name: String = row.get(1)?;
        let col_type: String = row.get(2)?;
        Ok((col_name, col_type))
    })?;
    for col in rows {
        let (name, col_type) = col?;
        if name == "id" && col_type.eq_ignore_ascii_case("INTEGER") {
            old_id_schema = true;
            break;
        }
    }
    drop(stmt);

    if old_id_schema {
        conn.execute("ALTER TABLE messages RENAME TO messages_old", [])?;
        create_table_messages(conn)?;
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content)
             SELECT CAST(id AS TEXT), session_id, role, content
             FROM messages_old",
            [],
        )?;
        conn.execute("DROP TABLE messages_old", [])?;
    }
    Ok(())
}

fn messages_add_session_id_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
    let mut has_session_id = false;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        if col? == "session_id" {
            has_session_id = true;
            break;
        }
    }
    if !has_session_id {
        conn.execute(
            "ALTER TABLE messages ADD COLUMN session_id TEXT NOT NULL",
            [],
        )?;
    }
    drop(stmt);
    Ok(())
}

fn messages_add_role_column(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
    let mut has_role = false;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        if col? == "role" {
            has_role = true;
            break;
        }
    }
    if !has_role {
        conn.execute("ALTER TABLE messages ADD COLUMN role TEXT NOT NULL", [])?;
    }
    drop(stmt);
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
            "ALTER TABLE sessions ADD COLUMN current INTEGER NOT NULL DEFAULT 0",
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
