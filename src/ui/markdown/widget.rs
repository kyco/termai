use crate::ui::markdown::{MarkdownResult, MarkdownRenderer, DefaultMarkdownRenderer};
use crate::ui::markdown::error::ErrorRecovery;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Ratatui widget for rendering markdown content
pub struct MarkdownWidget<'a> {
    content: &'a str,
    renderer: &'a mut dyn MarkdownRenderer,
    block: Option<Block<'a>>,
    wrap: Option<Wrap>,
    scroll_offset: u16,
    show_scrollbar: bool,
    max_height: Option<u16>,
    error_style: Style,
}

impl<'a> MarkdownWidget<'a> {
    /// Create a new markdown widget
    pub fn new(content: &'a str, renderer: &'a mut dyn MarkdownRenderer) -> Self {
        Self {
            content,
            renderer,
            block: None,
            wrap: None,
            scroll_offset: 0,
            show_scrollbar: false,
            max_height: None,
            error_style: Style::default().fg(Color::Red),
        }
    }
    
    /// Add a block border around the widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    
    /// Enable text wrapping
    pub fn wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }
    
    /// Set scroll offset for long content
    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
        self
    }
    
    /// Show scrollbar for long content
    pub fn scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }
    
    /// Set maximum height (enables scrolling if content is longer)
    pub fn max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }
    
    /// Set style for error messages
    pub fn error_style(mut self, style: Style) -> Self {
        self.error_style = style;
        self
    }
}

impl<'a> Widget for MarkdownWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Try to render markdown content
        let text = match self.renderer.render(self.content) {
            Ok(text) => text,
            Err(error) => {
                // Create fallback text with error indication
                crate::ui::markdown::error::DefaultErrorRecovery::create_fallback(
                    self.content, 
                    &error
                )
            }
        };
        
        // Create paragraph widget with rendered text
        let mut paragraph = Paragraph::new(text);
        
        if let Some(ref block) = self.block {
            paragraph = paragraph.block(block.clone());
        }
        
        if let Some(wrap) = self.wrap {
            paragraph = paragraph.wrap(wrap);
        }
        
        if self.scroll_offset > 0 {
            paragraph = paragraph.scroll((self.scroll_offset, 0));
        }
        
        // Render the paragraph
        paragraph.render(area, buf);
        
        // TODO: Implement scrollbar if requested
        if self.show_scrollbar {
            self.render_scrollbar(area, buf);
        }
    }
}

impl<'a> MarkdownWidget<'a> {
    /// Render scrollbar (placeholder implementation)
    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer) {
        // Simple scrollbar implementation
        if area.width > 0 {
            let scrollbar_x = area.right() - 1;
            let scrollbar_style = Style::default().fg(Color::Gray);
            
            for y in area.top()..area.bottom() {
                if let Some(cell) = buf.cell_mut((scrollbar_x, y)) {
                    cell.set_char('│').set_style(scrollbar_style);
                }
            }
            
            // Show scroll position (simplified)
            if area.height > 2 {
                let thumb_y = area.top() + 1 + (self.scroll_offset as u16 % (area.height - 2));
                if let Some(cell) = buf.cell_mut((scrollbar_x, thumb_y)) {
                    cell.set_char('█').set_style(Style::default().fg(Color::White));
                }
            }
        }
    }
}

/// Convenient widget builder for common markdown display scenarios
pub struct MarkdownDisplay {
    renderer: DefaultMarkdownRenderer,
}

impl MarkdownDisplay {
    /// Create a new markdown display with default renderer
    pub fn new() -> MarkdownResult<Self> {
        let renderer = DefaultMarkdownRenderer::new()?;
        Ok(Self { renderer })
    }
    
    /// Create with specific theme
    pub fn with_theme(theme: Box<dyn crate::ui::markdown::MarkdownTheme>) -> MarkdownResult<Self> {
        let renderer = DefaultMarkdownRenderer::with_theme(theme)?;
        Ok(Self { renderer })
    }
    
    /// Get the underlying renderer for advanced configuration
    pub fn renderer(&self) -> &DefaultMarkdownRenderer {
        &self.renderer
    }
    
    /// Get mutable access to renderer for configuration
    pub fn renderer_mut(&mut self) -> &mut DefaultMarkdownRenderer {
        &mut self.renderer
    }
    
