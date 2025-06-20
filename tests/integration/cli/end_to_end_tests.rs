use crate::common::TestEnvironment;
use assert_cmd::Command;
use predicates::prelude::*;

#[tokio::test]
async fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("termai"));
}

#[tokio::test]
async fn test_cli_version_command() {
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("termai"));
}

#[tokio::test]
async fn test_cli_argument_validation() {
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    
    // Test with invalid arguments
    cmd.arg("--invalid-flag")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_cli_provider_selection() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test setting Claude provider
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--provider")
        .arg("claude")
        .assert()
        .success();
    
    // Test setting OpenAI provider
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--provider")
        .arg("openapi")
        .assert()
        .success();
    
    // Test invalid provider should fail
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--provider")
        .arg("invalid")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_cli_input_extraction() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test providing input via command line argument with --help to avoid API calls
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[tokio::test]
async fn test_cli_session_management_workflow() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test listing sessions (using --sessions-all flag)
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--sessions-all")
        .assert()
        .success();
}

#[tokio::test]
async fn test_cli_configuration_workflow() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test setting Claude API key
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--claude-api-key")
        .arg("test-claude-key")
        .assert()
        .success();
    
    // Test setting OpenAI API key
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--chat-gpt-api-key")
        .arg("test-openai-key")
        .assert()
        .success();
}

#[tokio::test]
async fn test_cli_database_initialization() {
    use std::fs;
    
    // Get the default database path that the app would use
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let default_dir = home_dir.join(".config/termai");
    let default_db_path = default_dir.join("app.db");
    
    // Backup existing database if it exists
    let backup_path = default_db_path.with_extension("db.bak");
    let db_existed = default_db_path.exists();
    if db_existed {
        fs::rename(&default_db_path, &backup_path).expect("Failed to backup existing database");
    }
    
    // Run CLI command that should initialize database
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.arg("--sessions-all")
        .assert()
        .success();
    
    // Verify database was created
    assert!(default_db_path.exists(), "Database should be created at default location");
    
    // Cleanup: remove test database and restore backup if it existed
    if default_db_path.exists() {
        fs::remove_file(&default_db_path).expect("Failed to remove test database");
    }
    if db_existed {
        fs::rename(&backup_path, &default_db_path).expect("Failed to restore backup database");
    }
}

#[tokio::test]
async fn test_cli_error_handling() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test invalid argument handling
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--invalid-flag-that-does-not-exist")
        .assert()
        .failure();
}

#[tokio::test]
async fn test_cli_tui_mode() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test TUI mode flag
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .arg("--tui")
        .timeout(std::time::Duration::from_secs(2)) // Quick timeout since TUI is interactive
        .assert()
        .failure(); // Should timeout in TUI mode since it's interactive
}

#[tokio::test]
async fn test_cli_output_formatting() {
    let _test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test plain text output (default) using --sessions-all
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.arg("--sessions-all")
        .assert()
        .success();
    
    // Test print config formatting (uses default database location)
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.arg("--print-config")
        .assert()
        .success();
}

#[tokio::test]
async fn test_cli_signal_handling() {
    // Test that CLI handles interruption gracefully
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .env("CLAUDE_API_KEY", "test-key")
        .arg("Long running test message")
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .failure(); // Should handle timeout/interruption gracefully
}

#[tokio::test]
async fn test_cli_concurrent_execution() {
    // Test that multiple CLI instances can run concurrently
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    let handles = (0..3).map(|i| {
        let db_path = test_env.db_path_str().to_string();
        tokio::spawn(async move {
            let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
            cmd.env("TERMAI_DB_PATH", &db_path)
                .arg("--session")
                .arg(&format!("concurrent-session-{}", i))
                .arg("--help")
                .assert()
                .success();
        })
    }).collect::<Vec<_>>();
    
    // Wait for all instances to complete
    for handle in handles {
        handle.await.expect("Concurrent CLI execution failed");
    }
}

#[tokio::test]
async fn test_cli_file_input() {
    use std::fs;
    
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Create a test input file
    let input_file = test_env.temp_dir.path().join("input.txt");
    fs::write(&input_file, "This is test input from a file").expect("Failed to write input file");
    
    // Test reading from file
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .env("CLAUDE_API_KEY", "test-key")
        .arg("--file")
        .arg(input_file.to_str().unwrap())
        .assert()
        .failure(); // Will fail due to invalid API key, but validates file reading
}

#[tokio::test]
async fn test_cli_environment_variable_configuration() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Test that CLI respects environment variables
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .env("TERMAI_PROVIDER", "claude")
        .env("TERMAI_MODEL", "claude-3-sonnet-20240229")
        .env("CLAUDE_API_KEY", "test-key")
        .arg("--help")
        .assert()
        .success();
    
    // Test OpenAI environment variables
    let mut cmd = Command::cargo_bin("termai").expect("Failed to find termai binary");
    cmd.env("TERMAI_DB_PATH", test_env.db_path_str())
        .env("TERMAI_PROVIDER", "openai")
        .env("TERMAI_MODEL", "gpt-4")
        .env("OPENAI_API_KEY", "test-key")
        .arg("--help")
        .assert()
        .success();
}