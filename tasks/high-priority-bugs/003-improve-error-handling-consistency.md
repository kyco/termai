# Task: Improve Error Handling Consistency and User Experience

## Priority: High
## Estimated Effort: 2-3 days
## Dependencies: None
## Files Affected: Multiple files across the codebase

## Overview
Standardize error handling throughout the application to provide consistent, user-friendly error messages, proper logging, and graceful recovery mechanisms. Currently, errors are handled inconsistently with silent failures and poor user feedback.

## Bug Description
Multiple error handling issues exist:
1. Silent failures in database operations
2. Inconsistent error message formats
3. Missing error logging
4. Poor user feedback for failures
5. No error recovery mechanisms

## Root Cause Analysis
1. **No Error Standards**: Each module handles errors differently
2. **Silent Failures**: Errors absorbed without notification
3. **Technical Exposure**: Raw technical errors shown to users
4. **No Logging Strategy**: Errors not properly logged for debugging
5. **Poor Recovery**: No graceful degradation on failures

## Current Problematic Patterns
```rust
// Silent error handling
Err(_) => {
    // Keep default temporary session
}

// Technical errors exposed to users
.expect("Invalid DateTime format")

// Inconsistent error types
Result<(), rusqlite::Error>
Result<Session, anyhow::Error>
```

## Implementation Steps

### 1. Create Centralized Error Management System
```rust
// src/error/mod.rs
use thiserror::Error;
use std::fmt;

#[derive(Error, Debug)]
pub enum TermAIError {
    #[error("Database error: {message}")]
    Database { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Network error: {message}")]
    Network { 
        message: String,
        retry_after: Option<std::time::Duration>,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("API error: {provider} returned {status}: {message}")]
    ApiError {
        provider: String,
        status: u16,
        message: String,
        retry_after: Option<std::time::Duration>,
    },
    
    #[error("Configuration error: {message}")]
    Configuration { message: String },
    
    #[error("Session error: {message}")]
    Session { message: String },
    
    #[error("Input validation error: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("File system error: {operation} failed: {message}")]
    FileSystem { operation: String, message: String },
    
    #[error("Internal error: {message}")]
    Internal { message: String },
    
    #[error("Operation cancelled by user")]
    Cancelled,
    
    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },
}

impl TermAIError {
    pub fn database<E: std::error::Error + Send + Sync + 'static>(msg: &str, source: E) -> Self {
        Self::Database {
            message: msg.to_string(),
            source: Some(Box::new(source)),
        }
    }
    
    pub fn network<E: std::error::Error + Send + Sync + 'static>(msg: &str, source: E) -> Self {
        Self::Network {
            message: msg.to_string(),
            retry_after: None,
            source: Some(Box::new(source)),
        }
    }
    
    pub fn api_error(provider: &str, status: u16, message: &str) -> Self {
        Self::ApiError {
            provider: provider.to_string(),
            status,
            message: message.to_string(),
            retry_after: None,
        }
    }
    
    pub fn validation(field: &str, message: &str) -> Self {
        Self::Validation {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
    
    pub fn session(message: &str) -> Self {
        Self::Session {
            message: message.to_string(),
        }
    }
    
    pub fn user_message(&self) -> String {
        match self {
            Self::Database { .. } => {
                "There was a problem saving your data. Your recent changes may not be saved.".to_string()
            }
            Self::Network { .. } => {
                "Unable to connect to the internet. Please check your connection and try again.".to_string()
            }
            Self::ApiError { provider, status, .. } => {
                match *status {
                    401 => format!("{} API key is invalid. Please check your settings.", provider),
                    429 => format!("{} rate limit exceeded. Please wait a moment and try again.", provider),
                    500..=599 => format!("{} service is temporarily unavailable. Please try again later.", provider),
                    _ => format!("Unable to communicate with {}. Please try again.", provider),
                }
            }
            Self::Configuration { .. } => {
                "There's an issue with your settings. Please check the configuration.".to_string()
            }
            Self::Session { message } => {
                format!("Session error: {}", message)
            }
            Self::Validation { field, message } => {
                format!("{}: {}", field, message)
            }
            Self::FileSystem { operation, .. } => {
                format!("Unable to {0}. Please check file permissions.", operation)
            }
            Self::Cancelled => {
                "Operation was cancelled.".to_string()
            }
            Self::Internal { .. } => {
                "An unexpected error occurred. Please try again.".to_string()
            }
            Self::NotImplemented { feature } => {
                format!("{} is not yet implemented.", feature)
            }
        }
    }
    
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. } => true,
            Self::ApiError { status, .. } => matches!(*status, 429 | 500..=599),
            Self::Database { .. } => false, // Usually not retryable
            _ => false,
        }
    }
    
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            Self::Network { retry_after, .. } => *retry_after,
            Self::ApiError { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, TermAIError>;
```

