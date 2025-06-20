# Task: Improve Session Lifetime Management

## Priority: High
## Estimated Effort: 2-3 days
## Dependencies: None

## Overview
Replace the fixed 24-hour session expiration with a flexible, user-controlled system that prevents important conversations from being lost while still managing storage effectively.

## Requirements

### Functional Requirements
1. **Flexible Expiration Options**
   - Never expire (permanent sessions)
   - Custom expiration periods (hours, days, weeks, months)
   - Activity-based expiration (inactive for X days)
   - Manual session archiving
   - Batch expiration management

2. **User Control**
   - Pin important sessions (never expire)
   - Extend expiration for active sessions
   - Archive old sessions instead of deleting
   - Bulk session management by age/activity
   - Session expiration warnings

3. **Storage Management**
   - Automatic cleanup of very old sessions
   - Configurable storage limits
   - Archive compression
   - Export before deletion options

### Technical Requirements
1. **Updated Session Model**
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct Session {
       pub id: String,
       pub name: String,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
       pub last_activity: DateTime<Utc>,
       pub expiration: SessionExpiration,
       pub status: SessionStatus,
       pub is_pinned: bool,
       pub folder_id: Option<String>,
       pub tags: Vec<String>,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum SessionExpiration {
       Never,
       Fixed(DateTime<Utc>),
       ActivityBased { inactive_days: u32 },
       Custom { period: Duration },
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum SessionStatus {
       Active,
       Archived,
       Scheduled,
       Temporary,
   }
   ```

2. **Session Lifecycle Service**
   ```rust
   pub struct SessionLifecycleService {
       session_repo: Arc<SessionRepository>,
       config_repo: Arc<ConfigRepository>,
       export_service: Arc<ExportService>,
   }
   
   impl SessionLifecycleService {
       pub async fn extend_session(&self, session_id: &str, new_expiration: SessionExpiration) -> Result<()>;
       pub async fn pin_session(&self, session_id: &str) -> Result<()>;
       pub async fn archive_session(&self, session_id: &str) -> Result<()>;
       pub async fn find_expiring_sessions(&self, within_hours: u32) -> Result<Vec<Session>>;
       pub async fn cleanup_expired_sessions(&self) -> Result<CleanupResult>;
       pub async fn get_storage_stats(&self) -> Result<StorageStats>;
   }
   ```

## Implementation Steps

1. **Database Migration**
   ```sql
   -- Add new columns to sessions table
   ALTER TABLE sessions ADD COLUMN last_activity INTEGER;
   ALTER TABLE sessions ADD COLUMN expiration_type TEXT DEFAULT 'Fixed';
   ALTER TABLE sessions ADD COLUMN expiration_value TEXT; -- JSON for flexible expiration data
   ALTER TABLE sessions ADD COLUMN status TEXT DEFAULT 'Active';
   ALTER TABLE sessions ADD COLUMN is_pinned BOOLEAN DEFAULT FALSE;
   
   -- Update existing sessions
   UPDATE sessions SET 
       last_activity = updated_at,
       expiration_type = 'Fixed',
       expiration_value = json_object('expires_at', expiration_time),
       status = CASE WHEN temporary THEN 'Temporary' ELSE 'Active' END;
   
   -- Create index for cleanup queries
   CREATE INDEX idx_sessions_expiration ON sessions(status, expiration_type, expiration_value);
   CREATE INDEX idx_sessions_activity ON sessions(last_activity, status);
   ```

2. **Session Expiration Settings UI**
   ```rust
   pub struct ExpirationSettings {
       pub default_expiration: SessionExpiration,
       pub cleanup_enabled: bool,
       pub cleanup_threshold_days: u32,
       pub storage_limit_mb: Option<u32>,
       pub warn_before_expiration_hours: u32,
   }
   
   impl ExpirationSettings {
       pub fn default() -> Self {
           Self {
               default_expiration: SessionExpiration::ActivityBased { inactive_days: 30 },
               cleanup_enabled: true,
               cleanup_threshold_days: 90,
               storage_limit_mb: Some(500),
               warn_before_expiration_hours: 24,
           }
       }
   }
   ```

3. **Session Management UI Enhancements**
   ```rust
   // In session list UI
   fn draw_session_with_expiration_info(f: &mut Frame, session: &Session, area: Rect) {
       let expiration_info = match &session.expiration {
           SessionExpiration::Never => "📌 Pinned".to_string(),
           SessionExpiration::Fixed(expires_at) => {
               let remaining = expires_at.signed_duration_since(Utc::now());
               if remaining.num_hours() < 24 {
                   format!("⏰ {}h left", remaining.num_hours())
               } else {
                   format!("📅 {} days", remaining.num_days())
               }
           }
           SessionExpiration::ActivityBased { inactive_days } => {
               let inactive_for = Utc::now().signed_duration_since(session.last_activity).num_days();
               format!("💤 {}/{} days", inactive_for, inactive_days)
           }
           SessionExpiration::Custom { period } => {
               format!("⚙️ Custom ({}d)", period.as_secs() / 86400)
           }
       };
       
       let chunks = Layout::default()
           .direction(Direction::Horizontal)
           .constraints([Constraint::Min(0), Constraint::Length(15)])
           .split(area);
       
       // Session name
       let session_name = Paragraph::new(session.name.clone())
           .style(Style::default().fg(Color::White));
       f.render_widget(session_name, chunks[0]);
       
       // Expiration info
       let expiration_style = match session.get_urgency() {
           Urgency::Critical => Style::default().fg(Color::Red),
           Urgency::Warning => Style::default().fg(Color::Yellow),
           Urgency::Normal => Style::default().fg(Color::Gray),
       };
       
       let expiration_text = Paragraph::new(expiration_info)
           .style(expiration_style);
       f.render_widget(expiration_text, chunks[1]);
   }
   ```

4. **Expiration Warnings System**
   ```rust
   pub struct ExpirationWarningService {
       session_repo: Arc<SessionRepository>,
       notification_service: Arc<NotificationService>,
   }
   
   impl ExpirationWarningService {
       pub async fn check_and_warn(&self) -> Result<()> {
           let expiring_sessions = self.session_repo
               .find_expiring_sessions(24) // Within 24 hours
               .await?;
           
           if !expiring_sessions.is_empty() {
               self.notification_service.show_expiration_warning(expiring_sessions).await?;
           }
           
           Ok(())
       }
   }
   
   // In UI
   fn draw_expiration_warning(f: &mut Frame, sessions: &[Session], area: Rect) {
       let warning_text = format!(
           "⚠️ {} session(s) will expire soon:\n\n{}",
           sessions.len(),
           sessions.iter()
               .map(|s| format!("• {}", s.name))
               .collect::<Vec<_>>()
               .join("\n")
       );
       
       let warning = Paragraph::new(warning_text)
           .style(Style::default().fg(Color::Yellow))
           .block(Block::default()
               .title("Expiring Sessions")
               .borders(Borders::ALL)
               .border_style(Style::default().fg(Color::Red))
           )
           .wrap(Wrap { trim: true });
       
       f.render_widget(warning, area);
   }
   ```

5. **Automatic Cleanup Service**
   ```rust
   pub struct CleanupService;
   
   impl CleanupService {
       pub async fn run_cleanup(&self, settings: &ExpirationSettings) -> Result<CleanupResult> {
           let mut result = CleanupResult::default();
           
           // Find expired sessions
           let expired_sessions = self.session_repo
               .find_expired_sessions()
               .await?;
           
           for session in expired_sessions {
               match session.status {
                   SessionStatus::Active => {
                       // Archive first, don't delete immediately
                       self.session_repo.archive_session(&session.id).await?;
                       result.archived += 1;
                   }
                   SessionStatus::Archived => {
                       // Check if archived long enough to delete
                       let archived_duration = Utc::now()
                           .signed_duration_since(session.updated_at)
                           .num_days();
                       
                       if archived_duration > settings.cleanup_threshold_days as i64 {
                           // Export before deletion if configured
                           if settings.export_before_delete {
                               self.export_service.export_session(&session.id, ExportFormat::Json).await?;
                           }
                           
                           self.session_repo.delete_session(&session.id).await?;
                           result.deleted += 1;
                       }
                   }
                   _ => {}
               }
           }
           
           Ok(result)
       }
   }
   ```

## Context Menu Integration
```rust
// Add expiration options to session context menu
pub enum SessionContextAction {
    // Existing actions...
    PinSession,
    UnpinSession,
    ExtendExpiration,
    ArchiveSession,
    SetCustomExpiration,
}

fn show_session_context_menu(&mut self, session: &Session) {
    let mut actions = vec![
        SessionContextAction::Rename,
        SessionContextAction::Delete,
        SessionContextAction::Export,
    ];
    
    if session.is_pinned {
        actions.push(SessionContextAction::UnpinSession);
    } else {
        actions.push(SessionContextAction::PinSession);
    }
    
    actions.extend([
        SessionContextAction::ExtendExpiration,
        SessionContextAction::SetCustomExpiration,
        SessionContextAction::ArchiveSession,
    ]);
    
    self.show_context_menu(actions);
}
```

## Testing Requirements
- Unit tests for expiration logic
- Integration tests for cleanup service
- UI tests for expiration warnings
- Migration tests for existing sessions
- Performance tests with large session counts

## Acceptance Criteria
- [ ] Sessions have flexible expiration options
- [ ] Users can pin important sessions
- [ ] Expiration warnings appear when appropriate
- [ ] Expired sessions are archived, not immediately deleted
- [ ] Bulk expiration management works
- [ ] Storage limits are respected
- [ ] Migration preserves existing sessions

## Default Behaviors
1. **New Sessions**: 30-day activity-based expiration
2. **Temporary Sessions**: Convert to permanent after first exchange with default expiration
3. **Pinned Sessions**: Never expire unless manually unpinned
4. **Archived Sessions**: Kept for 90 days before permanent deletion
5. **Cleanup**: Runs weekly by default

## Future Enhancements
- Smart expiration based on session importance
- Machine learning to suggest which sessions to keep
- Cloud backup integration for long-term storage
- Session analytics to optimize expiration policies
- Collaborative session sharing with expiration controls