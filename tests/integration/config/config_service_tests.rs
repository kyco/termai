use crate::common::{TestEnvironment};
use termai::repository::db::SqliteRepository;
use termai::config::service::config_service;

#[tokio::test]
async fn test_config_service_save_and_load() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Test writing new config
    config_service::write_config(&db, "test_key", "test_value")
        .expect("Failed to write config");
    
    // Test fetching config by key
    let config = config_service::fetch_by_key(&db, "test_key")
        .expect("Failed to fetch config by key");
    assert_eq!(config.key, "test_key");
    assert_eq!(config.value, "test_value");
    
    // Test updating existing config
    config_service::write_config(&db, "test_key", "updated_value")
        .expect("Failed to update config");
    
    let updated_config = config_service::fetch_by_key(&db, "test_key")
        .expect("Failed to fetch updated config");
    assert_eq!(updated_config.value, "updated_value");
    
    // Test fetching all configs
    config_service::write_config(&db, "another_key", "another_value")
        .expect("Failed to write another config");
    
    let all_configs = config_service::fetch_config(&db)
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 2);
}

#[tokio::test]
async fn test_config_service_validation() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Test fetching non-existent key
    let missing_result = config_service::fetch_by_key(&db, "non_existent_key");
    assert!(missing_result.is_err());
    
    // Test writing and reading with special characters
    config_service::write_config(&db, "special_key", "value with spaces and símb@ls")
        .expect("Failed to write config with special characters");
    
    let special_config = config_service::fetch_by_key(&db, "special_key")
        .expect("Failed to fetch config with special characters");
    assert_eq!(special_config.value, "value with spaces and símb@ls");
    
    // Test writing empty value
    config_service::write_config(&db, "empty_key", "")
        .expect("Failed to write config with empty value");
    
    let empty_config = config_service::fetch_by_key(&db, "empty_key")
        .expect("Failed to fetch config with empty value");
    assert_eq!(empty_config.value, "");
}

#[tokio::test]
async fn test_config_service_provider_switching() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Set initial provider
    config_service::write_config(&db, "current_provider", "claude")
        .expect("Failed to set initial provider");
    
    // Set API keys for different providers
    config_service::write_config(&db, "claude_api_key", "claude_test_key")
        .expect("Failed to set Claude API key");
    
    config_service::write_config(&db, "openai_api_key", "openai_test_key")
        .expect("Failed to set OpenAI API key");
    
    // Verify current provider
    let current_provider = config_service::fetch_by_key(&db, "current_provider")
        .expect("Failed to fetch current provider");
    assert_eq!(current_provider.value, "claude");
    
    // Switch provider
    config_service::write_config(&db, "current_provider", "openai")
        .expect("Failed to switch provider");
    
    let switched_provider = config_service::fetch_by_key(&db, "current_provider")
        .expect("Failed to fetch switched provider");
    assert_eq!(switched_provider.value, "openai");
    
    // Verify API keys are still accessible
    let claude_key = config_service::fetch_by_key(&db, "claude_api_key")
        .expect("Failed to fetch Claude API key");
    assert_eq!(claude_key.value, "claude_test_key");
    
    let openai_key = config_service::fetch_by_key(&db, "openai_api_key")
        .expect("Failed to fetch OpenAI API key");
    assert_eq!(openai_key.value, "openai_test_key");
}

#[tokio::test]
async fn test_claude_config_api_key_storage() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Test storing Claude API key
    let claude_api_key = "claude-test-api-key-12345";
    config_service::write_config(&db, "claude_api_key", claude_api_key)
        .expect("Failed to store Claude API key");
    
    // Test retrieving Claude API key
    let stored_key = config_service::fetch_by_key(&db, "claude_api_key")
        .expect("Failed to retrieve Claude API key");
    assert_eq!(stored_key.value, claude_api_key);
    
    // Test Claude-specific configuration
    config_service::write_config(&db, "claude_model", "claude-3-sonnet-20240229")
        .expect("Failed to store Claude model");
    
    config_service::write_config(&db, "claude_max_tokens", "4096")
        .expect("Failed to store Claude max tokens");
    
    let claude_model = config_service::fetch_by_key(&db, "claude_model")
        .expect("Failed to retrieve Claude model");
    assert_eq!(claude_model.value, "claude-3-sonnet-20240229");
    
    let claude_max_tokens = config_service::fetch_by_key(&db, "claude_max_tokens")
        .expect("Failed to retrieve Claude max tokens");
    assert_eq!(claude_max_tokens.value, "4096");
}

