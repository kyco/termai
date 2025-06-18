// Core markdown rendering functionality
#[allow(dead_code)]
pub mod error;
#[allow(dead_code)]
pub mod themes;
#[allow(dead_code)]
pub mod parser;
#[allow(dead_code)]
pub mod renderer;
#[allow(dead_code)]
pub mod highlighting;
#[allow(dead_code)]
pub mod cache;
#[allow(dead_code)]
pub mod widget;

// Re-export key types for convenience
pub use error::{MarkdownError, MarkdownResult};
pub use themes::MarkdownTheme;
pub use renderer::{MarkdownRenderer, DefaultMarkdownRenderer};
pub use widget::MarkdownDisplay;