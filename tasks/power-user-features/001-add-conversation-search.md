# Task: Add In-Conversation Search Functionality

## Priority: Medium
## Estimated Effort: 2-3 days
## Dependencies: Basic search infrastructure

## Overview
Implement search functionality within individual conversations to help users find specific messages, code snippets, or information within long chat sessions.

## Requirements

### Functional Requirements
1. **Search Interface**
   - Trigger with `Ctrl+F` when in chat view
   - Search bar appears at top of chat area
   - Real-time search as user types
   - Search history (last 10 searches)

2. **Search Features**
   - Case-insensitive search by default
   - Option for case-sensitive (`Ctrl+Shift+F`)
   - Regex support with `/pattern/` syntax
   - Search in code blocks
   - Highlight all matches
   - Navigate between matches with `n`/`N`

3. **Visual Feedback**
   - Highlight current match (bright yellow)
   - Highlight other matches (dim yellow)
   - Match counter (e.g., "3 of 15 matches")
   - Smooth scrolling to matches

### Technical Requirements
1. **Search State Management**
   ```rust
   // In app.rs
   pub struct ConversationSearch {
       query: String,
       matches: Vec<MessageMatch>,
       current_match_index: Option<usize>,
       case_sensitive: bool,
       regex_mode: bool,
   }
   
   pub struct MessageMatch {
       message_index: usize,
       start_offset: usize,
       end_offset: usize,
       context: String,
   }
   ```

2. **Search Implementation**
   ```rust
   impl ConversationSearch {
       pub fn search(&mut self, messages: &[Message], query: &str) -> Result<()>;
       pub fn next_match(&mut self) -> Option<&MessageMatch>;
       pub fn previous_match(&mut self) -> Option<&MessageMatch>;
       pub fn clear(&mut self);
   }
   ```

## Implementation Steps

1. **Add Search Mode to App State**
   ```rust
   // In app.rs
   pub enum Mode {
       Normal,
       Edit,
       Visual,
       ConversationSearch, // New mode
   }
   
   impl App {
       pub fn enter_search_mode(&mut self) {
           self.mode = Mode::ConversationSearch;
           self.search_state = Some(ConversationSearch::new());
       }
   }
   ```

2. **Update Event Handling**
   ```rust
   // In events.rs
   Mode::ConversationSearch => {
       match key.code {
           KeyCode::Char(c) => {
               app.search_state.as_mut().unwrap().add_char(c);
               app.update_search_results();
           }
           KeyCode::Enter | KeyCode::Char('n') => {
               app.next_search_match();
           }
           KeyCode::Char('N') => {
               app.previous_search_match();
           }
           KeyCode::Esc => {
               app.exit_search_mode();
           }
           _ => {}
       }
   }
   ```

3. **Implement Search UI**
   ```rust
   // In ui.rs
   fn draw_conversation_search(f: &mut Frame, app: &App, area: Rect) {
       let search_bar = Paragraph::new(format!("Search: {}", app.search_query()))
           .style(Style::default().fg(Color::Yellow))
           .block(Block::default().borders(Borders::ALL).title("Search"));
       
       let match_info = if let Some(search) = &app.search_state {
           format!("{} of {} matches", 
               search.current_match_index.map_or(0, |i| i + 1),
               search.matches.len()
           )
       } else {
           String::new()
       };
       
       // Draw search bar at top
       let chunks = Layout::default()
           .direction(Direction::Vertical)
           .constraints([Constraint::Length(3), Constraint::Min(0)])
           .split(area);
           
       f.render_widget(search_bar, chunks[0]);
       
       // Draw conversation with highlights
       draw_conversation_with_highlights(f, app, chunks[1]);
   }
   ```

4. **Implement Text Highlighting**
   ```rust
   fn highlight_search_matches(
       text: &str, 
       matches: &[MessageMatch], 
       current_match: Option<usize>
   ) -> Text {
       let mut spans = vec![];
       let mut last_end = 0;
       
       for (i, match_info) in matches.iter().enumerate() {
           // Add text before match
           if match_info.start_offset > last_end {
               spans.push(Span::raw(&text[last_end..match_info.start_offset]));
           }
           
           // Add highlighted match
           let style = if Some(i) == current_match {
               Style::default().bg(Color::Yellow).fg(Color::Black)
           } else {
               Style::default().bg(Color::DarkGray)
           };
           
           spans.push(Span::styled(
               &text[match_info.start_offset..match_info.end_offset],
               style
           ));
           
           last_end = match_info.end_offset;
       }
       
       // Add remaining text
       if last_end < text.len() {
           spans.push(Span::raw(&text[last_end..]));
       }
       
       Text::from(Line::from(spans))
   }
   ```

5. **Add Smooth Scrolling**
   ```rust
   impl App {
       pub fn scroll_to_match(&mut self, match_info: &MessageMatch) {
           let message_position = self.calculate_message_position(match_info.message_index);
           let target_scroll = message_position.saturating_sub(self.chat_viewport_height / 2);
           
           // Smooth scroll animation
           self.animate_scroll_to(target_scroll);
       }
   }
   ```

## Testing Requirements
- Unit tests for search algorithm
- Tests for regex pattern matching
- UI tests for highlighting
- Performance tests with large conversations
- Edge cases (empty search, no matches)

## Acceptance Criteria
- [ ] Ctrl+F opens search in conversation
- [ ] Real-time search updates as user types
- [ ] All matches are highlighted
- [ ] Navigation between matches works
- [ ] Current match is visually distinct
- [ ] Search state persists during session
- [ ] Performance is good for 1000+ messages

## Performance Considerations
- Debounce search input (100ms)
- Limit context shown for matches
- Use incremental search for large texts
- Cache compiled regex patterns

## Future Enhancements
- Search filters (by role, date, code only)
- Export search results
- Search and replace in user messages
- Fuzzy search support
- Search across all sessions