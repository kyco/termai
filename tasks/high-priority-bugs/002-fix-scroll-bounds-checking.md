# Task: Fix Scroll Bounds Checking and Navigation Issues

## Priority: High
## Estimated Effort: 1 day
## Dependencies: None
## Files Affected: `src/ui/tui/app.rs`, `src/ui/tui/ui.rs`

## Overview
Fix scroll bounds checking issues where users can scroll infinitely beyond content, leading to blank screens and confusing navigation. Also fix cursor movement bounds checking in visual mode.

## Bug Description
Multiple scroll-related issues exist:
1. `scroll_down()` has no bounds checking, allowing infinite scrolling
2. `scroll_to_bottom()` sets scroll to `usize::MAX` causing overflow issues
3. Cursor movement in visual mode can go out of bounds
4. Mouse scroll doesn't validate content boundaries

## Root Cause Analysis
1. **Missing Bounds Validation**: No checks against actual content size
2. **Overflow Risk**: Using `usize::MAX` as scroll target
3. **Race Conditions**: Content can change while scrolling
4. **Inconsistent Clamping**: Some methods clamp, others don't

## Current Buggy Code
```rust
// In app.rs:216-218
pub fn scroll_down(&mut self) {
    self.scroll_offset += 1; // BUG: No bounds checking
}

// In app.rs:221-224
pub fn scroll_to_bottom(&mut self) {
    self.scroll_offset = usize::MAX; // BUG: Overflow risk
}

// In app.rs:557-610
pub fn move_cursor(&mut self, direction: Direction) {
    // Limited bounds checking, can still go out of bounds
}
```

## Implementation Steps

### 1. Add Content-Aware Scroll Management
```rust
// src/ui/scroll/scroll_manager.rs
#[derive(Debug, Clone)]
pub struct ScrollState {
    pub offset: usize,
    pub viewport_height: usize,
    pub content_height: usize,
    pub smooth_scroll: bool,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            viewport_height: 0,
            content_height: 0,
            smooth_scroll: false,
        }
    }
    
    pub fn update_content_size(&mut self, content_height: usize, viewport_height: usize) {
        self.content_height = content_height;
        self.viewport_height = viewport_height;
        self.clamp_offset();
    }
    
    pub fn scroll_up(&mut self, lines: usize) -> bool {
        let old_offset = self.offset;
        self.offset = self.offset.saturating_sub(lines);
        old_offset != self.offset
    }
    
    pub fn scroll_down(&mut self, lines: usize) -> bool {
        let old_offset = self.offset;
        self.offset = self.offset.saturating_add(lines);
        self.clamp_offset();
        old_offset != self.offset
    }
    
    pub fn scroll_to_top(&mut self) -> bool {
        let old_offset = self.offset;
        self.offset = 0;
        old_offset != self.offset
    }
    
    pub fn scroll_to_bottom(&mut self) -> bool {
        let old_offset = self.offset;
        self.offset = self.max_scroll_offset();
        old_offset != self.offset
    }
    
    pub fn scroll_page_up(&mut self) -> bool {
        let page_size = self.viewport_height.saturating_sub(1);
        self.scroll_up(page_size)
    }
    
    pub fn scroll_page_down(&mut self) -> bool {
        let page_size = self.viewport_height.saturating_sub(1);
        self.scroll_down(page_size)
    }
    
    pub fn can_scroll_up(&self) -> bool {
        self.offset > 0
    }
    
    pub fn can_scroll_down(&self) -> bool {
        self.offset < self.max_scroll_offset()
    }
    
    pub fn max_scroll_offset(&self) -> usize {
        if self.content_height <= self.viewport_height {
            0
        } else {
            self.content_height - self.viewport_height
        }
    }
    
    pub fn clamp_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        self.offset = self.offset.min(max_offset);
    }
    
    pub fn scroll_percentage(&self) -> f64 {
        let max_offset = self.max_scroll_offset();
        if max_offset == 0 {
            0.0
        } else {
            self.offset as f64 / max_offset as f64
        }
    }
    
    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.offset;
        let end = (self.offset + self.viewport_height).min(self.content_height);
        (start, end)
    }
    
    pub fn is_at_top(&self) -> bool {
        self.offset == 0
    }
    
    pub fn is_at_bottom(&self) -> bool {
        self.offset >= self.max_scroll_offset()
    }
}
```