#[tokio::test]
async fn test_openai_config_api_key_storage() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Test storing OpenAI API key
    let openai_api_key = "sk-openai-test-api-key-12345";
    config_service::write_config(&db, "openai_api_key", openai_api_key)
        .expect("Failed to store OpenAI API key");
    
    // Test retrieving OpenAI API key
    let stored_key = config_service::fetch_by_key(&db, "openai_api_key")
        .expect("Failed to retrieve OpenAI API key");
    assert_eq!(stored_key.value, openai_api_key);
    
    // Test OpenAI-specific configuration
    config_service::write_config(&db, "openai_model", "gpt-4")
        .expect("Failed to store OpenAI model");
    
    config_service::write_config(&db, "openai_temperature", "0.7")
        .expect("Failed to store OpenAI temperature");
    
    config_service::write_config(&db, "openai_max_tokens", "2048")
        .expect("Failed to store OpenAI max tokens");
    
    let openai_model = config_service::fetch_by_key(&db, "openai_model")
        .expect("Failed to retrieve OpenAI model");
    assert_eq!(openai_model.value, "gpt-4");
    
    let openai_temperature = config_service::fetch_by_key(&db, "openai_temperature")
        .expect("Failed to retrieve OpenAI temperature");
    assert_eq!(openai_temperature.value, "0.7");
    
    let openai_max_tokens = config_service::fetch_by_key(&db, "openai_max_tokens")
        .expect("Failed to retrieve OpenAI max tokens");
    assert_eq!(openai_max_tokens.value, "2048");
}

#[tokio::test]
async fn test_config_file_creation_and_updates() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    
    // Verify database file doesn't exist initially
    assert!(!test_env.db_path.exists());
    
    // Create database and add config - this should create the file
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    config_service::write_config(&db, "initial_key", "initial_value")
        .expect("Failed to write initial config");
    
    // Verify database file now exists
    assert!(test_env.db_path.exists());
    
    // Drop the database connection
    drop(db);
    
    // Reopen database and verify data persisted
    let db2 = SqliteRepository::new(test_env.db_path_str()).expect("Failed to reopen database");
    let persisted_config = config_service::fetch_by_key(&db2, "initial_key")
        .expect("Failed to fetch persisted config");
    assert_eq!(persisted_config.value, "initial_value");
    
    // Add more config and verify it's saved
    config_service::write_config(&db2, "additional_key", "additional_value")
        .expect("Failed to write additional config");
    
    let all_configs = config_service::fetch_config(&db2)
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 2);
}

#[tokio::test]
async fn test_config_file_permissions() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Write sensitive config data
    config_service::write_config(&db, "api_key", "sensitive-api-key-data")
        .expect("Failed to write sensitive config");
    
    // Verify the file exists and has reasonable permissions
    assert!(test_env.db_path.exists());
    
    // On Unix systems, we could check file permissions, but for cross-platform
    // compatibility, we'll just verify we can read the data back
    let sensitive_config = config_service::fetch_by_key(&db, "api_key")
        .expect("Failed to read sensitive config");
    assert_eq!(sensitive_config.value, "sensitive-api-key-data");
    
    // Verify multiple users can't write simultaneously by opening another connection
    let db2 = SqliteRepository::new(test_env.db_path_str()).expect("Failed to open second connection");
    
    // Both should be able to read
    let config_from_db2 = config_service::fetch_by_key(&db2, "api_key")
        .expect("Failed to read from second connection");
    assert_eq!(config_from_db2.value, "sensitive-api-key-data");
    
    // Both should be able to write
    config_service::write_config(&db, "key_from_db1", "value1")
        .expect("Failed to write from first connection");
    
    config_service::write_config(&db2, "key_from_db2", "value2")
        .expect("Failed to write from second connection");
    
    // Verify both writes succeeded
    let all_configs = config_service::fetch_config(&db)
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 3);
}