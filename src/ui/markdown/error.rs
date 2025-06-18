use std::fmt;

/// Errors that can occur during markdown rendering
#[derive(Debug, Clone)]
pub enum MarkdownError {
    /// Error parsing markdown content
    ParseError(String),
    /// Error during syntax highlighting
    HighlightError(String),
    /// Error loading or applying themes
    ThemeError(String),
    /// Unsupported language for syntax highlighting
    UnsupportedLanguage(String),
    /// Cache-related errors
    CacheError(String),
    /// Generic rendering error
    RenderError(String),
}

impl fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkdownError::ParseError(msg) => write!(f, "Markdown parse error: {}", msg),
            MarkdownError::HighlightError(msg) => write!(f, "Syntax highlighting error: {}", msg),
            MarkdownError::ThemeError(msg) => write!(f, "Theme error: {}", msg),
            MarkdownError::UnsupportedLanguage(lang) => write!(f, "Unsupported language: {}", lang),
            MarkdownError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            MarkdownError::RenderError(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

impl std::error::Error for MarkdownError {}

/// Result type for markdown operations
pub type MarkdownResult<T> = Result<T, MarkdownError>;

/// Provides graceful error handling for markdown rendering
pub trait ErrorRecovery {
    /// Create a fallback representation when markdown rendering fails
    fn create_fallback(content: &str, error: &MarkdownError) -> ratatui::text::Text<'static>;
    
    /// Determine if an error should be shown to the user
    fn should_display_error(error: &MarkdownError) -> bool;
    
    /// Create a user-friendly error message
    fn user_friendly_message(error: &MarkdownError) -> String;
}

pub struct DefaultErrorRecovery;

impl ErrorRecovery for DefaultErrorRecovery {
    fn create_fallback(content: &str, error: &MarkdownError) -> ratatui::text::Text<'static> {
        use ratatui::text::{Line, Span, Text};
        use ratatui::style::{Color, Style};
        
        let mut lines = vec![];
        
        // Add subtle error indicator if appropriate
        if Self::should_display_error(error) {
            lines.push(Line::from(Span::styled(
                format!("⚠ {}", Self::user_friendly_message(error)),
                Style::default().fg(Color::Yellow)
            )));
            lines.push(Line::from(""));
        }
        
        // Add original content as plain text
        for line in content.lines() {
            lines.push(Line::from(line.to_string()));
        }
        
        Text::from(lines)
    }
    
    fn should_display_error(error: &MarkdownError) -> bool {
        match error {
            MarkdownError::UnsupportedLanguage(_) => false, // Common, don't show
            MarkdownError::ParseError(_) => true,
            MarkdownError::HighlightError(_) => false, // Fallback gracefully
            MarkdownError::ThemeError(_) => true,
            MarkdownError::CacheError(_) => false, // Internal issue
            MarkdownError::RenderError(_) => true,
        }
    }
    
    fn user_friendly_message(error: &MarkdownError) -> String {
        match error {
            MarkdownError::ParseError(_) => "Markdown formatting issue".to_string(),
            MarkdownError::HighlightError(_) => "Syntax highlighting unavailable".to_string(),
            MarkdownError::ThemeError(_) => "Theme loading failed".to_string(),
            MarkdownError::UnsupportedLanguage(lang) => format!("Language '{}' not supported", lang),
            MarkdownError::CacheError(_) => "Rendering cache issue".to_string(),
            MarkdownError::RenderError(_) => "Rendering failed".to_string(),
        }
    }
}