### 2. Create Error Context and Logging
```rust
// src/error/context.rs
use super::{TermAIError, Result};
use std::fmt::Display;

pub trait ErrorContext<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
    
    fn with_session_context(self, session_id: &str) -> Result<T>;
    fn with_operation_context(self, operation: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            // Determine error type based on error type
            if e.to_string().contains("database") || e.to_string().contains("sqlite") {
                TermAIError::database(&f(), e)
            } else if e.to_string().contains("network") || e.to_string().contains("connection") {
                TermAIError::network(&f(), e)
            } else {
                TermAIError::Internal { message: f() }
            }
        })
    }
    
    fn with_session_context(self, session_id: &str) -> Result<T> {
        self.with_context(|| format!("Session operation failed for session {}", session_id))
    }
    
    fn with_operation_context(self, operation: &str) -> Result<T> {
        self.with_context(|| format!("Operation '{}' failed", operation))
    }
}

// Logging infrastructure
pub struct ErrorLogger;

impl ErrorLogger {
    pub fn log_error(error: &TermAIError, context: Option<&str>) {
        let context_str = context.unwrap_or("Unknown");
        
        match error {
            TermAIError::Database { message, source } => {
                eprintln!("[ERROR] Database error in {}: {}", context_str, message);
                if let Some(source) = source {
                    eprintln!("  Caused by: {}", source);
                }
            }
            TermAIError::Network { message, source, .. } => {
                eprintln!("[ERROR] Network error in {}: {}", context_str, message);
                if let Some(source) = source {
                    eprintln!("  Caused by: {}", source);
                }
            }
            TermAIError::ApiError { provider, status, message, .. } => {
                eprintln!("[ERROR] API error in {}: {} {} - {}", context_str, provider, status, message);
            }
            _ => {
                eprintln!("[ERROR] Error in {}: {}", context_str, error);
            }
        }
    }
    
    pub fn log_warning(message: &str, context: Option<&str>) {
        let context_str = context.unwrap_or("Unknown");
        eprintln!("[WARN] Warning in {}: {}", context_str, message);
    }
    
    pub fn log_info(message: &str, context: Option<&str>) {
        let context_str = context.unwrap_or("Unknown");
        eprintln!("[INFO] Info in {}: {}", context_str, message);
    }
}
```

