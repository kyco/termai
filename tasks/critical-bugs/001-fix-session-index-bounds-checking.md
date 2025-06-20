# Task: Fix Session Index Out of Bounds Bug

## Priority: Critical
## Estimated Effort: 1 day
## Dependencies: None
## Files Affected: `src/ui/tui/app.rs`

## Overview
Fix a critical bug where session navigation can cause index out of bounds panics when the sessions list becomes empty or when switching between sessions.

## Bug Description
In `app.rs:132-134`, the code sets `current_session_index = 0` when the index exceeds sessions length, but doesn't check if the sessions list is empty. This causes panics when accessing `sessions[0]` on an empty vector.

## Root Cause Analysis
1. **Primary Issue**: `set_sessions()` method doesn't validate index bounds properly
2. **Secondary Issue**: Session navigation methods don't handle empty session lists
3. **Tertiary Issue**: No defensive programming around session access

## Current Buggy Code
```rust
// In set_sessions method
pub fn set_sessions(&mut self, sessions: Vec<Session>) {
    self.sessions = sessions;
    if self.current_session_index >= self.sessions.len() {
        self.current_session_index = 0; // BUG: Could be accessing empty vector
    }
}

// In current_session method
pub fn current_session(&self) -> Option<&Session> {
    self.sessions.get(self.current_session_index) // Can return None unexpectedly
}
```

## Implementation Steps

### 1. Fix Session Index Validation
```rust
// In app.rs
impl App {
    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
        
        // Safely handle index bounds
        if self.sessions.is_empty() {
            self.current_session_index = 0; // Safe even for empty vec
        } else if self.current_session_index >= self.sessions.len() {
            self.current_session_index = self.sessions.len() - 1; // Last valid index
        }
        
        // Reset scroll when sessions change
        self.scroll_offset = 0;
        self.session_scroll_offset = 0;
    }
    
    fn ensure_valid_session_index(&mut self) {
        if !self.sessions.is_empty() && self.current_session_index >= self.sessions.len() {
            self.current_session_index = self.sessions.len() - 1;
        }
    }
}
```

### 2. Add Safe Session Access Methods
```rust
impl App {
    pub fn current_session(&self) -> Option<&Session> {
        if self.sessions.is_empty() {
            None
        } else {
            self.sessions.get(self.current_session_index)
        }
    }
    
    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        if self.sessions.is_empty() {
            None
        } else {
            self.sessions.get_mut(self.current_session_index)
        }
    }
    
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }
    
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}
```

### 3. Fix Navigation Methods
```rust
impl App {
    pub fn next_session(&mut self) {
        if self.sessions.len() <= 1 {
            return; // No navigation needed
        }
        
        let new_index = (self.current_session_index + 1) % self.sessions.len();
        if new_index != self.current_session_index {
            self.current_session_index = new_index;
            self.scroll_offset = 0;
            self.session_needs_refresh = true;
        }
    }

    pub fn previous_session(&mut self) {
        if self.sessions.len() <= 1 {
            return; // No navigation needed
        }
        
        let new_index = if self.current_session_index == 0 {
            self.sessions.len() - 1
        } else {
            self.current_session_index - 1
        };
        
        if new_index != self.current_session_index {
            self.current_session_index = new_index;
            self.scroll_offset = 0;
            self.session_needs_refresh = true;
        }
    }
    
    pub fn switch_to_session_by_id(&mut self, session_id: &str) -> bool {
        for (index, session) in self.sessions.iter().enumerate() {
            if session.id == session_id {
                if index != self.current_session_index {
                    self.current_session_index = index;
                    self.scroll_offset = 0;
                    self.session_needs_refresh = true;
                }
                return true;
            }
        }
        false
    }
}
```

