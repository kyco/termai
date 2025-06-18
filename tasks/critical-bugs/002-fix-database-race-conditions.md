# Task: Fix Database Race Conditions and Synchronization Issues

## Priority: Critical
## Estimated Effort: 2-3 days
## Dependencies: None
## Files Affected: `src/ui/tui/runner.rs`, `src/session/repository/*.rs`, `src/ui/tui/app.rs`

## Overview
Fix critical race conditions between UI state and database operations that can cause data corruption, inconsistent state, and potential crashes during concurrent session modifications.

## Bug Description
Multiple race conditions exist in the session management system:
1. UI state can be modified while database operations are in progress
2. Session refresh can overwrite unsaved changes
3. Concurrent message additions can be lost
4. Database connection not properly synchronized

## Root Cause Analysis
1. **No Synchronization**: Database operations are not atomic with UI updates
2. **Shared Mutable State**: Session objects modified in multiple places without coordination
3. **Async/Sync Mixing**: Async operations mixed with synchronous database calls
4. **No Transaction Management**: Individual operations not grouped into transactions

## Current Buggy Code
```rust
// In runner.rs:143-184
let chat_result = if let Some(session) = app.current_session_mut() {
    let was_temporary = session.temporary;
    // BUG: session could be modified by another operation here
    let result = chat::send_message_async(/* ... */).await;
    
    // BUG: Session state might be stale by now
    let should_convert = was_temporary && session.messages.len() >= 2;
```

## Implementation Steps

### 1. Add Database Transaction Support
```rust
// src/repository/db.rs
use rusqlite::{Connection, Result, Transaction};
use std::sync::{Arc, Mutex};

pub struct SqliteRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteRepository {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // ... existing setup code ...
        Ok(Self { 
            conn: Arc::new(Mutex::new(conn)) 
        })
    }
    
    pub fn with_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Transaction) -> Result<R>,
    {
        let conn = self.conn.lock().unwrap();
        let tx = conn.unchecked_transaction()?;
        let result = f(&tx);
        match result {
            Ok(value) => {
                tx.commit()?;
                Ok(value)
            }
            Err(e) => {
                let _ = tx.rollback();
                Err(e)
            }
        }
    }
}
```

### 2. Create Atomic Session Operations
```rust
// src/session/service/session_service.rs
use std::sync::{Arc, RwLock};

pub struct SessionService<SR, MR> {
    session_repo: Arc<SR>,
    message_repo: Arc<MR>,
    // Cache with read-write lock for thread safety
    session_cache: Arc<RwLock<HashMap<String, Session>>>,
}

impl<SR, MR> SessionService<SR, MR>
where
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    pub async fn send_message_atomic(
        &self,
        session_id: &str,
        message: String,
        role: Role,
    ) -> Result<Session> {
        // Use database transaction to ensure atomicity
        self.session_repo.with_transaction(|tx| {
            // 1. Load fresh session from DB
            let session_entity = self.session_repo.fetch_session_by_id_tx(tx, session_id)?;
            let mut session = Session::from(&session_entity);
            
            // 2. Load messages
            let message_entities = self.message_repo.fetch_messages_for_session_tx(tx, session_id)?;
            let messages = message_entities.iter()
                .map(|m| Message::from(m))
                .collect();
            session = session.copy_with_messages(messages);
            
            // 3. Add new message
            session.add_raw_message(message, role);
            
            // 4. Save to database within transaction
            let new_message = session.messages.last().unwrap();
            let message_entity = MessageEntity::from(new_message);
            self.message_repo.add_message_to_session_tx(tx, &message_entity)?;
            
            // 5. Update session metadata
            self.session_repo.update_session_tx(
                tx,
                &session.id,
                &session.name,
                session.expires_at,
                session.current,
            )?;
            
            Ok(session)
        })
    }
    
    pub async fn convert_session_atomic(
        &self,
        session_id: &str,
        new_name: String,
    ) -> Result<Session> {
        self.session_repo.with_transaction(|tx| {
            // Load session
            let session_entity = self.session_repo.fetch_session_by_id_tx(tx, session_id)?;
            let mut session = Session::from(&session_entity);
            
            // Update properties
            session.name = new_name;
            session.temporary = false;
            
            // Save changes
            self.session_repo.update_session_tx(
                tx,
                &session.id,
                &session.name,
                session.expires_at,
                session.current,
            )?;
            
            Ok(session)
        })
    }
}
```