### 2. Update App with Proper Scroll Management
```rust
// In app.rs
use crate::ui::scroll::scroll_manager::ScrollState;

pub struct App {
    // Replace scroll_offset with scroll manager
    pub chat_scroll: ScrollState,
    pub session_scroll: ScrollState,
    // ... other fields
}

impl Default for App {
    fn default() -> Self {
        Self {
            chat_scroll: ScrollState::new(),
            session_scroll: ScrollState::new(),
            // ... other fields
        }
    }
}

impl App {
    pub fn update_chat_viewport(&mut self, viewport_height: usize) {
        let content_height = self.get_chat_content_height();
        self.chat_scroll.update_content_size(content_height, viewport_height);
    }
    
    pub fn update_session_viewport(&mut self, viewport_height: usize) {
        let content_height = self.sessions.len();
        self.session_scroll.update_content_size(content_height, viewport_height);
    }
    
    fn get_chat_content_height(&self) -> usize {
        if self.is_in_visual_mode() {
            self.content_cache.get_lines().len()
        } else if let Some(session) = self.current_session() {
            // Calculate rendered height
            let mut height = 0;
            for message in &session.messages {
                if message.role != Role::System {
                    height += 1; // Role header
                    height += message.content.lines().count();
                    height += 2; // Separators
                }
            }
            height
        } else {
            0
        }
    }
    
    // Updated scroll methods
    pub fn scroll_up(&mut self) {
        if self.chat_scroll.scroll_up(1) {
            // Scroll happened, may need to update UI
        }
    }
    
    pub fn scroll_down(&mut self) {
        if self.chat_scroll.scroll_down(1) {
            // Scroll happened, may need to update UI
        }
    }
    
    pub fn scroll_to_bottom(&mut self) {
        if self.chat_scroll.scroll_to_bottom() {
            // Auto-scroll after new message
        }
    }
    
    pub fn scroll_page_up(&mut self) {
        self.chat_scroll.scroll_page_up();
    }
    
    pub fn scroll_page_down(&mut self) {
        self.chat_scroll.scroll_page_down();
    }
    
    // Session scrolling
    pub fn session_scroll_up(&mut self) {
        self.session_scroll.scroll_up(1);
    }
    
    pub fn session_scroll_down(&mut self) {
        self.session_scroll.scroll_down(1);
    }
    
    // Remove old method - replaced by scroll manager
    pub fn clamp_scroll_to_content_lines(&mut self, content_lines: usize, available_height: usize) {
        // This is now handled automatically by ScrollState
        self.chat_scroll.update_content_size(content_lines, available_height);
    }
}
```

