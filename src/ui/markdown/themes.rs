use ratatui::style::{Color, Style, Modifier};
use std::collections::HashMap;

/// Types of tokens that can be styled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Programming language tokens
    Keyword,
    String,
    Comment,
    Function,
    Variable,
    Type,
    Number,
    Operator,
    Delimiter,
    Error,
    
    // Markdown-specific tokens
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
    Emphasis,
    Strong,
    Code,
    CodeBlock,
    Link,
    LinkUrl,
    Quote,
    ListMarker,
    TableHeader,
    TableCell,
    
    // Default styling
    Text,
    Background,
}

/// Theme interface for customizable styling
pub trait MarkdownTheme: Send + Sync {
    /// Get style for a specific token type
    fn get_style_for_token(&self, token_type: TokenType) -> Style;
    
    /// Get style for heading at specific level (1-6)
    fn get_heading_style(&self, level: u8) -> Style {
        match level {
            1 => self.get_style_for_token(TokenType::Heading1),
            2 => self.get_style_for_token(TokenType::Heading2),
            3 => self.get_style_for_token(TokenType::Heading3),
            4 => self.get_style_for_token(TokenType::Heading4),
            5 => self.get_style_for_token(TokenType::Heading5),
            6 => self.get_style_for_token(TokenType::Heading6),
            _ => self.get_style_for_token(TokenType::Text),
        }
    }
    
    /// Get the background style for code blocks
    fn get_code_block_style(&self) -> Style;
    
    /// Get the theme name for identification
    fn name(&self) -> &str;
    
    /// Check if theme supports true color
    fn supports_true_color(&self) -> bool {
        true
    }
    
    /// Get fallback colors for limited terminals
    fn get_fallback_style(&self, token_type: TokenType) -> Style;
}

/// Default theme implementation using a color map
pub struct DefaultTheme {
    name: String,
    styles: HashMap<TokenType, Style>,
    fallback_styles: HashMap<TokenType, Style>,
    code_block_style: Style,
}

impl DefaultTheme {
    pub fn new(name: String, styles: HashMap<TokenType, Style>) -> Self {
        let fallback_styles = Self::create_fallback_styles(&styles);
        let code_block_style = Style::default().bg(Color::Rgb(40, 44, 52));
        
        Self {
            name,
            styles,
            fallback_styles,
            code_block_style,
        }
    }
    
    fn create_fallback_styles(styles: &HashMap<TokenType, Style>) -> HashMap<TokenType, Style> {
        let mut fallback = HashMap::new();
        
        for (token_type, style) in styles {
            let fallback_style = match token_type {
                TokenType::Keyword => Style::default().fg(Color::Blue),
                TokenType::String => Style::default().fg(Color::Green),
                TokenType::Comment => Style::default().fg(Color::Gray),
                TokenType::Function => Style::default().fg(Color::Yellow),
                TokenType::Type => Style::default().fg(Color::Cyan),
                TokenType::Number => Style::default().fg(Color::Magenta),
                TokenType::Heading1 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                TokenType::Heading2 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                TokenType::Strong => Style::default().add_modifier(Modifier::BOLD),
                TokenType::Emphasis => Style::default().add_modifier(Modifier::ITALIC),
                TokenType::Code => Style::default().fg(Color::Cyan),
                TokenType::Link => Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
                _ => style.clone(),
            };
            
            fallback.insert(*token_type, fallback_style);
        }
        
        fallback
    }
}

impl MarkdownTheme for DefaultTheme {
    fn get_style_for_token(&self, token_type: TokenType) -> Style {
        self.styles.get(&token_type)
            .copied()
            .unwrap_or_else(|| Style::default())
    }
    
    fn get_code_block_style(&self) -> Style {
        self.code_block_style
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn get_fallback_style(&self, token_type: TokenType) -> Style {
        self.fallback_styles.get(&token_type)
            .copied()
            .unwrap_or_else(|| Style::default())
    }
}

/// Pre-built theme implementations
pub mod themes {
    use super::*;
    
    /// GitHub Dark theme
    pub fn github_dark() -> DefaultTheme {
        let mut styles = HashMap::new();
        
        // Programming language styles
        styles.insert(TokenType::Keyword, Style::default().fg(Color::Rgb(255, 123, 114))); // Red
        styles.insert(TokenType::String, Style::default().fg(Color::Rgb(165, 214, 167))); // Green
        styles.insert(TokenType::Comment, Style::default().fg(Color::Rgb(106, 115, 125))); // Gray
        styles.insert(TokenType::Function, Style::default().fg(Color::Rgb(220, 220, 170))); // Yellow
        styles.insert(TokenType::Type, Style::default().fg(Color::Rgb(86, 182, 194))); // Cyan
        styles.insert(TokenType::Number, Style::default().fg(Color::Rgb(181, 206, 168))); // Light green
        styles.insert(TokenType::Variable, Style::default().fg(Color::Rgb(212, 212, 212))); // Light gray
        styles.insert(TokenType::Operator, Style::default().fg(Color::Rgb(212, 212, 212))); // Light gray
        
        // Markdown styles
        styles.insert(TokenType::Heading1, Style::default().fg(Color::Rgb(88, 166, 255)).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Heading2, Style::default().fg(Color::Rgb(88, 166, 255)).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Heading3, Style::default().fg(Color::Rgb(88, 166, 255)).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Strong, Style::default().add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Emphasis, Style::default().add_modifier(Modifier::ITALIC));
        styles.insert(TokenType::Code, Style::default().fg(Color::Rgb(255, 123, 114)).bg(Color::Rgb(40, 44, 52)));
        styles.insert(TokenType::Link, Style::default().fg(Color::Rgb(88, 166, 255)).add_modifier(Modifier::UNDERLINED));
        styles.insert(TokenType::Quote, Style::default().fg(Color::Rgb(139, 148, 158)));
        
        DefaultTheme::new("GitHub Dark".to_string(), styles)
    }
    
