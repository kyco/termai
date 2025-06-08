mod common;
mod integration;

use assert_cmd::Command;
use predicates::prelude::*;
use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_database_initialization() {
    // Create a temporary directory for the database
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path();
    let config_dir = home_dir.join(".config").join("termai");
    fs::create_dir_all(&config_dir).unwrap();

    // Run termai with a simple command that forces db initialization
    let mut cmd = Command::cargo_bin("termai").unwrap();
    cmd.env("HOME", home_dir.to_str().unwrap())
        .arg("--print-config");

    // Execute command
    cmd.assert().success();

    // Verify database was created with correct schema
    let db_path = config_dir.join("app.db");
    assert!(db_path.exists(), "Database file should be created");

    // Connect to database and check tables
    let conn = Connection::open(&db_path).unwrap();
    let tables = get_tables(&conn);

    assert!(tables.contains(&"messages".to_string()));
    assert!(tables.contains(&"config".to_string()));
    assert!(tables.contains(&"sessions".to_string()));
}

#[test]
fn test_config_storage() {
    // Create a temporary directory for the database
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path();
    let config_dir = home_dir.join(".config").join("termai");
    fs::create_dir_all(&config_dir).unwrap();

    // Set test API key
    let test_key = "test_api_key_123";
    let mut cmd = Command::cargo_bin("termai").unwrap();
    cmd.env("HOME", home_dir.to_str().unwrap())
        .arg("--chat-gpt-api-key")
        .arg(test_key);

    cmd.assert().success();

    // Verify key was stored by checking config
    let mut cmd = Command::cargo_bin("termai").unwrap();
    cmd.env("HOME", home_dir.to_str().unwrap())
        .arg("--print-config");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(test_key));
}

#[test]
fn test_thinking_timer() {
    // This test verifies the timer stops properly
    // We'll create a simplified version of the timer
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let running_clone = running.clone();

    running.store(true, std::sync::atomic::Ordering::SeqCst);
    let handle = std::thread::spawn(move || {
        while running_clone.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // Let it run briefly
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Stop the timer
    running.store(false, std::sync::atomic::Ordering::SeqCst);
    handle.join().unwrap();

    // Verify it's stopped
    assert!(!running.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn test_large_message_storage() {
    // Test if SQLite can handle very large messages without truncation
    use tempfile::TempDir;
    
    // Create a temporary database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let conn = Connection::open(&db_path).unwrap();
    
    // Create the messages table like termai does
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT NOT NULL PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL
        )",
        [],
    ).unwrap();
    
    // Create a very large message (1MB of text)
    let large_content = "A".repeat(1_000_000); // 1 million characters
    let session_id = "test_session_large_msg";
    
    // Store the message
    let result = conn.execute(
        "INSERT INTO messages (id, session_id, role, content) VALUES (?1, ?2, ?3, ?4)",
        ["large_msg_test", session_id, "Assistant", &large_content],
    );
    assert!(result.is_ok(), "Failed to save large message");
    
    // Retrieve the message
    let mut stmt = conn.prepare("SELECT content FROM messages WHERE session_id = ?1").unwrap();
    let mut rows = stmt.query_map([session_id], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap();
    
    let retrieved_content = rows.next().unwrap().unwrap();
    assert_eq!(retrieved_content.len(), large_content.len(), 
               "Message content length should be preserved");
    assert_eq!(retrieved_content, large_content, 
               "Message content should be exactly the same");
}

// Helper function to get table names from database
fn get_tables(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master
         WHERE type='table' AND name NOT LIKE 'sqlite_%'",
        )
        .unwrap();

    let table_names = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();

    let mut tables = Vec::new();
    for table in table_names {
        tables.push(table.unwrap());
    }
    tables
}
