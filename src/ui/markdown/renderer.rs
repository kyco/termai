use crate::ui::markdown::{MarkdownResult, MarkdownTheme};
use ratatui::text::Text;

/// Core trait for markdown rendering engines
pub trait MarkdownRenderer: Send + Sync {
    /// Render markdown text to Ratatui Text
    fn render(&mut self, markdown: &str) -> MarkdownResult<Text<'static>>;
    
    /// Set the theme for syntax highlighting and styling
    fn set_theme(&mut self, theme: Box<dyn MarkdownTheme>);
    
    /// Get the current theme
    fn current_theme(&self) -> &dyn MarkdownTheme;
    
    /// Check if a language is supported for syntax highlighting
    fn supports_language(&self, language: &str) -> bool;
    
    /// Get list of supported languages
    fn supported_languages(&self) -> Vec<String>;
    
    /// Configure rendering options
    fn configure(&mut self, config: RendererConfig);
    
    /// Get current configuration
    fn config(&self) -> &RendererConfig;
    
    /// Clear internal caches (useful for theme changes)
    fn clear_cache(&mut self);
}

/// Configuration for markdown rendering
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Enable syntax highlighting for code blocks
    pub enable_syntax_highlighting: bool,
    
    /// Show line numbers in code blocks
    pub show_line_numbers: bool,
    
    /// Maximum width for code blocks (None = no limit)
    pub max_code_width: Option<u16>,
    
    /// Maximum height for code blocks (None = no limit) 
    pub max_code_height: Option<u16>,
    
    /// Enable smart quotes conversion
    pub smart_quotes: bool,
    
    /// Enable strikethrough support
    pub enable_strikethrough: bool,
    
    /// Enable table support
    pub enable_tables: bool,
    
    /// Enable task list support
    pub enable_task_lists: bool,
    
    /// Wrap long lines in code blocks
    pub wrap_code_blocks: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            enable_syntax_highlighting: true,
            show_line_numbers: false,
            max_code_width: Some(120),
            max_code_height: Some(50),
            smart_quotes: true,
            enable_strikethrough: true,
            enable_tables: true,
            enable_task_lists: true,
            wrap_code_blocks: false,
        }
    }
}

/// Default markdown renderer implementation
pub struct DefaultMarkdownRenderer {
    parser: crate::ui::markdown::parser::MarkdownParser,
    highlighter: crate::ui::markdown::highlighting::SyntaxHighlighter,
    theme: Box<dyn MarkdownTheme>,
    config: RendererConfig,
    cache: crate::ui::markdown::cache::MarkdownCache,
}

impl DefaultMarkdownRenderer {
    /// Create a new renderer with default theme and configuration
    pub fn new() -> MarkdownResult<Self> {
        let theme = Box::new(crate::ui::markdown::themes::themes::github_dark());
        Self::with_theme(theme)
    }
    
    /// Create a new renderer with specified theme
    pub fn with_theme(theme: Box<dyn MarkdownTheme>) -> MarkdownResult<Self> {
        let parser = crate::ui::markdown::parser::MarkdownParser::new()?;
        let highlighter = crate::ui::markdown::highlighting::SyntaxHighlighter::new()?;
        let config = RendererConfig::default();
        let cache = crate::ui::markdown::cache::MarkdownCache::new(100); // 100 item cache
        
        Ok(Self {
            parser,
            highlighter,
            theme,
            config,
            cache,
        })
    }
    
    /// Create renderer with custom configuration
    pub fn with_config(config: RendererConfig) -> MarkdownResult<Self> {
        let mut renderer = Self::new()?;
        renderer.config = config;
        Ok(renderer)
    }
}

impl MarkdownRenderer for DefaultMarkdownRenderer {
    fn render(&mut self, markdown: &str) -> MarkdownResult<Text<'static>> {
        // Try cache first
        if let Some(cached) = self.cache.get(markdown) {
            return Ok(cached);
        }
        