    /// Monokai theme
    pub fn monokai() -> DefaultTheme {
        let mut styles = HashMap::new();
        
        styles.insert(TokenType::Keyword, Style::default().fg(Color::Rgb(249, 38, 114))); // Pink
        styles.insert(TokenType::String, Style::default().fg(Color::Rgb(230, 219, 116))); // Yellow
        styles.insert(TokenType::Comment, Style::default().fg(Color::Rgb(117, 113, 94))); // Gray
        styles.insert(TokenType::Function, Style::default().fg(Color::Rgb(166, 226, 46))); // Green
        styles.insert(TokenType::Type, Style::default().fg(Color::Rgb(102, 217, 239))); // Cyan
        styles.insert(TokenType::Number, Style::default().fg(Color::Rgb(174, 129, 255))); // Purple
        
        // Markdown
        styles.insert(TokenType::Heading1, Style::default().fg(Color::Rgb(249, 38, 114)).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Strong, Style::default().add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Emphasis, Style::default().add_modifier(Modifier::ITALIC));
        styles.insert(TokenType::Code, Style::default().fg(Color::Rgb(230, 219, 116)));
        styles.insert(TokenType::Link, Style::default().fg(Color::Rgb(102, 217, 239)).add_modifier(Modifier::UNDERLINED));
        
        DefaultTheme::new("Monokai".to_string(), styles)
    }
    
    /// Solarized Dark theme
    pub fn solarized_dark() -> DefaultTheme {
        let mut styles = HashMap::new();
        
        styles.insert(TokenType::Keyword, Style::default().fg(Color::Rgb(220, 50, 47))); // Red
        styles.insert(TokenType::String, Style::default().fg(Color::Rgb(42, 161, 152))); // Cyan
        styles.insert(TokenType::Comment, Style::default().fg(Color::Rgb(88, 110, 117))); // Base01
        styles.insert(TokenType::Function, Style::default().fg(Color::Rgb(38, 139, 210))); // Blue
        styles.insert(TokenType::Type, Style::default().fg(Color::Rgb(181, 137, 0))); // Yellow
        styles.insert(TokenType::Number, Style::default().fg(Color::Rgb(211, 54, 130))); // Magenta
        
        // Markdown
        styles.insert(TokenType::Heading1, Style::default().fg(Color::Rgb(38, 139, 210)).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Strong, Style::default().add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Emphasis, Style::default().add_modifier(Modifier::ITALIC));
        styles.insert(TokenType::Code, Style::default().fg(Color::Rgb(220, 50, 47)));
        styles.insert(TokenType::Link, Style::default().fg(Color::Rgb(38, 139, 210)).add_modifier(Modifier::UNDERLINED));
        
        DefaultTheme::new("Solarized Dark".to_string(), styles)
    }
    
    /// High contrast theme for accessibility
    pub fn high_contrast() -> DefaultTheme {
        let mut styles = HashMap::new();
        
        styles.insert(TokenType::Keyword, Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::String, Style::default().fg(Color::Yellow));
        styles.insert(TokenType::Comment, Style::default().fg(Color::Gray));
        styles.insert(TokenType::Function, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Type, Style::default().fg(Color::Green));
        styles.insert(TokenType::Number, Style::default().fg(Color::Magenta));
        
        // Markdown
        styles.insert(TokenType::Heading1, Style::default().fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
        styles.insert(TokenType::Strong, Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        styles.insert(TokenType::Emphasis, Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC));
        styles.insert(TokenType::Code, Style::default().fg(Color::Yellow).bg(Color::Black));
        styles.insert(TokenType::Link, Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED));
        
        DefaultTheme::new("High Contrast".to_string(), styles)
    }
    
    /// Get all available themes
    pub fn all_themes() -> Vec<Box<dyn MarkdownTheme>> {
        vec![
            Box::new(github_dark()),
            Box::new(monokai()),
            Box::new(solarized_dark()),
            Box::new(high_contrast()),
        ]
    }
    
    /// Get theme by name
    pub fn get_theme(name: &str) -> Option<Box<dyn MarkdownTheme>> {
        match name.to_lowercase().as_str() {
            "github_dark" | "github-dark" => Some(Box::new(github_dark())),
            "monokai" => Some(Box::new(monokai())),
            "solarized_dark" | "solarized-dark" => Some(Box::new(solarized_dark())),
            "high_contrast" | "high-contrast" => Some(Box::new(high_contrast())),
            _ => None,
        }
    }
}