### 3. Add Transaction Methods to Repositories
```rust
// src/session/repository/session_repository.rs
pub trait SessionRepository {
    type Error;
    
    // Existing methods...
    
    // New transaction-aware methods
    fn fetch_session_by_id_tx(&self, tx: &Transaction, id: &str) -> Result<SessionEntity, Self::Error>;
    fn update_session_tx(
        &self,
        tx: &Transaction,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error>;
    fn add_session_tx(
        &self,
        tx: &Transaction,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error>;
}

impl SessionRepository for SqliteRepository {
    fn fetch_session_by_id_tx(&self, tx: &Transaction, id: &str) -> Result<SessionEntity, Self::Error> {
        let session = tx.query_row(
            "SELECT id, name, expires_at, current FROM sessions WHERE id = ?1",
            params![id],
            row_to_session_entity(),
        )?;
        Ok(session)
    }
    
    fn update_session_tx(
        &self,
        tx: &Transaction,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error> {
        let expires_at_str = expires_at.format(DATE_TIME_FORMAT).to_string();
        let current_i = if current { 1 } else { 0 };
        tx.execute(
            "UPDATE sessions SET name = ?1, expires_at = ?2, current = ?3 WHERE id = ?4",
            params![name, expires_at_str, current_i, id],
        )?;
        Ok(())
    }
    
    // Similar implementations for other transaction methods...
}
```

### 4. Synchronize UI Updates
```rust
// src/ui/tui/app.rs
use std::sync::{Arc, RwLock};

pub struct App {
    // ... existing fields ...
    
    // Add operation locks
    session_operation_lock: Arc<RwLock<()>>,
    pending_operations: HashSet<String>, // Track sessions being modified
}

impl App {
    pub fn begin_session_operation(&mut self, session_id: &str) -> bool {
        if self.pending_operations.contains(session_id) {
            return false; // Operation already in progress
        }
        self.pending_operations.insert(session_id.to_string());
        true
    }
    
    pub fn end_session_operation(&mut self, session_id: &str) {
        self.pending_operations.remove(session_id);
    }
    
    pub fn is_session_busy(&self, session_id: &str) -> bool {
        self.pending_operations.contains(session_id)
    }
    
    pub fn refresh_session_safe(&mut self, session_id: &str, updated_session: Session) {
        // Only update if no operations are pending
        if !self.is_session_busy(session_id) {
            if let Some(index) = self.sessions.iter().position(|s| s.id == session_id) {
                self.sessions[index] = updated_session;
            }
        }
    }
}
```

### 5. Fix Event Loop Race Conditions
```rust
// src/ui/tui/runner.rs
pub async fn run_tui<R, SR, MR>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    // Create session service for atomic operations
    let session_service = Arc::new(SessionService::new(
        Arc::new(session_repository),
        Arc::new(message_repository),
    ));
    
    // Main event loop
    loop {
        // Check if current session needs refresh - but only if not busy
        if app.session_needs_refresh {
            if let Some(current_session) = app.current_session() {
                let session_id = current_session.id.clone();
                if !app.is_session_busy(&session_id) {
                    match session_service.load_session(&session_id).await {
                        Ok(updated_session) => {
                            app.refresh_session_safe(&session_id, updated_session);
                        }
                        Err(e) => {
                            eprintln!("Failed to refresh session: {}", e);
                        }
                    }
                }
            }
            app.session_needs_refresh = false;
        }
        
        // ... rest of event loop with atomic operations ...
        
        // Handle message sending with proper synchronization
        KeyAction::EnterEditMode => {
            if !app.is_input_editing() {
                // ... existing logic ...
            } else {
                let message = app.get_input_text().trim().to_string();
                if !message.is_empty() {
                    if let Some(session) = app.current_session() {
                        let session_id = session.id.clone();
                        
                        // Check if operation can start
                        if app.begin_session_operation(&session_id) {
                            // Immediately add user message to UI
                            app.add_message_to_current_session(message.clone(), Role::User);
                            app.clear_input();
                            app.scroll_to_bottom();
                            app.set_loading(true);
                            
                            // Force redraw
                            terminal.draw(|f| ui::draw(f, &mut app, Some(repo)))?;
                            
                            // Perform atomic database operation
                            match session_service.send_message_atomic(&session_id, message, Role::User).await {
                                Ok(updated_session) => {
                                    // Update UI with fresh session data
                                    app.refresh_session_safe(&session_id, updated_session);
                                    app.set_error(None);
                                    app.scroll_to_bottom();
                                }
                                Err(e) => {
                                    app.set_error(Some(format!("Error: {}", e)));
                                    // Revert UI changes on failure
                                    if let Some(session) = app.current_session_mut() {
                                        session.messages.pop(); // Remove failed message
                                    }
                                }
                            }
                            
                            app.end_session_operation(&session_id);
                            app.set_loading(false);
                        } else {
                            app.set_error(Some("Session is busy, please wait...".to_string()));
                        }
                    }
                }
            }
        }
    }
}
```

