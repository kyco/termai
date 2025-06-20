# Task: Fix Memory Leaks in Visual Mode and Content Caching

## Priority: Critical
## Estimated Effort: 1-2 days
## Dependencies: None
## Files Affected: `src/ui/tui/app.rs`

## Overview
Fix memory leaks caused by unbounded content caching in visual mode, where the entire conversation history is cloned into memory without limits, leading to potential out-of-memory conditions with large conversations.

## Bug Description
In `app.rs:724-759`, the `update_chat_content_cache()` method clones the entire message history into memory every time visual mode is used, without any size limits or cleanup mechanisms. This can cause memory exhaustion with large conversations.

## Root Cause Analysis
1. **Unbounded Caching**: No limits on cache size or content length
2. **Frequent Allocation**: Cache is rebuilt on every visual mode entry
3. **No Cleanup**: Cache persists even when not in visual mode
4. **Inefficient Cloning**: Entire message vector is cloned unnecessarily
5. **String Duplication**: Message content is duplicated as String objects

## Current Buggy Code
```rust
// In app.rs:724-759
pub fn update_chat_content_cache(&mut self) {
    self.chat_content_lines.clear();
    
    // BUG: Clones entire message history into memory
    let messages = if let Some(session) = self.current_session() {
        session.messages.clone() // MEMORY LEAK: Unbounded clone
    } else {
        Vec::new()
    };
    
    // BUG: No size limits on content lines
    for (i, message) in filtered_messages.iter().enumerate() {
        // ... processes all messages without limits
        for line in message.content.lines() {
            self.chat_content_lines.push(line.to_string()); // String allocation for every line
        }
    }
}
```

## Implementation Steps

### 1. Add Content Cache Limits and Configuration
```rust
// src/config/cache_config.rs
#[derive(Debug, Clone)]
pub struct ContentCacheConfig {
    pub max_lines: usize,              // Maximum lines to cache
    pub max_memory_mb: usize,          // Maximum memory usage in MB
    pub max_message_length: usize,     // Maximum length of individual messages
    pub cleanup_threshold: f64,        // When to trigger cleanup (0.8 = 80% full)
    pub enable_compression: bool,      // Whether to compress old content
}

impl Default for ContentCacheConfig {
    fn default() -> Self {
        Self {
            max_lines: 10_000,
            max_memory_mb: 50,
            max_message_length: 100_000,
            cleanup_threshold: 0.8,
            enable_compression: false,
        }
    }
}

// Memory tracking utilities
pub struct MemoryTracker {
    current_bytes: usize,
    max_bytes: usize,
}

impl MemoryTracker {
    pub fn new(max_mb: usize) -> Self {
        Self {
            current_bytes: 0,
            max_bytes: max_mb * 1024 * 1024,
        }
    }
    
    pub fn add_content(&mut self, content: &str) -> bool {
        let content_bytes = content.len();
        if self.current_bytes + content_bytes > self.max_bytes {
            false // Would exceed limit
        } else {
            self.current_bytes += content_bytes;
            true
        }
    }
    
    pub fn remove_content(&mut self, content: &str) {
        self.current_bytes = self.current_bytes.saturating_sub(content.len());
    }
    
    pub fn usage_percent(&self) -> f64 {
        self.current_bytes as f64 / self.max_bytes as f64
    }
}
```

