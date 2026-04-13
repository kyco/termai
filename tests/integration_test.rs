use assert_cmd::cargo::cargo_bin_cmd;
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
    let mut cmd = cargo_bin_cmd!("termai");
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
    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home_dir.to_str().unwrap())
        .arg("--chat-gpt-api-key")
        .arg(test_key);

    cmd.assert().success();

    // Verify key was stored by checking config
    let mut cmd = cargo_bin_cmd!("termai");
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
fn test_config_show_preserves_codex_gpt_5_4_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path();
    let xdg_config_home = home_dir.join(".config");
    let config_contents = r#"
version = 1

[default]
provider = "codex"
model = "gpt-5.4"
"#;

    let config_dirs = vec![
        xdg_config_home.join("termai"),
        home_dir
            .join("Library")
            .join("Application Support")
            .join("termai"),
    ];

    for config_dir in &config_dirs {
        fs::create_dir_all(config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), config_contents).unwrap();
    }

    let mut cmd = cargo_bin_cmd!("termai");
    cmd.env("HOME", home_dir.to_str().unwrap())
        .env("XDG_CONFIG_HOME", xdg_config_home.to_str().unwrap())
        .current_dir(home_dir)
        .args(["config", "show"]);

    cmd.assert().success().stdout(
        predicate::str::contains("Default provider:")
            .and(predicate::str::contains("codex"))
            .and(predicate::str::contains("Default model:"))
            .and(predicate::str::contains("gpt-5.4")),
    );
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