        // Parse markdown to our internal representation
        let elements = self.parser.parse(markdown)?;
        
        // Convert to Ratatui Text with syntax highlighting
        let mut text_builder = TextBuilder::new(&*self.theme, &self.config);
        
        for element in elements {
            text_builder.add_element(element, &self.highlighter)?;
        }
        
        let result = text_builder.build();
        
        // Cache the result
        self.cache.insert(markdown.to_string(), result.clone());
        
        Ok(result)
    }
    
    fn set_theme(&mut self, theme: Box<dyn MarkdownTheme>) {
        self.theme = theme;
        self.clear_cache(); // Theme change invalidates cache
    }
    
    fn current_theme(&self) -> &dyn MarkdownTheme {
        &*self.theme
    }
    
    fn supports_language(&self, language: &str) -> bool {
        self.highlighter.supports_language(language)
    }
    
    fn supported_languages(&self) -> Vec<String> {
        self.highlighter.supported_languages()
    }
    
    fn configure(&mut self, config: RendererConfig) {
        self.config = config;
        self.clear_cache(); // Config change may affect rendering
    }
    
    fn config(&self) -> &RendererConfig {
        &self.config
    }
    
    fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// Helper for building Ratatui Text from markdown elements
struct TextBuilder<'a> {
    lines: Vec<ratatui::text::Line<'static>>,
    theme: &'a dyn MarkdownTheme,
    config: &'a RendererConfig,
}

impl<'a> TextBuilder<'a> {
    fn new(theme: &'a dyn MarkdownTheme, config: &'a RendererConfig) -> Self {
        Self {
            lines: Vec::new(),
            theme,
            config,
        }
    }
    