### 2. Implement Efficient Content Cache
```rust
// In app.rs
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct CachedLine {
    content: String,
    message_index: usize,
    line_type: LineType,
}

#[derive(Debug, Clone)]
pub enum LineType {
    RoleHeader,
    MessageContent,
    Separator,
}

pub struct ContentCache {
    lines: VecDeque<CachedLine>,
    memory_tracker: MemoryTracker,
    config: ContentCacheConfig,
    total_messages: usize,
    cached_messages: usize,
}

impl ContentCache {
    pub fn new(config: ContentCacheConfig) -> Self {
        Self {
            lines: VecDeque::new(),
            memory_tracker: MemoryTracker::new(config.max_memory_mb),
            config,
            total_messages: 0,
            cached_messages: 0,
        }
    }
    
    pub fn update_from_messages(&mut self, messages: &[Message]) {
        // Only update if messages have changed
        if messages.len() == self.total_messages {
            return; // No change
        }
        
        self.total_messages = messages.len();
        
        // Determine which messages to cache (recent ones first)
        let visible_messages = self.select_visible_messages(messages);
        
        // Clear and rebuild cache efficiently
        self.rebuild_cache(visible_messages);
    }
    
    fn select_visible_messages<'a>(&self, messages: &'a [Message]) -> Vec<&'a Message> {
        // Filter out system messages first
        let filtered: Vec<_> = messages
            .iter()
            .filter(|msg| msg.role != Role::System)
            .collect();
        
        // If too many messages, take the most recent ones
        if filtered.len() > self.config.max_lines / 4 { // Assume ~4 lines per message on average
            let start_index = filtered.len().saturating_sub(self.config.max_lines / 4);
            filtered[start_index..].to_vec()
        } else {
            filtered
        }
    }
    
    fn rebuild_cache(&mut self, messages: Vec<&Message>) {
        self.lines.clear();
        self.memory_tracker = MemoryTracker::new(self.config.max_memory_mb);
        self.cached_messages = 0;
        
        for (i, message) in messages.iter().enumerate() {
            if !self.add_message_to_cache(i, message) {
                // Hit memory limit, stop caching
                break;
            }
        }
    }
    
    fn add_message_to_cache(&mut self, message_index: usize, message: &Message) -> bool {
        // Check if message is too long
        if message.content.len() > self.config.max_message_length {
            // Add truncated version
            let truncated = self.truncate_message(&message.content);
            return self.add_truncated_message(message_index, message, &truncated);
        }
        
        // Add separator if not first message
        if !self.lines.is_empty() {
            let separator = CachedLine {
                content: String::new(),
                message_index,
                line_type: LineType::Separator,
            };
            if !self.try_add_line(separator) {
                return false;
            }
        }
        
        // Add role header
        let role_prefix = match message.role {
            Role::User => "👤 You:",
            Role::Assistant => "🤖 AI:",
            Role::System => "⚙️ System:",
        };
        
        let header = CachedLine {
            content: role_prefix.to_string(),
            message_index,
            line_type: LineType::RoleHeader,
        };
        
        if !self.try_add_line(header) {
            return false;
        }
        
        // Add message content lines
        for line in message.content.lines() {
            let content_line = CachedLine {
                content: line.to_string(),
                message_index,
                line_type: LineType::MessageContent,
            };
            
            if !self.try_add_line(content_line) {
                return false;
            }
        }
        
        // Add trailing separator
        let separator = CachedLine {
            content: String::new(),
            message_index,
            line_type: LineType::Separator,
        };
        
        if !self.try_add_line(separator) {
            return false;
        }
        
        self.cached_messages += 1;
        true
    }
    
    fn try_add_line(&mut self, line: CachedLine) -> bool {
        // Check memory limit
        if !self.memory_tracker.add_content(&line.content) {
            return false;
        }
        
        // Check line limit
        if self.lines.len() >= self.config.max_lines {
            return false;
        }
        
        self.lines.push_back(line);
        true
    }
    
    fn truncate_message(&self, content: &str) -> String {
        let max_len = self.config.max_message_length;
        if content.len() <= max_len {
            return content.to_string();
        }
        
        // Find a good break point (prefer line boundaries)
        let truncate_point = content.char_indices()
            .take(max_len - 100) // Leave room for truncation notice
            .last()
            .map(|(i, _)| i)
            .unwrap_or(max_len - 100);
        
        // Find nearest line break
        let break_point = content[..truncate_point]
            .rfind('\n')
            .unwrap_or(truncate_point);
        
        format!("{}\n\n[... message truncated ({} more characters) ...]", 
                &content[..break_point], 
                content.len() - break_point)
    }
    
    fn add_truncated_message(&mut self, message_index: usize, message: &Message, truncated: &str) -> bool {
        // Similar to add_message_to_cache but with truncated content
        // Implementation similar to above but using truncated content
        true // Simplified for brevity
    }
    
    pub fn get_lines(&self) -> &VecDeque<CachedLine> {
        &self.lines
    }
    
    pub fn cleanup_if_needed(&mut self) {
        if self.memory_tracker.usage_percent() > self.config.cleanup_threshold {
            self.cleanup_old_content();
        }
    }
    
    fn cleanup_old_content(&mut self) {
        // Remove lines from the front (oldest content) until under threshold
        let target_usage = self.config.cleanup_threshold * 0.7; // Clean to 70%
        
        while self.memory_tracker.usage_percent() > target_usage && !self.lines.is_empty() {
            if let Some(line) = self.lines.pop_front() {
                self.memory_tracker.remove_content(&line.content);
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
        self.memory_tracker = MemoryTracker::new(self.config.max_memory_mb);
        self.total_messages = 0;
        self.cached_messages = 0;
    }
    
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            total_lines: self.lines.len(),
            memory_usage_mb: self.memory_tracker.current_bytes as f64 / (1024.0 * 1024.0),
            memory_usage_percent: self.memory_tracker.usage_percent(),
            cached_messages: self.cached_messages,
            total_messages: self.total_messages,
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_lines: usize,
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
    pub cached_messages: usize,
    pub total_messages: usize,
}
```

