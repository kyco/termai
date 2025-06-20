# Task: Fix DateTime Parsing Panics and Error Handling

## Priority: Critical
## Estimated Effort: 1 day
## Dependencies: None
## Files Affected: `src/session/repository/session_repository.rs`, `src/repository/db.rs`

## Overview
Fix critical panics caused by `.expect()` calls when parsing DateTime strings from the database. This can crash the application when the database contains malformed datetime data.

## Bug Description
In `session_repository.rs:99-100`, the code uses `.expect()` for DateTime parsing which will panic if the database contains malformed datetime strings. This is a critical reliability issue.

## Root Cause Analysis
1. **Unsafe Error Handling**: Using `.expect()` instead of proper error handling
2. **No Data Validation**: Database can contain invalid datetime formats
3. **No Migration Safety**: Schema changes could break datetime parsing
4. **Format Rigidity**: Fixed format string doesn't handle timezone variations

## Current Buggy Code
```rust
// In session_repository.rs:99-100
let expires_at = NaiveDateTime::parse_from_str(&expires_at_str, DATE_TIME_FORMAT)
    .expect("Invalid DateTime format"); // PANIC on malformed data
```

## Implementation Steps

### 1. Create Safe DateTime Parsing Utilities
```rust
// src/utils/datetime.rs
use chrono::{NaiveDateTime, Utc, DateTime};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DateTimeError {
    #[error("Invalid datetime format: {0}")]
    InvalidFormat(String),
    #[error("Datetime out of range: {0}")]
    OutOfRange(String),
    #[error("Timezone parsing error: {0}")]
    TimezoneError(String),
}

pub const PRIMARY_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const FALLBACK_DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S%.f",     // With microseconds
    "%Y-%m-%dT%H:%M:%S",        // ISO format without timezone
    "%Y-%m-%dT%H:%M:%S%.f",     // ISO with microseconds
    "%Y-%m-%dT%H:%M:%SZ",       // ISO with Z timezone
    "%Y-%m-%d %H:%M",           // Without seconds
    "%d/%m/%Y %H:%M:%S",        // Different date order
];

pub fn parse_datetime_safe(datetime_str: &str) -> Result<NaiveDateTime, DateTimeError> {
    // Try primary format first
    if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, PRIMARY_DATE_FORMAT) {
        return Ok(dt);
    }
    
    // Try fallback formats
    for format in FALLBACK_DATE_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
            return Ok(dt);
        }
    }
    
    // Try parsing as timestamp (Unix epoch)
    if let Ok(timestamp) = datetime_str.parse::<i64>() {
        if let Some(dt) = NaiveDateTime::from_timestamp_opt(timestamp, 0) {
            return Ok(dt);
        }
    }
    
    Err(DateTimeError::InvalidFormat(format!(
        "Unable to parse '{}' as datetime using any known format", 
        datetime_str
    )))
}

pub fn format_datetime_safe(datetime: &NaiveDateTime) -> String {
    datetime.format(PRIMARY_DATE_FORMAT).to_string()
}

pub fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub fn default_expiration() -> NaiveDateTime {
    Utc::now().naive_utc() + chrono::Duration::hours(24)
}

pub fn validate_datetime_range(datetime: &NaiveDateTime) -> Result<(), DateTimeError> {
    let min_date = NaiveDateTime::from_timestamp_opt(0, 0)
        .ok_or_else(|| DateTimeError::OutOfRange("Minimum date".to_string()))?;
    let max_date = NaiveDateTime::from_timestamp_opt(4_102_444_800, 0) // 2100-01-01
        .ok_or_else(|| DateTimeError::OutOfRange("Maximum date".to_string()))?;
    
    if *datetime < min_date || *datetime > max_date {
        return Err(DateTimeError::OutOfRange(format!(
            "DateTime {} is outside valid range ({} to {})",
            datetime, min_date, max_date
        )));
    }
    
    Ok(())
}
```

