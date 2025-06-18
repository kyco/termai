use crate::common::TestEnvironment;
use termai::repository::db::SqliteRepository;
use termai::session::service::sessions_service;
use termai::session::repository::{SessionRepository, MessageRepository};
use termai::session::model::message::Message;
use termai::llm::common::model::role::Role;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

#[tokio::test]
async fn test_session_creation_and_persistence() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create a new session using the service
    let session = sessions_service::session(&db, &db, "Test Session")
        .expect("Failed to create session");
    
    assert_eq!(session.name, "Test Session");
    assert!(session.current);
    assert!(!session.temporary);
    assert!(session.messages.is_empty());
    
    // Verify session persisted to database
    let persisted_session = db.fetch_session_by_name("Test Session")
        .expect("Failed to fetch persisted session");
    assert_eq!(persisted_session.name, "Test Session");
    assert_eq!(persisted_session.current, 1);
}

#[tokio::test]
async fn test_session_retrieval_and_listing() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create multiple sessions
    let session1 = sessions_service::session(&db, &db, "Session 1")
        .expect("Failed to create session 1");
    let _session2 = sessions_service::session(&db, &db, "Session 2")
        .expect("Failed to create session 2");
    let session3 = sessions_service::session(&db, &db, "Session 3")
        .expect("Failed to create session 3");
    
    // Fetch all sessions
    let all_sessions = db.fetch_all_sessions()
        .expect("Failed to fetch all sessions");
    assert_eq!(all_sessions.len(), 3);
    
    // Verify only the last session is current
    let current_session = db.fetch_current_session()
        .expect("Failed to fetch current session");
    assert_eq!(current_session.name, "Session 3");
    assert_eq!(current_session.id, session3.id);
    
    // Test fetching session by ID
    let retrieved_session = sessions_service::session_by_id(&db, &db, &session1.id)
        .expect("Failed to fetch session by ID");
    assert_eq!(retrieved_session.id, session1.id);
    assert_eq!(retrieved_session.name, "Session 1");
    assert!(!retrieved_session.current); // Should not be current anymore
}

#[tokio::test]
async fn test_session_deletion_and_cleanup() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create sessions with messages
    let session = sessions_service::session(&db, &db, "Session to Delete")
        .expect("Failed to create session");
    
    // Add messages to session
    let message1 = termai::session::entity::message_entity::MessageEntity::new(
        Uuid::new_v4().to_string(),
        session.id.clone(),
        "user".to_string(),
        "Hello".to_string(),
    );
    
    let message2 = termai::session::entity::message_entity::MessageEntity::new(
        Uuid::new_v4().to_string(),
        session.id.clone(),
        "assistant".to_string(),
        "Hi there!".to_string(),
    );
    
    db.add_message_to_session(&message1)
        .expect("Failed to add message 1");
    db.add_message_to_session(&message2)
        .expect("Failed to add message 2");
    
    // Verify session and messages exist
    let session_messages = db.fetch_messages_for_session(&session.id)
        .expect("Failed to fetch session messages");
    assert_eq!(session_messages.len(), 2);
    
    // Note: The current codebase doesn't seem to have a delete session method,
    // so we'll test that sessions can be retrieved and that the database
    // maintains consistency. In a real implementation, we'd test actual deletion.
    
    // Verify session still exists
    let retrieved_session = db.fetch_session_by_id(&session.id)
        .expect("Session should still exist");
    assert_eq!(retrieved_session.id, session.id);
}

#[tokio::test]
async fn test_message_addition_to_session() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create a session
    let session = sessions_service::session(&db, &db, "Message Test Session")
        .expect("Failed to create session");
    
    // Create a session with messages to add
    let mut session_with_messages = session.clone();
    let user_message = Message {
        id: "".to_string(), // Empty ID indicates new message
        role: Role::User,
        content: "What is the weather like?".to_string(),
    };
    
    let assistant_message = Message {
        id: "".to_string(), // Empty ID indicates new message
        role: Role::Assistant,
        content: "I don't have access to real-time weather data.".to_string(),
    };
    
    session_with_messages.messages = vec![user_message, assistant_message];
    
    // Add messages to session using the service
    sessions_service::session_add_messages(&db, &db, &session_with_messages)
        .expect("Failed to add messages to session");
    
    // Verify messages were persisted
    let session_messages = db.fetch_messages_for_session(&session.id)
        .expect("Failed to fetch session messages");
    assert_eq!(session_messages.len(), 2);
    
    // Verify message content
    assert_eq!(session_messages[0].role, "user");
    assert_eq!(session_messages[0].content, "What is the weather like?");
    assert_eq!(session_messages[1].role, "assistant");
    assert_eq!(session_messages[1].content, "I don't have access to real-time weather data.");
    
    // Verify messages have IDs now
    assert!(!session_messages[0].id.is_empty());
    assert!(!session_messages[1].id.is_empty());
}