### 3. Update App to Use Efficient Cache
```rust
// In app.rs
pub struct App {
    // ... existing fields ...
    
    // Replace chat_content_lines with efficient cache
    content_cache: ContentCache,
    cache_config: ContentCacheConfig,
}

impl Default for App {
    fn default() -> Self {
        let cache_config = ContentCacheConfig::default();
        Self {
            // ... existing fields ...
            content_cache: ContentCache::new(cache_config.clone()),
            cache_config,
        }
    }
}

impl App {
    pub fn update_chat_content_cache(&mut self) {
        // Only update cache if in visual mode or needed for rendering
        if !self.is_in_visual_mode() && !self.needs_content_cache() {
            return;
        }
        
        let messages = if let Some(session) = self.current_session() {
            &session.messages // Use reference instead of cloning
        } else {
            return;
        };
        
        self.content_cache.update_from_messages(messages);
        
        // Cleanup if needed
        self.content_cache.cleanup_if_needed();
    }
    
    fn needs_content_cache(&self) -> bool {
        // Determine if cache is needed for current UI state
        matches!(self.focused_area, FocusedArea::Chat) || self.is_in_visual_mode()
    }
    
    pub fn get_cached_lines(&self) -> Vec<String> {
        // Convert cache to format expected by UI
        self.content_cache
            .get_lines()
            .iter()
            .map(|cached_line| cached_line.content.clone())
            .collect()
    }
    
    pub fn enter_visual_mode(&mut self) {
        if matches!(self.focused_area, FocusedArea::Chat) && !self.is_input_editing() {
            self.selection_mode = SelectionMode::Visual;
            self.selection = None;
            
            // Update cache only when entering visual mode
            self.update_chat_content_cache();
            
            // Validate cursor position with cache
            self.validate_cursor_position();
        }
    }
    
    pub fn exit_visual_mode(&mut self) {
        self.selection_mode = SelectionMode::None;
        self.selection = None;
        
        // Clear cache when exiting visual mode to free memory
        if !self.needs_content_cache() {
            self.content_cache.clear();
        }
    }
    
    fn validate_cursor_position(&mut self) {
        let line_count = self.content_cache.get_lines().len();
        if line_count == 0 {
            self.cursor_position = CursorPosition { line: 0, column: 0 };
            return;
        }
        
        // Ensure cursor is within bounds
        if self.cursor_position.line >= line_count {
            self.cursor_position.line = line_count.saturating_sub(1);
        }
        
        // Ensure column is within line bounds
        if let Some(cached_line) = self.content_cache.get_lines().get(self.cursor_position.line) {
            let line_len = cached_line.content.len();
            if self.cursor_position.column > line_len {
                self.cursor_position.column = line_len;
            }
        }
    }
    
    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;
        let lines = self.content_cache.get_lines();
        
        if lines.is_empty() {
            return None;
        }

        let start = &selection.start;
        let end = &selection.end;
        
        // Ensure bounds are valid
        if start.line >= lines.len() || end.line >= lines.len() {
            return None;
        }
        
        // Ensure start is before end
        let (start, end) = if start.line < end.line || 
            (start.line == end.line && start.column <= end.column) {
            (start, end)
        } else {
            (end, start)
        };

        let mut selected_text = String::new();
        
        if start.line == end.line {
            // Single line selection
            if let Some(line) = lines.get(start.line) {
                let start_col = start.column.min(line.content.len());
                let end_col = end.column.min(line.content.len());
                if start_col < end_col {
                    selected_text = line.content[start_col..end_col].to_string();
                }
            }
        } else {
            // Multi-line selection
            for line_idx in start.line..=end.line {
                if let Some(line) = lines.get(line_idx) {
                    if line_idx == start.line {
                        // First line: from start column to end
                        let start_col = start.column.min(line.content.len());
                        selected_text.push_str(&line.content[start_col..]);
                    } else if line_idx == end.line {
                        // Last line: from beginning to end column
                        let end_col = end.column.min(line.content.len());
                        selected_text.push_str(&line.content[..end_col]);
                    } else {
                        // Middle lines: entire line
                        selected_text.push_str(&line.content);
                    }
                    
                    // Add newline except for the last line
                    if line_idx < end.line {
                        selected_text.push('\n');
                    }
                }
            }
        }

        if selected_text.is_empty() {
            None
        } else {
            Some(selected_text)
        }
    }
    
    pub fn get_cache_stats(&self) -> CacheStats {
        self.content_cache.stats()
    }
}
```

