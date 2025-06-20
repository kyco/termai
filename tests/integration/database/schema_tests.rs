use crate::common::{TestEnvironment, test_db::{create_test_database, create_in_memory_db}};
use rusqlite::Connection;

#[tokio::test]
async fn test_database_schema_creation() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let conn = create_test_database(&test_env.db_path).expect("Failed to create test database");
    
    // Check if sessions table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='sessions'").unwrap();
    let rows: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    assert!(rows.is_ok());
    let tables = rows.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0], "sessions");
    
    // Check if messages table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='messages'").unwrap();
    let rows: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    assert!(rows.is_ok());
    let tables = rows.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0], "messages");
    
    // Check if config table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='config'").unwrap();
    let rows: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    assert!(rows.is_ok());
    let tables = rows.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0], "config");
}

#[tokio::test]
async fn test_database_migration_from_empty() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Verify database file doesn't exist initially
    assert!(!test_env.db_path.exists());
    
    // Create database - this should create the file and initialize schema
    let _conn = create_test_database(&test_env.db_path).expect("Failed to create test database");
    
    // Verify database file now exists
    assert!(test_env.db_path.exists());
    
    // Verify we can open it again
    let conn2 = Connection::open(&test_env.db_path).expect("Failed to reopen database");
    
    // Verify all tables exist in reopened database
    let mut stmt = conn2.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap();
    let table_names: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    let tables = table_names.unwrap();
    assert_eq!(tables, vec!["config", "messages", "sessions"]);
}

#[tokio::test]
async fn test_database_connection_lifecycle() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test creating connection
    let conn1 = create_test_database(&test_env.db_path).expect("Failed to create database");
    
    // Test basic operation
    conn1.execute("INSERT INTO config (key, value) VALUES ('test_key', 'test_value')", [])
        .expect("Failed to insert test data");
    
    // Drop first connection
    drop(conn1);
    
    // Test reopening connection
    let conn2 = Connection::open(&test_env.db_path).expect("Failed to reopen database");
    
    // Verify data persisted
    let mut stmt = conn2.prepare("SELECT value FROM config WHERE key = 'test_key'").unwrap();
    let values: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    let results = values.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "test_value");
}

#[tokio::test]
async fn test_in_memory_database() {
    let conn = create_in_memory_db().expect("Failed to create in-memory database");
    
    // Test that schema is properly initialized
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap();
    let table_names: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    let tables = table_names.unwrap();
    assert_eq!(tables, vec!["config", "messages", "sessions"]);
    
    // Test basic operations work
    conn.execute("INSERT INTO config (key, value) VALUES ('memory_test', 'works')", [])
        .expect("Failed to insert into in-memory database");
    
    let mut stmt = conn.prepare("SELECT value FROM config WHERE key = 'memory_test'").unwrap();
    let values: Result<Vec<String>, _> = stmt.query_map([], |row| {
        Ok(row.get::<_, String>(0)?)
    }).unwrap().collect();
    
    let results = values.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], "works");
}