### 3. Add Safe Cursor Movement for Visual Mode
```rust
// In app.rs
impl App {
    pub fn move_cursor(&mut self, direction: Direction) {
        if !self.is_in_visual_mode() {
            return;
        }
        
        let content_lines = self.content_cache.get_lines();
        if content_lines.is_empty() {
            self.cursor_position = CursorPosition { line: 0, column: 0 };
            return;
        }

        let max_line = content_lines.len().saturating_sub(1);
        let old_position = self.cursor_position.clone();
        
        match direction {
            Direction::Up => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    self.clamp_cursor_to_line(content_lines);
                }
            }
            Direction::Down => {
                if self.cursor_position.line < max_line {
                    self.cursor_position.line += 1;
                    self.clamp_cursor_to_line(content_lines);
                }
            }
            Direction::Left => {
                self.move_cursor_left(content_lines);
            }
            Direction::Right => {
                self.move_cursor_right(content_lines, max_line);
            }
        }
        
        // Ensure cursor is visible by adjusting scroll
        self.ensure_cursor_visible();
        
        // Update selection based on cursor movement
        self.update_selection_from_cursor_movement(old_position);
    }
    
    fn clamp_cursor_to_line(&mut self, content_lines: &VecDeque<CachedLine>) {
        if let Some(line) = content_lines.get(self.cursor_position.line) {
            let line_len = line.content.len();
            self.cursor_position.column = self.cursor_position.column.min(line_len);
        } else {
            // Line doesn't exist, move to last valid line
            if !content_lines.is_empty() {
                self.cursor_position.line = content_lines.len() - 1;
                if let Some(line) = content_lines.get(self.cursor_position.line) {
                    self.cursor_position.column = line.content.len();
                }
            } else {
                self.cursor_position = CursorPosition { line: 0, column: 0 };
            }
        }
    }
    
    fn move_cursor_left(&mut self, content_lines: &VecDeque<CachedLine>) {
        if self.cursor_position.column > 0 {
            self.cursor_position.column -= 1;
        } else if self.cursor_position.line > 0 {
            // Move to end of previous line
            self.cursor_position.line -= 1;
            if let Some(line) = content_lines.get(self.cursor_position.line) {
                self.cursor_position.column = line.content.len();
            }
        }
    }
    
    fn move_cursor_right(&mut self, content_lines: &VecDeque<CachedLine>, max_line: usize) {
        if let Some(line) = content_lines.get(self.cursor_position.line) {
            if self.cursor_position.column < line.content.len() {
                self.cursor_position.column += 1;
            } else if self.cursor_position.line < max_line {
                // Move to beginning of next line
                self.cursor_position.line += 1;
                self.cursor_position.column = 0;
            }
        }
    }
    
    fn ensure_cursor_visible(&mut self) {
        let cursor_line = self.cursor_position.line;
        let (visible_start, visible_end) = self.chat_scroll.visible_range();
        
        // If cursor is above visible area, scroll up
        if cursor_line < visible_start {
            let scroll_amount = visible_start - cursor_line;
            self.chat_scroll.scroll_up(scroll_amount);
        }
        // If cursor is below visible area, scroll down
        else if cursor_line >= visible_end {
            let scroll_amount = cursor_line - visible_end + 1;
            self.chat_scroll.scroll_down(scroll_amount);
        }
    }
    
    fn update_selection_from_cursor_movement(&mut self, old_position: CursorPosition) {
        match self.selection_mode {
            SelectionMode::Visual => {
                if self.selection.is_none() {
                    // Start selection from old position
                    self.selection = Some(TextSelection {
                        start: old_position,
                        end: self.cursor_position.clone(),
                    });
                } else if let Some(ref mut selection) = self.selection {
                    // Update end position
                    selection.end = self.cursor_position.clone();
                }
            }
            SelectionMode::VisualLine => {
                self.update_line_selection();
            }
            SelectionMode::None => {
                // No selection to update
            }
        }
    }
    
    fn update_line_selection(&mut self) {
        let content_lines = self.content_cache.get_lines();
        if let Some(line) = content_lines.get(self.cursor_position.line) {
            self.selection = Some(TextSelection {
                start: CursorPosition { 
                    line: self.cursor_position.line, 
                    column: 0 
                },
                end: CursorPosition { 
                    line: self.cursor_position.line, 
                    column: line.content.len()
                },
            });
        }
    }
}
```

### 4. Add Mouse Scroll Bounds Checking
```rust
// In app.rs
impl App {
    pub fn handle_mouse_scroll(&mut self, x: u16, y: u16, direction: ScrollDirection) {
        // Check if scroll is in session list area
        if self.is_point_in_session_area(x, y) {
            match direction {
                ScrollDirection::Up => self.session_scroll_up(),
                ScrollDirection::Down => self.session_scroll_down(),
            }
        }
        // Check if scroll is in chat area
        else if self.is_point_in_chat_area(x, y) {
            match direction {
                ScrollDirection::Up => {
                    if self.chat_scroll.can_scroll_up() {
                        self.scroll_up();
                    }
                }
                ScrollDirection::Down => {
                    if self.chat_scroll.can_scroll_down() {
                        self.scroll_down();
                    }
                }
            }
        }
    }
    
    fn is_point_in_session_area(&self, x: u16, y: u16) -> bool {
        self.session_list_area.x <= x 
            && x < self.session_list_area.x + self.session_list_area.width
            && self.session_list_area.y <= y 
            && y < self.session_list_area.y + self.session_list_area.height
    }
    
    fn is_point_in_chat_area(&self, x: u16, y: u16) -> bool {
        self.chat_area.x <= x 
            && x < self.chat_area.x + self.chat_area.width
            && self.chat_area.y <= y 
            && y < self.chat_area.y + self.chat_area.height
    }
}
```

