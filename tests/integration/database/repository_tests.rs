use crate::common::TestEnvironment;
use termai::repository::db::SqliteRepository;
use termai::config::repository::ConfigRepository;
use termai::session::repository::{SessionRepository, MessageRepository};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[tokio::test]
async fn test_config_repository_crud_operations() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Test adding config
    db.add_config("test_key", "test_value")
        .expect("Failed to add config");
    
    // Test fetching config by key
    let config_entity = db.fetch_by_key("test_key")
        .expect("Failed to fetch config by key");
    assert_eq!(config_entity.key, "test_key");
    assert_eq!(config_entity.value, "test_value");
    
    // Test updating config
    db.update_config(config_entity.id.unwrap(), "test_key", "updated_value")
        .expect("Failed to update config");
    
    let updated_config = db.fetch_by_key("test_key")
        .expect("Failed to fetch updated config");
    assert_eq!(updated_config.value, "updated_value");
    
    // Test fetching all configs
    db.add_config("another_key", "another_value")
        .expect("Failed to add another config");
    
    let all_configs = db.fetch_all_configs()
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 2);
    
    // Test non-existent key
    let missing_result = db.fetch_by_key("non_existent_key");
    assert!(missing_result.is_err());
}

#[tokio::test]
async fn test_session_repository_crud_operations() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    let session_id = Uuid::new_v4().to_string();
    let session_name = "Test Session";
    let expires_at = DateTime::from_timestamp(Utc::now().timestamp() + 3600, 0).unwrap().naive_utc();
    
    // Test adding session
    db.add_session(&session_id, session_name, expires_at, false)
        .expect("Failed to add session");
    
    // Test fetching session by ID
    let retrieved_session = db.fetch_session_by_id(&session_id)
        .expect("Failed to fetch session by ID");
    assert_eq!(retrieved_session.id, session_id);
    assert_eq!(retrieved_session.name, session_name);
    
    // Test fetching session by name
    let retrieved_by_name = db.fetch_session_by_name(session_name)
        .expect("Failed to fetch session by name");
    assert_eq!(retrieved_by_name.id, session_id);
    
    // Test fetching all sessions
    let another_session_id = Uuid::new_v4().to_string();
    db.add_session(&another_session_id, "Another Session", expires_at, false)
        .expect("Failed to add another session");
    
    let all_sessions = db.fetch_all_sessions()
        .expect("Failed to fetch all sessions");
    assert_eq!(all_sessions.len(), 2);
    
    // Test updating session
    let new_expires_at = DateTime::from_timestamp(Utc::now().timestamp() + 7200, 0).unwrap().naive_utc();
    db.update_session(&session_id, "Updated Session", new_expires_at, true)
        .expect("Failed to update session");
    
    let updated_session = db.fetch_session_by_id(&session_id)
        .expect("Failed to fetch updated session");
    assert_eq!(updated_session.name, "Updated Session");
    assert_eq!(updated_session.current, 1);
    
    // Test fetching current session
    let current_session = db.fetch_current_session()
        .expect("Failed to fetch current session");
    assert_eq!(current_session.id, session_id);
    
    // Test removing current from all
    db.remove_current_from_all()
        .expect("Failed to remove current from all");
    
    let no_current_result = db.fetch_current_session();
    assert!(no_current_result.is_err());
}