### 3. Update Database Layer with Proper Error Handling
```rust
// src/session/repository/session_repository.rs
use crate::error::{TermAIError, Result, ErrorContext, ErrorLogger};

impl SessionRepository for SqliteRepository {
    type Error = TermAIError;

    fn fetch_session_by_id(&self, id: &str) -> Result<SessionEntity> {
        let result = self.conn.query_row(
            "SELECT id, name, expires_at, current FROM sessions WHERE id = ?1",
            params![id],
            row_to_session_entity(),
        );
        
        match result {
            Ok(session) => Ok(session),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(TermAIError::session(&format!("Session '{}' not found", id)))
            }
            Err(e) => {
                ErrorLogger::log_error(&TermAIError::database("Failed to fetch session", e), Some("session_repository"));
                Err(TermAIError::database("Failed to fetch session", e))
            }
        }
    }
    
    fn fetch_all_sessions(&self) -> Result<Vec<SessionEntity>> {
        let mut stmt = self.conn
            .prepare("SELECT id, name, expires_at, current FROM sessions ORDER BY ROWID DESC")
            .with_operation_context("prepare fetch all sessions query")?;
            
        let rows = stmt.query_map([], row_to_session_entity())
            .with_operation_context("execute fetch all sessions query")?;

        let mut sessions = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(session) => sessions.push(session),
                Err(e) => {
                    ErrorLogger::log_warning(
                        &format!("Skipping corrupted session row: {}", e), 
                        Some("fetch_all_sessions")
                    );
                    // Continue processing other sessions instead of failing completely
                }
            }
        }
        
        Ok(sessions)
    }
    
    fn add_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<()> {
        // Validate inputs
        if id.is_empty() {
            return Err(TermAIError::validation("session_id", "Session ID cannot be empty"));
        }
        if name.is_empty() {
            return Err(TermAIError::validation("session_name", "Session name cannot be empty"));
        }
        
        let expires_at_str = expires_at.format(DATE_TIME_FORMAT).to_string();
        let current_i = if current { 1 } else { 0 };
        
        self.conn.execute(
            "INSERT INTO sessions (id, name, expires_at, current) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, expires_at_str, current_i],
        )
        .with_context(|| format!("Failed to add session '{}'", id))?;
        
        ErrorLogger::log_info(&format!("Added session '{}' ('{}')", id, name), Some("session_repository"));
        Ok(())
    }
    
    fn update_session(
        &self,
        id: &str,
        name: &str,
        expires_at: NaiveDateTime,
        current: bool,
    ) -> Result<()> {
        let expires_at_str = expires_at.format(DATE_TIME_FORMAT).to_string();
        let current_i = if current { 1 } else { 0 };
        
        let rows_affected = self.conn.execute(
            "UPDATE sessions SET name = ?1, expires_at = ?2, current = ?3 WHERE id = ?4",
            params![name, expires_at_str, current_i, id],
        )
        .with_context(|| format!("Failed to update session '{}'", id))?;
        
        if rows_affected == 0 {
            return Err(TermAIError::session(&format!("Session '{}' not found for update", id)));
        }
        
        Ok(())
    }
}

// Update row parsing with better error handling
fn row_to_session_entity() -> fn(&Row) -> rusqlite::Result<SessionEntity> {
    |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let expires_at_str: String = row.get(2)?;
        let current: i32 = row.get(3)?;
        
        // Use safe datetime parsing
        let expires_at = crate::utils::datetime::parse_datetime_safe(&expires_at_str)
            .unwrap_or_else(|_| crate::utils::datetime::default_expiration());

        Ok(SessionEntity::new(id, name, expires_at, current))
    }
}
```