### 5. Update UI Rendering with Scroll Information
```rust
// In ui.rs
use crate::ui::scroll::scroll_manager::ScrollState;

pub fn draw_chat_area(f: &mut Frame, app: &mut App, area: Rect) {
    // Update viewport size
    app.update_chat_viewport(area.height as usize);
    
    let messages = if let Some(session) = app.current_session() {
        &session.messages
    } else {
        return;
    };
    
    // Get visible content range
    let (start, end) = app.chat_scroll.visible_range();
    
    // Render messages within visible range
    let visible_content = render_messages_in_range(messages, start, end, app);
    
    // Create scrollable widget
    let mut chat_widget = Paragraph::new(visible_content)
        .block(create_chat_block(app))
        .wrap(Wrap { trim: true });
    
    // Add scroll indicators
    if !app.chat_scroll.is_at_top() {
        // Show up arrow or indicator
    }
    if !app.chat_scroll.is_at_bottom() {
        // Show down arrow or indicator
    }
    
    f.render_widget(chat_widget, area);
    
    // Draw scrollbar if needed
    if app.chat_scroll.content_height > app.chat_scroll.viewport_height {
        draw_scrollbar(f, &app.chat_scroll, area);
    }
}

fn draw_scrollbar(f: &mut Frame, scroll: &ScrollState, area: Rect) {
    if area.width == 0 || area.height <= 2 {
        return; // Not enough space for scrollbar
    }
    
    let scrollbar_area = Rect {
        x: area.x + area.width - 1,
        y: area.y + 1,
        width: 1,
        height: area.height - 2,
    };
    
    let scrollbar_height = scrollbar_area.height as usize;
    let content_height = scroll.content_height;
    let viewport_height = scroll.viewport_height;
    
    if content_height <= viewport_height {
        return; // No scrolling needed
    }
    
    // Calculate thumb position and size
    let thumb_size = ((viewport_height as f64 / content_height as f64) * scrollbar_height as f64)
        .max(1.0) as usize;
    let thumb_position = ((scroll.offset as f64 / content_height as f64) * scrollbar_height as f64) as usize;
    
    // Draw scrollbar track
    let track_style = Style::default().fg(Color::DarkGray);
    for y in 0..scrollbar_height {
        if y >= thumb_position && y < thumb_position + thumb_size {
            // Draw thumb
            let thumb_char = "█";
            let thumb_style = Style::default().fg(Color::White);
            f.render_widget(
                Paragraph::new(thumb_char).style(thumb_style),
                Rect {
                    x: scrollbar_area.x,
                    y: scrollbar_area.y + y as u16,
                    width: 1,
                    height: 1,
                }
            );
        } else {
            // Draw track
            let track_char = "░";
            f.render_widget(
                Paragraph::new(track_char).style(track_style),
                Rect {
                    x: scrollbar_area.x,
                    y: scrollbar_area.y + y as u16,
                    width: 1,
                    height: 1,
                }
            );
        }
    }
}

fn render_messages_in_range(
    messages: &[Message], 
    start: usize, 
    end: usize,
    app: &App
) -> Text {
    // Implementation to render only the visible portion of messages
    // This optimizes rendering for large conversations
    let mut lines = Vec::new();
    let mut current_line = 0;
    
    for message in messages.iter().filter(|m| m.role != Role::System) {
        // Add role header
        if current_line >= start && current_line < end {
            let role_text = match message.role {
                Role::User => "👤 You:",
                Role::Assistant => "🤖 AI:",
                Role::System => "⚙️ System:",
            };
            lines.push(Line::from(role_text));
        }
        current_line += 1;
        
        // Add message content
        for content_line in message.content.lines() {
            if current_line >= start && current_line < end {
                lines.push(Line::from(content_line));
            }
            current_line += 1;
            
            if current_line >= end {
                break;
            }
        }
        
        // Add separator
        if current_line >= start && current_line < end {
            lines.push(Line::from(""));
        }
        current_line += 1;
        
        if current_line >= end {
            break;
        }
    }
    
    Text::from(lines)
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_bounds() {
        let mut scroll = ScrollState::new();
        scroll.update_content_size(100, 20);
        
        // Test scrolling down
        assert!(scroll.scroll_down(10));
        assert_eq!(scroll.offset, 10);
        
        // Test scrolling to bottom
        assert!(scroll.scroll_to_bottom());
        assert_eq!(scroll.offset, 80); // 100 - 20
        
        // Test can't scroll beyond bottom
        assert!(!scroll.scroll_down(10));
        assert_eq!(scroll.offset, 80);
        
        // Test scrolling up
        assert!(scroll.scroll_up(10));
        assert_eq!(scroll.offset, 70);
        
        // Test scrolling to top
        assert!(scroll.scroll_to_top());
        assert_eq!(scroll.offset, 0);
        
        // Test can't scroll above top
        assert!(!scroll.scroll_up(10));
        assert_eq!(scroll.offset, 0);
    }
    
    #[test]
    fn test_scroll_with_small_content() {
        let mut scroll = ScrollState::new();
        scroll.update_content_size(10, 20); // Content smaller than viewport
        
        // Should not be able to scroll
        assert!(!scroll.scroll_down(5));
        assert!(!scroll.scroll_up(5));
        assert_eq!(scroll.offset, 0);
        assert_eq!(scroll.max_scroll_offset(), 0);
    }
    
    #[test]
    fn test_cursor_bounds_checking() {
        let mut app = App::new();
        app.enter_visual_mode();
        
        // Set up content cache with known content
        // ... (would need test helper to create cache)
        
        // Test cursor movement bounds
        app.cursor_position = CursorPosition { line: 0, column: 0 };
        
        // Can't move up from top
        app.move_cursor(Direction::Up);
        assert_eq!(app.cursor_position.line, 0);
        
        // Can't move left from beginning
        app.move_cursor(Direction::Left);
        assert_eq!(app.cursor_position, CursorPosition { line: 0, column: 0 });
    }
    
    #[test]
    fn test_scroll_percentage() {
        let mut scroll = ScrollState::new();
        scroll.update_content_size(100, 20);
        
        assert_eq!(scroll.scroll_percentage(), 0.0);
        
        scroll.scroll_to_bottom();
        assert_eq!(scroll.scroll_percentage(), 1.0);
        
        scroll.offset = 40; // Halfway
        scroll.clamp_offset();
        assert_eq!(scroll.scroll_percentage(), 0.5);
    }
}
```

