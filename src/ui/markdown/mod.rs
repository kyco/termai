// Core markdown rendering functionality
pub mod error;
pub mod themes;
pub mod parser;
pub mod renderer;
pub mod highlighting;
pub mod cache;
pub mod widget;

// Re-export key types for convenience
pub use error::{MarkdownError, MarkdownResult};
pub use themes::{MarkdownTheme, TokenType};
pub use renderer::{MarkdownRenderer, DefaultMarkdownRenderer, RendererConfig};
pub use parser::{MarkdownParser, MarkdownElement, TextSpan};
pub use highlighting::{SyntaxHighlighter, LanguageDetector};
pub use cache::{MarkdownCache, CacheStats};
pub use widget::{MarkdownWidget, MarkdownDisplay, ScrollableMarkdown, ScrollState};