### 6. Add Optimistic Locking
```rust
// src/session/model/session.rs
pub struct Session {
    // ... existing fields ...
    pub version: u64, // Add version field for optimistic locking
}

// In database schema
// ALTER TABLE sessions ADD COLUMN version INTEGER DEFAULT 1;

// In update operations
fn update_session_with_version_check(
    &self,
    tx: &Transaction,
    session: &Session,
) -> Result<bool, Self::Error> {
    let rows_affected = tx.execute(
        "UPDATE sessions SET name = ?1, expires_at = ?2, version = version + 1 
         WHERE id = ?3 AND version = ?4",
        params![session.name, session.expires_at.format(DATE_TIME_FORMAT).to_string(), session.id, session.version],
    )?;
    
    Ok(rows_affected > 0) // Returns false if version mismatch (concurrent update)
}
```

## Testing Requirements

### Unit Tests
```rust
#[tokio::test]
async fn test_concurrent_message_addition() {
    let service = create_test_session_service().await;
    let session_id = "test_session";
    
    // Simulate concurrent message additions
    let handles: Vec<_> = (0..10).map(|i| {
        let service = service.clone();
        let session_id = session_id.to_string();
        tokio::spawn(async move {
            service.send_message_atomic(&session_id, format!("Message {}", i), Role::User).await
        })
    }).collect();
    
    // Wait for all operations
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    // Verify all messages were saved
    let session = service.load_session(session_id).await.unwrap();
    assert_eq!(session.messages.len(), 10);
}

#[test]
fn test_transaction_rollback() {
    let repo = create_test_repository();
    
    // Test that failed operations don't leave partial data
    let result = repo.with_transaction(|tx| {
        repo.add_session_tx(tx, "test", "Test Session", Utc::now().naive_utc(), false)?;
        // Simulate failure
        Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
            Some("Test failure".to_string()),
        ))
    });
    
    assert!(result.is_err());
    // Verify session wasn't created
    assert!(repo.fetch_session_by_id("test").is_err());
}
```

### Integration Tests
- Test UI consistency during database operations
- Test recovery from database connection failures
- Test concurrent user interactions

## Performance Considerations
1. **Connection Pooling**: Use connection pool for better performance
2. **Read Replicas**: Consider read replicas for query operations
3. **Batch Operations**: Group related operations into single transactions
4. **Cache Invalidation**: Implement proper cache invalidation strategy

## Acceptance Criteria
- [ ] No data loss during concurrent operations
- [ ] Session state remains consistent between UI and database
- [ ] Failed operations don't corrupt application state
- [ ] All database operations are atomic
- [ ] UI shows appropriate loading states during operations
- [ ] Error recovery works correctly
- [ ] Performance doesn't degrade significantly

## Rollback Plan
1. Keep existing synchronous operations as fallback
2. Add feature flag to enable/disable transaction mode
3. Monitor for deadlocks and implement timeout handling

## Future Enhancements
- Implement event sourcing for better conflict resolution
- Add distributed locking for multi-instance scenarios
- Implement automatic retry with exponential backoff
- Add metrics for operation timing and failure rates