### 2. Update Session Repository with Safe Parsing
```rust
// src/session/repository/session_repository.rs
use crate::utils::datetime::{parse_datetime_safe, format_datetime_safe, default_expiration};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("DateTime error: {0}")]
    DateTime(#[from] crate::utils::datetime::DateTimeError),
    #[error("Session not found: {0}")]
    NotFound(String),
    #[error("Invalid session data: {0}")]
    InvalidData(String),
}

impl SessionRepository for SqliteRepository {
    type Error = SessionRepositoryError;

    fn fetch_session_by_id(&self, id: &str) -> Result<SessionEntity, Self::Error> {
        let result = self.conn.query_row(
            "SELECT id, name, expires_at, current FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let expires_at_str: String = row.get(2)?;
                let current: i32 = row.get(3)?;
                
                Ok((id, name, expires_at_str, current))
            },
        );
        
        match result {
            Ok((id, name, expires_at_str, current)) => {
                // Safe datetime parsing with fallback
                let expires_at = parse_datetime_safe(&expires_at_str)
                    .or_else(|_| {
                        // Log warning and use default expiration
                        eprintln!("Warning: Invalid datetime '{}' for session '{}', using default", 
                                expires_at_str, id);
                        Ok(default_expiration())
                    })?;
                
                Ok(SessionEntity::new(id, name, expires_at, current))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(SessionRepositoryError::NotFound(id.to_string()))
            }
            Err(e) => Err(SessionRepositoryError::Database(e))
        }
    }
    
    fn add_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error> {
        // Validate datetime before saving
        crate::utils::datetime::validate_datetime_range(&expires_at)?;
        
        let expires_at_str = format_datetime_safe(&expires_at);
        let current_i = if current { 1 } else { 0 };
        
        self.conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, expires_at_str, current_i],
        ).map_err(SessionRepositoryError::Database)?;
        
        Ok(())
    }

    fn update_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<(), Self::Error> {
        // Validate datetime before saving
        crate::utils::datetime::validate_datetime_range(&expires_at)?;
        
        let expires_at_str = format_datetime_safe(&expires_at);
        let current_i = if current { 1 } else { 0 };
        
        let rows_affected = self.conn.execute(
            "UPDATE sessions SET name = ?1, expires_at = ?2, current = ?3 WHERE id = ?4",
            params![name, expires_at_str, current_i, id],
        ).map_err(SessionRepositoryError::Database)?;
        
        if rows_affected == 0 {
            return Err(SessionRepositoryError::NotFound(id.to_string()));
        }
        
        Ok(())
    }
    
    fn fetch_all_sessions(&self) -> Result<Vec<SessionEntity>, Self::Error> {
        let mut stmt = self.conn
            .prepare("SELECT id, name, expires_at, current FROM sessions ORDER BY ROWID DESC")
            .map_err(SessionRepositoryError::Database)?;
            
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let expires_at_str: String = row.get(2)?;
            let current: i32 = row.get(3)?;
            
            Ok((id, name, expires_at_str, current))
        }).map_err(SessionRepositoryError::Database)?;

        let mut sessions = Vec::new();
        for row_result in rows {
            let (id, name, expires_at_str, current) = row_result
                .map_err(SessionRepositoryError::Database)?;
            
            // Safe parsing with recovery
            let expires_at = match parse_datetime_safe(&expires_at_str) {
                Ok(dt) => dt,
                Err(e) => {
                    eprintln!("Warning: Corrupted datetime for session '{}': {}. Using default.", id, e);
                    default_expiration()
                }
            };
            
            sessions.push(SessionEntity::new(id, name, expires_at, current));
        }
        
        Ok(sessions)
    }
}
```