### 4. Add Cache Monitoring and Debug Tools
```rust
// src/ui/tui/debug.rs
use crate::ui::tui::app::App;
use ratatui::{
    widgets::{Block, Borders, Paragraph, Gauge},
    layout::{Layout, Constraint, Direction, Rect},
    style::{Style, Color},
    Frame,
};

pub fn draw_cache_debug_info(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.get_cache_stats();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Memory usage
            Constraint::Length(3), // Line count
            Constraint::Length(3), // Message count
        ])
        .split(area);
    
    // Memory usage gauge
    let memory_percent = (stats.memory_usage_percent * 100.0) as u16;
    let memory_gauge = Gauge::default()
        .block(Block::default()
            .title(format!("Memory: {:.1}MB", stats.memory_usage_mb))
            .borders(Borders::ALL))
        .gauge_style(Style::default().fg(
            if memory_percent > 80 { Color::Red } 
            else if memory_percent > 60 { Color::Yellow } 
            else { Color::Green }
        ))
        .percent(memory_percent);
    
    f.render_widget(memory_gauge, chunks[0]);
    
    // Line count
    let line_info = Paragraph::new(format!("Lines: {}", stats.total_lines))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(line_info, chunks[1]);
    
    // Message count
    let message_info = Paragraph::new(format!(
        "Messages: {}/{}", 
        stats.cached_messages, 
        stats.total_messages
    ))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(message_info, chunks[2]);
}

// Add to settings panel
pub fn show_cache_stats_in_settings(app: &App) {
    let stats = app.get_cache_stats();
    eprintln!("Cache Stats:");
    eprintln!("  Memory: {:.1}MB ({:.1}%)", stats.memory_usage_mb, stats.memory_usage_percent * 100.0);
    eprintln!("  Lines: {}", stats.total_lines);
    eprintln!("  Messages: {}/{}", stats.cached_messages, stats.total_messages);
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new(1); // 1MB limit
        
        // Should accept content under limit
        let small_content = "a".repeat(500_000); // 500KB
        assert!(tracker.add_content(&small_content));
        
        // Should reject content that exceeds limit
        let large_content = "b".repeat(600_000); // 600KB (would exceed 1MB total)
        assert!(!tracker.add_content(&large_content));
        
        // Should track usage correctly
        assert!(tracker.usage_percent() > 0.4);
        assert!(tracker.usage_percent() < 0.6);
    }
    
    #[test]
    fn test_content_cache_limits() {
        let config = ContentCacheConfig {
            max_lines: 100,
            max_memory_mb: 1,
            max_message_length: 1000,
            cleanup_threshold: 0.8,
            enable_compression: false,
        };
        
        let mut cache = ContentCache::new(config);
        
        // Create many large messages
        let mut messages = Vec::new();
        for i in 0..1000 {
            let mut msg = Message::new();
            msg.content = format!("Message {} with lots of content: {}", i, "x".repeat(500));
            msg.role = Role::User;
            messages.push(msg);
        }
        
        cache.update_from_messages(&messages);
        
        // Should respect limits
        assert!(cache.get_lines().len() <= 100);
        assert!(cache.stats().memory_usage_mb <= 1.0);
    }
    
    #[test]
    fn test_cache_cleanup() {
        let config = ContentCacheConfig {
            max_lines: 1000,
            max_memory_mb: 1,
            max_message_length: 10000,
            cleanup_threshold: 0.5, // Trigger cleanup at 50%
            enable_compression: false,
        };
        
        let mut cache = ContentCache::new(config);
        
        // Fill cache to trigger cleanup
        let large_messages: Vec<_> = (0..100).map(|i| {
            let mut msg = Message::new();
            msg.content = format!("Large message {}: {}", i, "x".repeat(10000));
            msg.role = Role::User;
            msg
        }).collect();
        
        cache.update_from_messages(&large_messages);
        
        // Should have cleaned up automatically
        assert!(cache.stats().memory_usage_percent < 0.8);
    }
    
    #[test]
    fn test_visual_mode_memory_efficiency() {
        let mut app = App::new();
        
        // Create session with many messages
        let mut session = Session::new_temporary();
        for i in 0..1000 {
            session.add_raw_message(format!("Message {}: {}", i, "x".repeat(1000)), Role::User);
        }
        app.set_sessions(vec![session]);
        
        // Cache should be empty initially
        assert_eq!(app.content_cache.stats().total_lines, 0);
        
        // Entering visual mode should populate cache
        app.enter_visual_mode();
        assert!(app.content_cache.stats().total_lines > 0);
        
        // Exiting visual mode should clear cache
        app.exit_visual_mode();
        assert_eq!(app.content_cache.stats().total_lines, 0);
    }
}
```