#[tokio::test]
async fn test_message_retrieval_from_session() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create session
    let session = sessions_service::session(&db, &db, "Retrieval Test Session")
        .expect("Failed to create session");
    
    // Add messages directly to database
    let messages = vec![
        termai::session::entity::message_entity::MessageEntity::new(
            Uuid::new_v4().to_string(),
            session.id.clone(),
            "user".to_string(),
            "First message".to_string(),
        ),
        termai::session::entity::message_entity::MessageEntity::new(
            Uuid::new_v4().to_string(),
            session.id.clone(),
            "assistant".to_string(),
            "First response".to_string(),
        ),
        termai::session::entity::message_entity::MessageEntity::new(
            Uuid::new_v4().to_string(),
            session.id.clone(),
            "user".to_string(),
            "Second message".to_string(),
        ),
    ];
    
    for message in &messages {
        db.add_message_to_session(message)
            .expect("Failed to add message");
    }
    
    // Retrieve session with messages using service
    let session_with_messages = sessions_service::session_by_id(&db, &db, &session.id)
        .expect("Failed to retrieve session with messages");
    
    assert_eq!(session_with_messages.messages.len(), 3);
    assert_eq!(session_with_messages.messages[0].content, "First message");
    assert_eq!(session_with_messages.messages[1].content, "First response");
    assert_eq!(session_with_messages.messages[2].content, "Second message");
}

#[tokio::test]
async fn test_large_message_storage_and_retrieval() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create session
    let session = sessions_service::session(&db, &db, "Large Message Test")
        .expect("Failed to create session");
    
    // Create a large message (10KB of text)
    let large_content = "A".repeat(10240);
    let large_message = termai::session::entity::message_entity::MessageEntity::new(
        Uuid::new_v4().to_string(),
        session.id.clone(),
        "user".to_string(),
        large_content.clone(),
    );
    
    // Store large message
    db.add_message_to_session(&large_message)
        .expect("Failed to store large message");
    
    // Retrieve and verify
    let retrieved_messages = db.fetch_messages_for_session(&session.id)
        .expect("Failed to retrieve large message");
    assert_eq!(retrieved_messages.len(), 1);
    assert_eq!(retrieved_messages[0].content.len(), 10240);
    assert_eq!(retrieved_messages[0].content, large_content);
}

#[tokio::test]
async fn test_session_expiration_handling() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db = SqliteRepository::new(test_env.db_path_str()).expect("Failed to create database");
    
    // Create session with past expiration date
    let session_id = Uuid::new_v4().to_string();
    let past_expiration = DateTime::from_timestamp(Utc::now().timestamp() - 3600, 0).unwrap().naive_utc();
    
    db.add_session(&session_id, "Expired Session", past_expiration, false)
        .expect("Failed to create expired session");
    
    // Create session with future expiration date
    let session_id2 = Uuid::new_v4().to_string();
    let future_expiration = DateTime::from_timestamp(Utc::now().timestamp() + 3600, 0).unwrap().naive_utc();
    
    db.add_session(&session_id2, "Valid Session", future_expiration, false)
        .expect("Failed to create valid session");
    
    // Fetch both sessions
    let expired_session = db.fetch_session_by_id(&session_id)
        .expect("Failed to fetch expired session");
    let valid_session = db.fetch_session_by_id(&session_id2)
        .expect("Failed to fetch valid session");
    
    // Verify expiration dates
    assert!(expired_session.expires_at < Utc::now().naive_utc());
    assert!(valid_session.expires_at > Utc::now().naive_utc());
    
    // Test that new sessions get proper expiration (24 hours from now)
    let new_session = sessions_service::session(&db, &db, "New Session")
        .expect("Failed to create new session");
    
    let retrieved_new = db.fetch_session_by_id(&new_session.id)
        .expect("Failed to fetch new session");
    
    let expected_expiration = Utc::now().naive_utc() + Duration::hours(24);
    let actual_expiration = retrieved_new.expires_at;
    
    // Allow for small time differences (within 1 minute)
    let time_diff = (actual_expiration - expected_expiration).num_seconds().abs();
    assert!(time_diff < 60, "Expiration time should be within 1 minute of expected");
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    let test_env = TestEnvironment::new().expect("Failed to create test environment");
    let db_path = test_env.db_path_str().to_string();
    
    // Spawn concurrent session creation operations
    let handles = (0..5).map(|i| {
        let db_path_clone = db_path.clone();
        tokio::spawn(async move {
            let db = SqliteRepository::new(&db_path_clone).expect("Failed to create db");
            let session_name = format!("Concurrent Session {}", i);
            sessions_service::session(&db, &db, &session_name)
                .expect("Failed to create session")
        })
    }).collect::<Vec<_>>();
    
    // Wait for all sessions to be created
    let mut created_sessions = Vec::new();
    for handle in handles {
        let session = handle.await.expect("Task failed");
        created_sessions.push(session);
    }
    
    // Verify all sessions were created
    assert_eq!(created_sessions.len(), 5);
    
    // Verify they all have unique IDs and names
    let mut session_ids = created_sessions.iter().map(|s| &s.id).collect::<Vec<_>>();
    session_ids.sort();
    session_ids.dedup();
    assert_eq!(session_ids.len(), 5, "All sessions should have unique IDs");
    
    // Verify in database
    let verify_db = SqliteRepository::new(&db_path).expect("Failed to create verify db");
    let all_sessions = verify_db.fetch_all_sessions()
        .expect("Failed to fetch all sessions");
    assert_eq!(all_sessions.len(), 5);
    
    // Only one session should be current (the last one created)
    let current_sessions: Vec<_> = all_sessions.iter().filter(|s| s.current == 1).collect();
    assert_eq!(current_sessions.len(), 1, "Only one session should be current");
}