    fn add_element(
        &mut self, 
        element: crate::ui::markdown::parser::MarkdownElement,
        highlighter: &crate::ui::markdown::highlighting::SyntaxHighlighter
    ) -> MarkdownResult<()> {
        use crate::ui::markdown::parser::MarkdownElement;
        use ratatui::text::{Line, Span};
        
        match element {
            MarkdownElement::Heading { level, text } => {
                let style = self.theme.get_heading_style(level);
                let prefix = "#".repeat(level as usize) + " ";
                self.lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(text, style)
                ]));
                self.lines.push(Line::from("")); // Add spacing after headings
            }
            
            MarkdownElement::Paragraph { spans } => {
                let line_spans: Vec<Span> = spans.into_iter()
                    .map(|span| self.convert_text_span(span))
                    .collect();
                self.lines.push(Line::from(line_spans));
            }
            
            MarkdownElement::CodeBlock { language, code } => {
                self.add_code_block(code, language.as_deref(), highlighter)?;
            }
            
            MarkdownElement::List { items } => {
                for (i, item) in items.into_iter().enumerate() {
                    // Add list marker
                    let marker = if i < 10 {
                        format!("{}. ", i + 1)
                    } else {
                        "• ".to_string()
                    };
                    
                    let marker_span = Span::styled(
                        marker, 
                        self.theme.get_style_for_token(crate::ui::markdown::themes::TokenType::ListMarker)
                    );
                    
                    // Recursively render list item
                    let mut item_builder = TextBuilder::new(self.theme, self.config);
                    item_builder.add_element(item, highlighter)?;
                    
                    // Combine marker with first line of item
                    if let Some(first_line) = item_builder.lines.first() {
                        let mut spans = vec![marker_span];
                        spans.extend(first_line.spans.clone());
                        self.lines.push(Line::from(spans));
                        
                        // Add remaining lines with proper indentation
                        for line in item_builder.lines.iter().skip(1) {
                            let mut indented_spans = vec![Span::from("   ")]; // 3 spaces indentation
                            indented_spans.extend(line.spans.clone());
                            self.lines.push(Line::from(indented_spans));
                        }
                    }
                }
            }
            
            MarkdownElement::Quote { content } => {
                let quote_style = self.theme.get_style_for_token(crate::ui::markdown::themes::TokenType::Quote);
                for line in content.lines() {
                    self.lines.push(Line::from(vec![
                        Span::styled("│ ", quote_style),
                        Span::styled(line.to_string(), quote_style)
                    ]));
                }
            }
            
            MarkdownElement::HorizontalRule => {
                let rule = "─".repeat(60);
                self.lines.push(Line::from(Span::styled(
                    rule,
                    self.theme.get_style_for_token(crate::ui::markdown::themes::TokenType::Text)
                )));
            }
        }
        
        Ok(())
    }
    
    fn add_code_block(
        &mut self,
        code: String,
        language: Option<&str>,
        highlighter: &crate::ui::markdown::highlighting::SyntaxHighlighter
    ) -> MarkdownResult<()> {
        use ratatui::text::{Line, Span};
        use crate::ui::markdown::themes::TokenType;
        
        if !self.config.enable_syntax_highlighting {
            // Plain code block without highlighting
            let code_style = self.theme.get_style_for_token(TokenType::Code);
            for line in code.lines() {
                self.lines.push(Line::from(Span::styled(line.to_string(), code_style)));
            }
            return Ok(());
        }
        
        // Add language label if present
        if let Some(lang) = language {
            let lang_style = self.theme.get_style_for_token(TokenType::Comment);
            self.lines.push(Line::from(Span::styled(
                format!("┌─ {} ─", lang),
                lang_style
            )));
        } else {
            let border_style = self.theme.get_style_for_token(TokenType::Comment);
            self.lines.push(Line::from(Span::styled("┌─", border_style)));
        }
        
        // Highlight code if language is supported
        if let Some(lang) = language {
            if let Ok(highlighted_lines) = highlighter.highlight_code(&code, lang) {
                for line_spans in highlighted_lines {
                    let mut full_line = vec![Span::styled("│ ", self.theme.get_style_for_token(TokenType::Comment))];
                    full_line.extend(line_spans);
                    self.lines.push(Line::from(full_line));
                }
            } else {
                // Fallback to plain code
                self.add_plain_code_lines(&code);
            }
        } else {
            // No language specified, use plain code
            self.add_plain_code_lines(&code);
        }
        
        // Add bottom border
        let border_style = self.theme.get_style_for_token(TokenType::Comment);
        self.lines.push(Line::from(Span::styled("└─", border_style)));
        
        Ok(())
    }
    
    fn add_plain_code_lines(&mut self, code: &str) {
        use ratatui::text::{Line, Span};
        use crate::ui::markdown::themes::TokenType;
        
        let code_style = self.theme.get_style_for_token(TokenType::Code);
        let border_style = self.theme.get_style_for_token(TokenType::Comment);
        
        for line in code.lines() {
            self.lines.push(Line::from(vec![
                Span::styled("│ ", border_style),
                Span::styled(line.to_string(), code_style)
            ]));
        }
    }
    
    fn convert_text_span(&self, span: crate::ui::markdown::parser::TextSpan) -> ratatui::text::Span<'static> {
        use crate::ui::markdown::parser::TextSpan;
        use crate::ui::markdown::themes::TokenType;
        use ratatui::text::Span;
        
        match span {
            TextSpan::Plain(text) => Span::from(text),
            TextSpan::Strong(text) => Span::styled(text, self.theme.get_style_for_token(TokenType::Strong)),
            TextSpan::Emphasis(text) => Span::styled(text, self.theme.get_style_for_token(TokenType::Emphasis)),
            TextSpan::Code(text) => Span::styled(text, self.theme.get_style_for_token(TokenType::Code)),
            TextSpan::Link { text, url: _ } => Span::styled(text, self.theme.get_style_for_token(TokenType::Link)),
        }
    }
    
    fn build(self) -> ratatui::text::Text<'static> {
        ratatui::text::Text::from(self.lines)
    }
}