### Integration Tests
```rust
#[test]
fn test_scroll_with_real_content() {
    let mut app = App::new();
    
    // Create session with known content
    let mut session = Session::new_temporary();
    for i in 0..50 {
        session.add_raw_message(format!("Message {}", i), Role::User);
    }
    app.set_sessions(vec![session]);
    
    // Test scrolling behavior
    app.update_chat_viewport(20);
    
    // Should be able to scroll down
    assert!(app.chat_scroll.can_scroll_down());
    
    // Scroll to bottom
    app.scroll_to_bottom();
    assert!(app.chat_scroll.is_at_bottom());
    
    // Should be able to scroll up
    assert!(app.chat_scroll.can_scroll_up());
}
```

## Acceptance Criteria
- [ ] No infinite scrolling beyond content bounds
- [ ] Cursor movement stays within valid ranges
- [ ] Mouse scrolling respects content boundaries
- [ ] Scrollbar accurately reflects position and content size
- [ ] Page up/down navigation works correctly
- [ ] Visual mode cursor is always visible
- [ ] Scroll position is preserved during content updates
- [ ] Performance is good with large content

## Performance Considerations
- Only render visible content to improve performance
- Use efficient bounds checking algorithms
- Cache content measurements when possible
- Optimize scrollbar calculations

## Future Enhancements
- Smooth scrolling animations
- Horizontal scrolling for wide content
- Minimap for very large conversations
- Search result navigation with scroll
- Bookmark positions in long conversations