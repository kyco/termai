# Task: Implement Session Search Functionality

## Priority: High
## Estimated Effort: 3-5 days
## Dependencies: None

## Overview
Implement a search feature that allows users to find sessions by title or content. This is critical for users with many sessions who need to quickly locate previous conversations.

## Requirements

### Functional Requirements
1. **Search Interface in TUI**
   - Add search mode triggered by `/` key (vim-style)
   - Display search box overlay on session list
   - Show real-time results as user types
   - Highlight matching text in results
   - Support case-insensitive search by default

2. **Search Capabilities**
   - Search by session title
   - Search by message content
   - Search by date range (optional advanced feature)
   - Support basic regex patterns

3. **Search Results Display**
   - Filter session list to show only matching sessions
   - Show match count per session
   - Indicate which part matched (title vs content)
   - Allow navigation through results with arrow keys

### Technical Requirements
1. **Database Layer**
   - Add search methods to `SessionRepository`
   - Implement SQL queries with LIKE or FTS (Full Text Search)
   - Consider SQLite FTS5 extension for better performance
   - Index message content for faster searches

2. **Service Layer**
   - Create `SearchService` with search logic
   - Cache recent searches for performance
   - Implement search result ranking

3. **UI Updates**
   - Add `SearchMode` to `App` state
   - Create search input widget
   - Update keybinding handlers
   - Modify session list rendering for filtered view

## Implementation Steps

1. **Backend Implementation**
   ```rust
   // In repositories/session_repository.rs
   pub async fn search_sessions(&self, query: &str) -> Result<Vec<Session>>;
   pub async fn search_messages(&self, query: &str) -> Result<Vec<(Session, Message)>>;
   ```

2. **Search Service**
   ```rust
   // New file: services/search_service.rs
   pub struct SearchService {
       session_repo: Arc<SessionRepository>,
       message_repo: Arc<MessageRepository>,
   }
   
   impl SearchService {
       pub async fn search(&self, query: &str) -> Result<SearchResults>;
   }
   ```

3. **UI Integration**
   - Add search mode to `app.rs`
   - Create search widget in `ui.rs`
   - Update event handling in `events.rs`

## Testing Requirements
- Unit tests for search queries
- Integration tests for full search flow
- Performance tests with large datasets (1000+ sessions)
- UI tests for search interaction

## Acceptance Criteria
- [ ] Users can trigger search with `/` key
- [ ] Search finds matches in both titles and content
- [ ] Results update in real-time as user types
- [ ] Search is performant (<100ms for 1000 sessions)
- [ ] Escape key exits search mode
- [ ] Search preserves current session selection

## Future Enhancements
- Advanced search syntax (AND, OR, NOT)
- Search history
- Saved searches
- Search within current session only