### 3. Add Database Repair and Migration Tools
```rust
// src/repository/migrations.rs
use crate::utils::datetime::{parse_datetime_safe, format_datetime_safe, default_expiration};
use rusqlite::{Connection, Result, params};

pub fn repair_invalid_datetimes(conn: &Connection) -> Result<usize> {
    let mut repaired_count = 0;
    
    // Find sessions with invalid datetime formats
    let mut stmt = conn.prepare(
        "SELECT id, expires_at FROM sessions"
    )?;
    
    let invalid_sessions: Vec<(String, String)> = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let expires_at_str: String = row.get(1)?;
        Ok((id, expires_at_str))
    })?
    .filter_map(|row| {
        if let Ok((id, expires_at_str)) = row {
            // Test if datetime is valid
            if parse_datetime_safe(&expires_at_str).is_err() {
                Some((id, expires_at_str))
            } else {
                None
            }
        } else {
            None
        }
    })
    .collect();
    
    // Repair invalid datetimes
    for (session_id, invalid_datetime) in invalid_sessions {
        let fixed_datetime = format_datetime_safe(&default_expiration());
        
        conn.execute(
            "UPDATE sessions SET expires_at = ?1 WHERE id = ?2",
            params![fixed_datetime, session_id],
        )?;
        
        eprintln!("Repaired session '{}': '{}' -> '{}'", 
                session_id, invalid_datetime, fixed_datetime);
        repaired_count += 1;
    }
    
    Ok(repaired_count)
}

pub fn validate_database_integrity(conn: &Connection) -> Result<Vec<String>> {
    let mut issues = Vec::new();
    
    // Check for invalid datetimes
    let mut stmt = conn.prepare("SELECT id, expires_at FROM sessions")?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let expires_at_str: String = row.get(1)?;
        Ok((id, expires_at_str))
    })?;
    
    for row in rows {
        if let Ok((id, expires_at_str)) = row {
            if let Err(e) = parse_datetime_safe(&expires_at_str) {
                issues.push(format!("Session '{}' has invalid datetime: {}", id, e));
            }
        }
    }
    
    // Check for orphaned messages
    let orphan_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages m 
         LEFT JOIN sessions s ON m.session_id = s.id 
         WHERE s.id IS NULL",
        [],
        |row| row.get(0),
    )?;
    
    if orphan_count > 0 {
        issues.push(format!("{} orphaned messages found", orphan_count));
    }
    
    Ok(issues)
}
```

### 4. Update Database Initialization with Repair
```rust
// src/repository/db.rs
impl SqliteRepository {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Create tables
        create_table_messages(&conn)?;
        create_table_config(&conn)?;
        create_table_sessions(&conn)?;
        
        // Run migrations
        migrate_messages_id_column(&conn)?;
        messages_add_session_id_column(&conn)?;
        messages_add_role_column(&conn)?;
        sessions_add_current_column(&conn)?;
        sessions_rename_column_key_to_name(&conn)?;
        
        // Repair any corrupted data
        if let Ok(repaired) = crate::repository::migrations::repair_invalid_datetimes(&conn) {
            if repaired > 0 {
                eprintln!("Repaired {} sessions with invalid datetimes", repaired);
            }
        }
        
        // Validate database integrity in debug mode
        if cfg!(debug_assertions) {
            if let Ok(issues) = crate::repository::migrations::validate_database_integrity(&conn) {
                if !issues.is_empty() {
                    eprintln!("Database integrity issues found:");
                    for issue in issues {
                        eprintln!("  - {}", issue);
                    }
                }
            }
            debug_print_tables(&conn)?;
        }
        
        Ok(Self { conn })
    }
}
```

