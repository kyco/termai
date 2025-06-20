use tempfile::TempDir;

#[test]
fn test_simple_database_creation() {
    // This test just verifies we can create a temporary database
    // without relying on the complex termai crate structure
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    
    // Create a simple SQLite database
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to create database");
    
    // Create a simple table
    conn.execute(
        "CREATE TABLE test_table (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
        [],
    ).expect("Failed to create table");
    
    // Insert test data
    conn.execute(
        "INSERT INTO test_table (name) VALUES (?1)",
        ["test_name"],
    ).expect("Failed to insert data");
    
    // Verify data was inserted
    let mut stmt = conn.prepare("SELECT name FROM test_table WHERE id = 1").expect("Failed to prepare statement");
    let name: String = stmt.query_row([], |row| row.get(0)).expect("Failed to query data");
    
    assert_eq!(name, "test_name");
    
    // Verify database file exists
    assert!(db_path.exists());
}

#[test] 
fn test_concurrent_database_access() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("concurrent_test.db");
    
    // Create database and schema
    {
        let conn = rusqlite::Connection::open(&db_path).expect("Failed to create database");
        conn.execute(
            "CREATE TABLE concurrent_test (
                id INTEGER PRIMARY KEY,
                thread_id INTEGER,
                value TEXT
            )",
            [],
        ).expect("Failed to create table");
    }
    
    // Test concurrent access
    let db_path_str = db_path.to_str().unwrap().to_string();
    let handles: Vec<_> = (0..3).map(|i| {
        let path = db_path_str.clone();
        std::thread::spawn(move || {
            let conn = rusqlite::Connection::open(&path).expect("Failed to open database");
            for j in 0..5 {
                conn.execute(
                    "INSERT INTO concurrent_test (thread_id, value) VALUES (?1, ?2)",
                    [i, j],
                ).expect("Failed to insert data");
            }
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread failed");
    }
    
    // Verify all data was inserted
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open database");
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM concurrent_test").expect("Failed to prepare count");
    let count: i64 = stmt.query_row([], |row| row.get(0)).expect("Failed to get count");
    
    assert_eq!(count, 15); // 3 threads * 5 inserts each
}

#[test]
fn test_large_text_storage() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("large_text_test.db");
    
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to create database");
    
    // Create table for large text
    conn.execute(
        "CREATE TABLE large_text_test (
            id INTEGER PRIMARY KEY,
            content TEXT
        )",
        [],
    ).expect("Failed to create table");
    
    // Create large text content (100KB)
    let large_content = "A".repeat(100_000);
    
    // Insert large content
    conn.execute(
        "INSERT INTO large_text_test (content) VALUES (?1)",
        [&large_content],
    ).expect("Failed to insert large content");
    
    // Retrieve and verify
    let mut stmt = conn.prepare("SELECT content FROM large_text_test WHERE id = 1").expect("Failed to prepare statement");
    let retrieved_content: String = stmt.query_row([], |row| row.get(0)).expect("Failed to retrieve content");
    
    assert_eq!(retrieved_content.len(), large_content.len());
    assert_eq!(retrieved_content, large_content);
}

#[test]
fn test_unicode_text_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("unicode_test.db");
    
    let conn = rusqlite::Connection::open(&db_path).expect("Failed to create database");
    
    conn.execute(
        "CREATE TABLE unicode_test (
            id INTEGER PRIMARY KEY,
            content TEXT
        )",
        [],
    ).expect("Failed to create table");
    
    // Test various unicode content
    let test_cases = vec![
        "Hello 🌍 World! 🚀",
        "Émojis and spëcial characters",
        "日本語 Japanese text",
        "中文 Chinese text", 
        "العربية Arabic text",
        "русский Russian text",
        "Math: ∑∞∈∅∩∪⊆⊇",
    ];
    
    // Insert all test cases
    for (i, test_case) in test_cases.iter().enumerate() {
        conn.execute(
            "INSERT INTO unicode_test (id, content) VALUES (?1, ?2)",
            rusqlite::params![i as i64 + 1, test_case],
        ).expect("Failed to insert unicode content");
    }
    
    // Retrieve and verify
    for (i, expected) in test_cases.iter().enumerate() {
        let mut stmt = conn.prepare("SELECT content FROM unicode_test WHERE id = ?1").expect("Failed to prepare statement");
        let retrieved: String = stmt.query_row([i as i64 + 1], |row| row.get(0)).expect("Failed to retrieve unicode content");
        assert_eq!(&retrieved, expected);
    }
}

#[test]
fn test_json_serialization() {
    use serde_json::{json, Value};
    
    // Test JSON serialization/deserialization for potential API responses
    let test_response = json!({
        "id": "test_123",
        "type": "message",
        "role": "assistant", 
        "content": [
            {
                "type": "text",
                "text": "Hello! This is a test response."
            }
        ],
        "model": "test-model",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 25
        }
    });
    
    // Serialize to string
    let json_string = serde_json::to_string(&test_response).expect("Failed to serialize JSON");
    
    // Deserialize back
    let parsed: Value = serde_json::from_str(&json_string).expect("Failed to parse JSON");
    
    // Verify structure
    assert_eq!(parsed["id"], "test_123");
    assert_eq!(parsed["role"], "assistant");
    assert_eq!(parsed["usage"]["input_tokens"], 10);
    assert_eq!(parsed["usage"]["output_tokens"], 25);
}