    /// Render markdown content directly to Text
    pub fn render_to_text(&mut self, content: &str) -> MarkdownResult<Text<'static>> {
        self.renderer.render(content)
    }
    
    /// Render a chat message with borders
    pub fn render_chat_message(&mut self, content: &str, title: &str) -> MarkdownResult<(Text<'static>, Block<'static>)> {
        let text = self.render_to_text(content)?;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title.to_string())
            .border_style(Style::default().fg(Color::Gray));
        Ok((text, block))
    }
    
    /// Create a code block widget with syntax highlighting
    /// Note: This method has lifetime constraints - consider using render_code_block instead
    pub fn code_block_widget<'a>(&'a mut self, markdown: &'a str) -> MarkdownWidget<'a> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Code Block")
            .border_style(Style::default().fg(Color::Cyan));
            
        MarkdownWidget::new(markdown, &mut self.renderer)
            .block(block)
    }
    
    /// Render help text with appropriate styling
    pub fn render_help_text(&mut self, content: &str) -> MarkdownResult<(Text<'static>, Block<'static>)> {
        let text = self.render_to_text(content)?;
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .border_style(Style::default().fg(Color::Yellow));
        Ok((text, block))
    }
}

/// Scrollable markdown viewer for long content
pub struct ScrollableMarkdown<'a> {
    content: &'a str,
    renderer: &'a mut dyn MarkdownRenderer,
    scroll_state: ScrollState,
}

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub offset: u16,
    pub content_height: u16,
    pub viewport_height: u16,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            content_height: 0,
            viewport_height: 0,
        }
    }
    
    pub fn scroll_up(&mut self, lines: u16) {
        self.offset = self.offset.saturating_sub(lines);
    }
    
    pub fn scroll_down(&mut self, lines: u16) {
        let max_offset = self.content_height.saturating_sub(self.viewport_height);
        self.offset = (self.offset + lines).min(max_offset);
    }
    
    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
    }
    
    pub fn scroll_to_bottom(&mut self) {
        self.offset = self.content_height.saturating_sub(self.viewport_height);
    }
    
    pub fn can_scroll_up(&self) -> bool {
        self.offset > 0
    }
    
    pub fn can_scroll_down(&self) -> bool {
        self.offset < self.content_height.saturating_sub(self.viewport_height)
    }
    
    pub fn scroll_percentage(&self) -> f32 {
        if self.content_height <= self.viewport_height {
            0.0
        } else {
            self.offset as f32 / (self.content_height - self.viewport_height) as f32
        }
    }
}

impl<'a> ScrollableMarkdown<'a> {
    pub fn new(content: &'a str, renderer: &'a mut dyn MarkdownRenderer) -> Self {
        Self {
            content,
            renderer,
            scroll_state: ScrollState::new(),
        }
    }
    
    pub fn scroll_state(&self) -> &ScrollState {
        &self.scroll_state
    }
    
    pub fn scroll_state_mut(&mut self) -> &mut ScrollState {
        &mut self.scroll_state
    }
    
    pub fn widget(self) -> MarkdownWidget<'a> {
        MarkdownWidget::new(self.content, self.renderer)
            .scroll(self.scroll_state.offset)
            .scrollbar(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_markdown_widget_creation() {
        let mut renderer = DefaultMarkdownRenderer::new().unwrap();
        let widget = MarkdownWidget::new("# Test", &mut renderer);
        // Basic widget creation should work
        assert_eq!(widget.content, "# Test");
    }
    
    #[test]
    fn test_markdown_display() {
        let mut display = MarkdownDisplay::new().unwrap();
        let (text, _block) = display.render_chat_message("# Test", "Test Title").unwrap();
        // Basic display creation should work
        assert!(!text.lines.is_empty());
    }
    
    #[test]
    fn test_scroll_state() {
        let mut state = ScrollState::new();
        state.content_height = 100;
        state.viewport_height = 20;
        
        assert!(state.can_scroll_down());
        assert!(!state.can_scroll_up());
        
        state.scroll_down(10);
        assert_eq!(state.offset, 10);
        assert!(state.can_scroll_up());
        
        state.scroll_to_bottom();
        assert_eq!(state.offset, 80);
        assert!(!state.can_scroll_down());
    }
}