#[tokio::test]
async fn test_message_repository_crud_operations() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create a session first
    let session_id = Uuid::new_v4().to_string();
    let expires_at = DateTime::from_timestamp(Utc::now().timestamp() + 3600, 0).unwrap().naive_utc();
    db.add_session(&session_id, "Test Session", expires_at, false)
        .expect("Failed to create test session");
    
    // Create test message entity
    let message_entity = termai::session::entity::message_entity::MessageEntity::new(
        Uuid::new_v4().to_string(),
        session_id.clone(),
        "user".to_string(),
        "Test message content".to_string(),
    );
    
    // Test adding message
    db.add_message_to_session(&message_entity)
        .expect("Failed to add message");
    
    // Test fetching messages for session
    let session_messages = db.fetch_messages_for_session(&session_id)
        .expect("Failed to fetch messages for session");
    assert_eq!(session_messages.len(), 1);
    assert_eq!(session_messages[0].id, message_entity.id);
    assert_eq!(session_messages[0].content, message_entity.content);
    assert_eq!(session_messages[0].role, "user");
    
    // Test adding another message
    let message_entity2 = termai::session::entity::message_entity::MessageEntity::new(
        Uuid::new_v4().to_string(),
        session_id.clone(),
        "assistant".to_string(),
        "Assistant response".to_string(),
    );
    
    db.add_message_to_session(&message_entity2)
        .expect("Failed to add second message");
    
    let all_session_messages = db.fetch_messages_for_session(&session_id)
        .expect("Failed to fetch all session messages");
    assert_eq!(all_session_messages.len(), 2);
    
    // Test fetching all messages
    let all_messages = db.fetch_all_messages()
        .expect("Failed to fetch all messages");
    assert_eq!(all_messages.len(), 2);
}

#[tokio::test]
async fn test_database_duplicate_key_handling() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Add a valid config
    db.add_config("valid_key", "valid_value")
        .expect("Failed to add valid config");
    
    // Verify the valid config was saved
    let config = db.fetch_by_key("valid_key")
        .expect("Failed to fetch valid config");
    assert_eq!(config.value, "valid_value");
    
    // Test that the system allows duplicate keys (current schema behavior)
    db.add_config("valid_key", "duplicate_value")
        .expect("Failed to add duplicate config (allowed in current schema)");
    
    // Verify that when multiple configs with same key exist, fetch_by_key returns the first one
    let retrieved_config = db.fetch_by_key("valid_key")
        .expect("Failed to fetch config");
    assert_eq!(retrieved_config.value, "valid_value");
    
    // Verify the database is still functional
    db.add_config("another_key", "another_value")
        .expect("Failed to add another config");
    
    let all_configs = db.fetch_all_configs()
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 3); // valid_key (2 entries) + another_key
}

#[tokio::test]
async fn test_concurrent_database_access() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db_path = test_env.db_path_str().to_string();
    
    // Spawn concurrent operations using separate database connections
    let db_path1 = db_path.clone();
    let handle1 = tokio::spawn(async move {
        let db1 = SqliteRepository::new(&db_path1).expect("Failed to create db1");
        for i in 0..5 {
            db1.add_config(&format!("key1_{}", i), &format!("value1_{}", i))
                .expect("Failed to add from db1");
        }
    });
    
    let db_path2 = db_path.clone();
    let handle2 = tokio::spawn(async move {
        let db2 = SqliteRepository::new(&db_path2).expect("Failed to create db2");
        for i in 0..5 {
            db2.add_config(&format!("key2_{}", i), &format!("value2_{}", i))
                .expect("Failed to add from db2");
        }
    });
    
    // Wait for both operations to complete
    handle1.await.expect("Handle 1 failed");
    handle2.await.expect("Handle 2 failed");
    
    // Verify all data was saved correctly
    let verify_db = SqliteRepository::new(&db_path).expect("Failed to create verify db");
    
    for i in 0..5 {
        let value1 = verify_db.fetch_by_key(&format!("key1_{}", i))
            .expect("Failed to fetch key1");
        assert_eq!(value1.value, format!("value1_{}", i));
        
        let value2 = verify_db.fetch_by_key(&format!("key2_{}", i))
            .expect("Failed to fetch key2");
        assert_eq!(value2.value, format!("value2_{}", i));
    }
    
    // Verify total count
    let all_configs = verify_db.fetch_all_configs()
        .expect("Failed to fetch all configs");
    assert_eq!(all_configs.len(), 10);
}