### 4. Add Session Removal Safety
```rust
impl App {
    pub fn remove_session(&mut self, session_id: &str) -> bool {
        if let Some(index) = self.sessions.iter().position(|s| s.id == session_id) {
            self.sessions.remove(index);
            
            // Adjust current index after removal
            if self.sessions.is_empty() {
                // Add a new temporary session if all sessions were removed
                self.sessions.push(Session::new_temporary());
                self.current_session_index = 0;
            } else if self.current_session_index >= self.sessions.len() {
                self.current_session_index = self.sessions.len() - 1;
            } else if index <= self.current_session_index && self.current_session_index > 0 {
                self.current_session_index -= 1;
            }
            
            self.scroll_offset = 0;
            self.session_scroll_offset = 0;
            return true;
        }
        false
    }
}
```

### 5. Add Validation to Critical Paths
```rust
impl App {
    pub fn add_message_to_current_session(&mut self, content: String, role: Role) {
        if let Some(session) = self.current_session_mut() {
            session.add_raw_message(content, role);
        } else {
            // Create a new session if none exists
            let mut new_session = Session::new_temporary();
            new_session.add_raw_message(content, role);
            self.sessions.push(new_session);
            self.current_session_index = self.sessions.len() - 1;
        }
    }
    
    pub fn create_new_session(&mut self) {
        let new_session = Session::new_temporary();
        self.sessions.insert(0, new_session);
        self.current_session_index = 0;
        self.scroll_offset = 0;
        self.session_scroll_offset = 0;
        self.focused_area = FocusedArea::Input;
        self.input_mode = InputMode::Editing;
    }
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_sessions_handling() {
        let mut app = App::new();
        app.set_sessions(vec![]);
        
        assert_eq!(app.current_session(), None);
        assert_eq!(app.session_count(), 0);
        assert!(!app.has_sessions());
    }
    
    #[test]
    fn test_session_index_bounds() {
        let mut app = App::new();
        let sessions = vec![
            Session::new_temporary(),
            Session::new_temporary(),
        ];
        
        app.current_session_index = 5; // Invalid index
        app.set_sessions(sessions);
        
        assert_eq!(app.current_session_index, 1); // Should be clamped to last valid
        assert!(app.current_session().is_some());
    }
    
    #[test]
    fn test_session_navigation_empty() {
        let mut app = App::new();
        app.set_sessions(vec![]);
        
        app.next_session();
        assert_eq!(app.current_session_index, 0);
        
        app.previous_session();
        assert_eq!(app.current_session_index, 0);
    }
    
    #[test]
    fn test_session_removal_edge_cases() {
        let mut app = App::new();
        let mut session1 = Session::new_temporary();
        session1.id = "session1".to_string();
        let mut session2 = Session::new_temporary();
        session2.id = "session2".to_string();
        
        app.set_sessions(vec![session1, session2]);
        app.current_session_index = 1;
        
        // Remove current session
        app.remove_session("session2");
        assert_eq!(app.current_session_index, 0);
        assert_eq!(app.session_count(), 1);
        
        // Remove last session
        app.remove_session("session1");
        assert_eq!(app.session_count(), 1); // Should add temporary session
        assert!(app.current_session().is_some());
    }
}
```

### Integration Tests
- Test session loading from database with corrupted data
- Test UI behavior when all sessions are deleted
- Test concurrent session modifications

## Error Handling Strategy
1. **Defensive Programming**: Always check bounds before accessing
2. **Graceful Degradation**: Create temporary session if none exist
3. **User Feedback**: Clear error messages for session-related issues
4. **Logging**: Log unexpected session state changes

## Acceptance Criteria
- [ ] No panics when sessions list is empty
- [ ] Session navigation works correctly with 0, 1, or many sessions
- [ ] Session index always points to valid session or None
- [ ] Session removal doesn't break current index
- [ ] UI remains responsive when session operations fail
- [ ] All unit tests pass
- [ ] Memory usage remains stable during session operations

## Rollback Plan
If issues arise:
1. Revert to previous session handling logic
2. Add additional bounds checking as hotfix
3. Test with small session sets first

## Future Enhancements
- Add session state validation on app startup
- Implement session backup/restore mechanism
- Add metrics for session operation failures
- Consider immutable session state management