### Memory Leak Tests
```rust
#[test]
fn test_no_memory_leak_in_visual_mode() {
    use std::process;
    
    let mut app = App::new();
    
    // Create large session
    let mut session = Session::new_temporary();
    for i in 0..10000 {
        session.add_raw_message("x".repeat(1000), Role::User);
    }
    app.set_sessions(vec![session]);
    
    // Get initial memory usage
    let initial_memory = get_process_memory();
    
    // Enter/exit visual mode many times
    for _ in 0..100 {
        app.enter_visual_mode();
        app.exit_visual_mode();
    }
    
    // Memory should not grow significantly
    let final_memory = get_process_memory();
    let memory_growth = final_memory - initial_memory;
    
    // Allow some growth but not proportional to iterations
    assert!(memory_growth < initial_memory / 10, 
           "Memory grew too much: {} -> {} (+{})", 
           initial_memory, final_memory, memory_growth);
}

fn get_process_memory() -> usize {
    // Platform-specific memory measurement
    // Implementation would depend on target platform
    0 // Simplified for example
}
```

## Performance Benchmarks
```rust
#[bench]
fn bench_cache_update_large_session(b: &mut Bencher) {
    let mut app = App::new();
    
    let mut session = Session::new_temporary();
    for i in 0..1000 {
        session.add_raw_message(format!("Message {}", i), Role::User);
    }
    app.set_sessions(vec![session]);
    
    b.iter(|| {
        app.update_chat_content_cache();
    });
}

#[bench] 
fn bench_visual_mode_selection(b: &mut Bencher) {
    let mut app = App::new();
    
    // Setup large cached content
    app.enter_visual_mode();
    
    b.iter(|| {
        app.cursor_position.line = 100;
        app.cursor_position.column = 50;
        let _ = app.get_selected_text();
    });
}
```

## Acceptance Criteria
- [ ] Memory usage is bounded by configurable limits
- [ ] Cache automatically cleans up when needed
- [ ] Visual mode performance is acceptable with large conversations
- [ ] No memory leaks when entering/exiting visual mode repeatedly
- [ ] Cache statistics are available for monitoring
- [ ] Truncated messages are handled gracefully
- [ ] Selection works correctly with cached content
- [ ] Memory usage doesn't grow indefinitely over time

## Monitoring and Alerting
- Add cache statistics to debug UI
- Log warnings when memory limits are hit
- Track cache hit/miss ratios
- Monitor cleanup frequency and efficiency

## Future Enhancements
- Implement content compression for older messages
- Add virtual scrolling for very large conversations
- Implement lazy loading of message content
- Add user preferences for cache behavior
- Consider using memory-mapped files for very large caches