### 5. Add Error Recovery in UI Layer
```rust
// src/session/service/sessions_service.rs
pub fn session_by_id<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repo: &MR,
    session_id: &str,
) -> Result<Session> {
    match session_repo.fetch_session_by_id(session_id) {
        Ok(session_entity) => {
            let mut session = Session::from(&session_entity);
            
            // Load messages with error handling
            match message_repo.fetch_messages_for_session(session_id) {
                Ok(message_entities) => {
                    let messages = message_entities
                        .iter()
                        .map(|m| Message::from(m))
                        .collect();
                    session = session.copy_with_messages(messages);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load messages for session {}: {}", session_id, e);
                    // Continue with empty messages rather than failing
                }
            }
            
            Ok(session)
        }
        Err(SessionRepositoryError::NotFound(_)) => {
            Err(anyhow::anyhow!("Session not found: {}", session_id))
        }
        Err(SessionRepositoryError::DateTime(e)) => {
            eprintln!("DateTime error in session {}: {}. Creating recovery session.", session_id, e);
            
            // Create a recovery session with safe defaults
            let mut recovery_session = Session::new_temporary();
            recovery_session.id = session_id.to_string();
            recovery_session.name = format!("Recovered Session ({})", session_id);
            
            Ok(recovery_session)
        }
        Err(e) => Err(anyhow::anyhow!("Database error: {}", e))
    }
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::datetime::{parse_datetime_safe, format_datetime_safe};

    #[test]
    fn test_safe_datetime_parsing() {
        // Valid formats
        assert!(parse_datetime_safe("2024-01-15 10:30:45").is_ok());
        assert!(parse_datetime_safe("2024-01-15T10:30:45").is_ok());
        assert!(parse_datetime_safe("2024-01-15 10:30").is_ok());
        
        // Invalid formats
        assert!(parse_datetime_safe("invalid").is_err());
        assert!(parse_datetime_safe("").is_err());
        assert!(parse_datetime_safe("2024-13-50 25:70:90").is_err());
    }
    
    #[test]
    fn test_datetime_validation() {
        use crate::utils::datetime::{validate_datetime_range, now_naive};
        
        // Valid datetime
        assert!(validate_datetime_range(&now_naive()).is_ok());
        
        // Out of range datetimes
        let too_early = NaiveDateTime::from_timestamp_opt(-1, 0).unwrap();
        assert!(validate_datetime_range(&too_early).is_err());
    }
    
    #[test]
    fn test_repository_error_handling() {
        let repo = create_test_repository();
        
        // Insert session with valid datetime
        assert!(repo.add_session("test", "Test", now_naive(), false).is_ok());
        
        // Try to fetch non-existent session
        match repo.fetch_session_by_id("nonexistent") {
            Err(SessionRepositoryError::NotFound(_)) => {}, // Expected
            _ => panic!("Should return NotFound error"),
        }
    }
    
    #[test]
    fn test_corrupted_datetime_recovery() {
        let repo = create_test_repository();
        
        // Manually insert corrupted datetime
        repo.conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?, ?, ?, ?)",
            params!["corrupted", "Test", "invalid-datetime", 0],
        ).unwrap();
        
        // Should recover gracefully
        let sessions = repo.fetch_all_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "corrupted");
        // Should have valid datetime (default)
        assert!(sessions[0].expires_at > NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    }
}
```

### Integration Tests
```rust
#[test]
fn test_database_repair_on_startup() {
    let temp_db = create_temp_database();
    
    // Insert corrupted data
    {
        let conn = Connection::open(&temp_db).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?, ?, ?, ?)",
            params!["test1", "Test 1", "2024-13-50 25:70:90", 0],
        ).unwrap();
        conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?, ?, ?, ?)",
            params!["test2", "Test 2", "invalid", 0],
        ).unwrap();
    }
    
    // Opening repository should repair the data
    let repo = SqliteRepository::new(&temp_db).unwrap();
    let sessions = repo.fetch_all_sessions().unwrap();
    
    assert_eq!(sessions.len(), 2);
    // Both sessions should have valid datetimes
    for session in sessions {
        assert!(session.expires_at > NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    }
}
```

## Acceptance Criteria
- [ ] No panics when database contains invalid datetime strings
- [ ] Graceful recovery from corrupted datetime data
- [ ] All datetime operations use safe parsing
- [ ] Database repair runs automatically on startup
- [ ] Error messages are informative and actionable
- [ ] Performance impact is minimal
- [ ] All existing datetime data remains valid after migration

## Rollback Plan
1. Keep original parsing code as commented fallback
2. Add feature flag to disable safe parsing if needed
3. Create database backup before running repair operations

## Future Enhancements
- Add timezone support for international users
- Implement automatic data validation and repair jobs
- Add metrics for datetime parsing errors
- Consider using timestamp integers instead of string storage