### 4. Update UI Layer with User-Friendly Error Display
```rust
// src/ui/error/error_display.rs
use crate::error::TermAIError;
use ratatui::{
    widgets::{Block, Borders, Paragraph, Clear},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    Frame,
};

#[derive(Debug, Clone)]
pub struct ErrorState {
    pub error: Option<TermAIError>,
    pub show_details: bool,
    pub auto_dismiss_timer: Option<std::time::Instant>,
}

impl ErrorState {
    pub fn new() -> Self {
        Self {
            error: None,
            show_details: false,
            auto_dismiss_timer: None,
        }
    }
    
    pub fn set_error(&mut self, error: TermAIError) {
        self.error = Some(error);
        self.show_details = false;
        // Auto-dismiss after 10 seconds for non-critical errors
        if !self.is_critical_error() {
            self.auto_dismiss_timer = Some(std::time::Instant::now());
        }
    }
    
    pub fn clear_error(&mut self) {
        self.error = None;
        self.show_details = false;
        self.auto_dismiss_timer = None;
    }
    
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    pub fn should_auto_dismiss(&self) -> bool {
        if let Some(timer) = self.auto_dismiss_timer {
            timer.elapsed() > std::time::Duration::from_secs(10)
        } else {
            false
        }
    }
    
    fn is_critical_error(&self) -> bool {
        match &self.error {
            Some(TermAIError::Database { .. }) => true,
            Some(TermAIError::Configuration { .. }) => true,
            _ => false,
        }
    }
}

pub fn draw_error_popup(f: &mut Frame, error_state: &ErrorState, area: Rect) {
    if let Some(error) = &error_state.error {
        let popup_area = centered_rect(70, 40, area);
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        // Create error content
        let error_content = create_error_content(error, error_state.show_details);
        
        // Determine colors based on error severity
        let (border_color, title_color) = match error {
            TermAIError::Database { .. } | TermAIError::Configuration { .. } => {
                (Color::Red, Color::Red)
            }
            TermAIError::Network { .. } | TermAIError::ApiError { .. } => {
                (Color::Yellow, Color::Yellow)
            }
            _ => (Color::Blue, Color::Blue),
        };
        
        let error_widget = Paragraph::new(error_content)
            .block(
                Block::default()
                    .title(" Error ")
                    .title_style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(error_widget, popup_area);
        
        // Draw help text at bottom
        let help_area = Rect {
            y: popup_area.y + popup_area.height - 2,
            height: 1,
            ..popup_area
        };
        
        let help_text = if error_state.show_details {
            "Press 'd' to hide details, ESC to dismiss"
        } else {
            "Press 'd' for details, ESC to dismiss"
        };
        
        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(help_widget, help_area);
    }
}

fn create_error_content(error: &TermAIError, show_details: bool) -> Text {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            error.user_message(),
            Style::default().add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
    ];
    
    // Add retry information if applicable
    if error.is_retryable() {
        if let Some(retry_after) = error.retry_after() {
            lines.push(Line::from(format!(
                "You can try again in {} seconds.",
                retry_after.as_secs()
            )));
        } else {
            lines.push(Line::from("You can try again in a moment."));
        }
        lines.push(Line::from(""));
    }
    
    // Add suggestions based on error type
    match error {
        TermAIError::ApiError { provider, status, .. } => {
            match *status {
                401 => {
                    lines.push(Line::from("💡 Suggestion:"));
                    lines.push(Line::from(format!(
                        "   • Check your {} API key in settings",
                        provider
                    )));
                    lines.push(Line::from("   • Make sure your API key is active"));
                }
                429 => {
                    lines.push(Line::from("💡 Suggestion:"));
                    lines.push(Line::from("   • Wait a moment before trying again"));
                    lines.push(Line::from("   • Consider upgrading your API plan"));
                }
                _ => {}
            }
        }
        TermAIError::Network { .. } => {
            lines.push(Line::from("💡 Suggestion:"));
            lines.push(Line::from("   • Check your internet connection"));
            lines.push(Line::from("   • Try again in a moment"));
        }
        TermAIError::Configuration { .. } => {
            lines.push(Line::from("💡 Suggestion:"));
            lines.push(Line::from("   • Check your settings (Ctrl+S)"));
            lines.push(Line::from("   • Verify your API keys"));
        }
        _ => {}
    }
    
    // Add technical details if requested
    if show_details {
        lines.push(Line::from(""));
        lines.push(Line::from("Technical Details:"));
        lines.push(Line::from(format!("  {}", error)));
        
        // Add chain of causes
        let mut source = error.source();
        while let Some(err) = source {
            lines.push(Line::from(format!("  Caused by: {}", err)));
            source = err.source();
        }
    }
    
    Text::from(lines)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

### 5. Update App State with Error Management
```rust
// In app.rs
use crate::ui::error::error_display::ErrorState;
use crate::error::{TermAIError, ErrorLogger};

