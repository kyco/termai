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
        create_table_conversation_branches(&conn)?;
        create_table_branch_messages(&conn)?;
        create_table_branch_metadata(&conn)?;
        migrate_messages_id_column(&conn)?;
        messages_add_session_id_column(&conn)?;
        messages_add_role_column(&conn)?;
        messages_add_type_columns(&conn)?;
        sessions_add_current_column(&conn)?;
        sessions_rename_column_key_to_name(&conn)?;
        if cfg!(debug_assertions) {
            debug_print_tables(&conn)?;
        }
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

fn messages_add_type_columns(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(messages)")?;
    let mut has_message_type = false;
    let mut has_compaction_metadata = false;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for col in rows {
        let col_name = col?;
        if col_name == "message_type" {
            has_message_type = true;
        }
        if col_name == "compaction_metadata" {
            has_compaction_metadata = true;
        }
    }
    drop(stmt);

    if !has_message_type {
        conn.execute(
            "ALTER TABLE messages ADD COLUMN message_type TEXT NOT NULL DEFAULT 'standard'",
            [],
        )?;
    }
    if !has_compaction_metadata {
        conn.execute(
            "ALTER TABLE messages ADD COLUMN compaction_metadata TEXT",
            [],
        )?;
    }
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

fn create_table_conversation_branches(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS conversation_branches (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            parent_branch_id TEXT,
            branch_name TEXT,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_activity DATETIME DEFAULT CURRENT_TIMESTAMP,
            status TEXT DEFAULT 'active',
            FOREIGN KEY (session_id) REFERENCES sessions (id),
            FOREIGN KEY (parent_branch_id) REFERENCES conversation_branches (id)
        )",
        [],
    )?;
    Ok(())
}

fn create_table_branch_messages(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS branch_messages (
            id TEXT PRIMARY KEY,
            branch_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            FOREIGN KEY (branch_id) REFERENCES conversation_branches (id),
            FOREIGN KEY (message_id) REFERENCES messages (id)
        )",
        [],
    )?;
    Ok(())
}

fn create_table_branch_metadata(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS branch_metadata (
            branch_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (branch_id, key),
            FOREIGN KEY (branch_id) REFERENCES conversation_branches (id)
        )",
        [],
    )?;
    Ok(())
}

fn debug_print_tables(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master
         WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let table_names = stmt.query_map([], |row| row.get::<_, String>(0))?;

    for table in table_names {
        let table_name = table?;
        println!("Table: {}", table_name);

        let mut info = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns = info.query_map([], |row| {
            let col_name: String = row.get(1)?;
            let col_type: String = row.get(2)?;
            let notnull: i64 = row.get(3)?;
            let _default: Option<String> = row.get(4)?;
            let pk: i64 = row.get(5)?;
            Ok(format!(
                "  {} {}{} {}",
                col_name,
                col_type,
                if notnull == 1 { " NOT NULL" } else { "" },
                if pk == 1 { "[PK]" } else { "" }
            ))
        })?;

        for col in columns {
            println!("{}", col?);
        }
    }
    Ok(())
}