pub struct App {
    // Replace simple error_message with comprehensive error state
    pub error_state: ErrorState,
    // ... other fields
}

impl App {
    pub fn set_error(&mut self, error: TermAIError) {
        ErrorLogger::log_error(&error, Some("UI"));
        self.error_state.set_error(error);
    }
    
    pub fn clear_error(&mut self) {
        self.error_state.clear_error();
    }
    
    pub fn toggle_error_details(&mut self) {
        self.error_state.toggle_details();
    }
    
    pub fn check_error_auto_dismiss(&mut self) {
        if self.error_state.should_auto_dismiss() {
            self.error_state.clear_error();
        }
    }
    
    // Update existing error handling methods
    pub fn handle_operation_result<T>(&mut self, result: crate::error::Result<T>, operation: &str) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                ErrorLogger::log_error(&error, Some(operation));
                self.set_error(error);
                None
            }
        }
    }
}
```

### 6. Update Event Handling with Error Management
```rust
// In runner.rs
// Update error handling in the main event loop
match chat_result.0 {
    Ok(_) => {
        app.clear_error();
        app.scroll_to_bottom();
        // ... success handling
    }
    Err(e) => {
        // Convert anyhow error to TermAIError
        let termai_error = match e.downcast::<TermAIError>() {
            Ok(termai_err) => termai_err,
            Err(other_err) => TermAIError::Internal { 
                message: other_err.to_string() 
            },
        };
        app.set_error(termai_error);
    }
}

// Add error key handling
KeyAction::ToggleErrorDetails => {
    app.toggle_error_details();
}
KeyAction::ExitEditMode => {
    if app.error_state.error.is_some() {
        app.clear_error();
    } else if app.show_help {
        app.toggle_help();
    }
    // ... other exit logic
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let db_error = TermAIError::database("Connection failed", std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(db_error.user_message().contains("saving your data"));
        
        let api_error = TermAIError::api_error("OpenAI", 401, "Invalid API key");
        assert!(api_error.user_message().contains("API key is invalid"));
        
        let network_error = TermAIError::network("Connection timeout", std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"));
        assert!(network_error.user_message().contains("connect to the internet"));
    }
    
    #[test]
    fn test_error_retryable() {
        let retryable = TermAIError::api_error("OpenAI", 429, "Rate limited");
        assert!(retryable.is_retryable());
        
        let not_retryable = TermAIError::validation("field", "Invalid value");
        assert!(!not_retryable.is_retryable());
    }
    
    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        let with_context = result.with_operation_context("test operation");
        
        assert!(with_context.is_err());
        assert!(with_context.unwrap_err().to_string().contains("test operation"));
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_error_handling_flow() {
    let mut app = App::new();
    
    // Simulate an error
    let error = TermAIError::api_error("OpenAI", 401, "Invalid API key");
    app.set_error(error);
    
    // Verify error state
    assert!(app.error_state.error.is_some());
    
    // Test error dismissal
    app.clear_error();
    assert!(app.error_state.error.is_none());
}
```

## Acceptance Criteria
- [ ] All errors have user-friendly messages
- [ ] Technical details are hidden by default but accessible
- [ ] Errors are properly logged for debugging
- [ ] Retryable errors are clearly indicated
- [ ] Silent failures are eliminated
- [ ] Error recovery mechanisms work correctly
- [ ] Error display is consistent throughout the app
- [ ] Performance impact is minimal

## Error Recovery Strategies
1. **Graceful Degradation**: Continue operation with reduced functionality
2. **Automatic Retry**: Retry transient failures with backoff
3. **User Guidance**: Provide clear steps for user resolution
4. **State Recovery**: Restore previous working state when possible

## Future Enhancements
- Error reporting to external services
- Error analytics and trending
- Smart error suggestions based on patterns
- Multi-language error messages
- Voice error announcements for accessibility