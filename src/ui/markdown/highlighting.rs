use crate::ui::markdown::{MarkdownError, MarkdownResult};
use ratatui::text::Span;
use ratatui::style::{Color, Style};
use syntect::highlighting::{ThemeSet, HighlightIterator, HighlightState, Highlighter};
use syntect::parsing::{SyntaxSet, ParseState};
use syntect::util::LinesWithEndings;
use std::collections::HashMap;

/// Syntax highlighter using syntect
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    /// Cache for syntax definitions to avoid repeated lookups
    syntax_cache: HashMap<String, usize>, // language -> syntax index
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with default theme
    pub fn new() -> MarkdownResult<Self> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let current_theme = "base16-ocean.dark".to_string();
        let syntax_cache = HashMap::new();
        
        Ok(Self {
            syntax_set,
            theme_set,
            current_theme,
            syntax_cache,
        })
    }
    
    /// Set the syntect theme (different from our markdown theme)
    pub fn set_syntect_theme(&mut self, theme_name: &str) -> MarkdownResult<()> {
        if !self.theme_set.themes.contains_key(theme_name) {
            return Err(MarkdownError::ThemeError(format!("Theme '{}' not found", theme_name)));
        }
        self.current_theme = theme_name.to_string();
        Ok(())
    }
    
    /// Get available syntect themes
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_set.themes.keys().cloned().collect()
    }
    
    /// Check if a language is supported
    pub fn supports_language(&self, language: &str) -> bool {
        self.find_syntax_for_language(language).is_some()
    }
    
    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.syntax_set.syntaxes()
            .iter()
            .flat_map(|syntax| {
                let mut langs = vec![syntax.name.clone()];
                langs.extend(syntax.file_extensions.iter().cloned());
                langs
            })
            .collect()
    }
    
    /// Highlight code and return Ratatui spans
    pub fn highlight_code(&self, code: &str, language: &str) -> MarkdownResult<Vec<Vec<Span<'static>>>> {
        let syntax = self.find_syntax_for_language(language)
            .ok_or_else(|| MarkdownError::UnsupportedLanguage(language.to_string()))?;
        
        let theme = self.theme_set.themes.get(&self.current_theme)
            .ok_or_else(|| MarkdownError::ThemeError("Current theme not found".to_string()))?;
            
        let highlighter = Highlighter::new(theme);
        let mut highlight_state = HighlightState::new(&highlighter, Default::default());
        let mut parse_state = ParseState::new(syntax);
        
        let mut result = Vec::new();
        
        for line in LinesWithEndings::from(code) {
            let line_result = self.highlight_line(line, &mut highlight_state, &mut parse_state, &highlighter)?;
            result.push(line_result);
        }
        
        Ok(result)
    }
    
    /// Highlight a single line and return spans
    fn highlight_line(
        &self,
        line: &str,
        highlight_state: &mut HighlightState,
        parse_state: &mut ParseState,
        highlighter: &Highlighter,
    ) -> MarkdownResult<Vec<Span<'static>>> {
        let ops = parse_state.parse_line(line, &self.syntax_set)
            .map_err(|e| MarkdownError::HighlightError(format!("Parse error: {}", e)))?;
        
        let iter = HighlightIterator::new(highlight_state, &ops, line, highlighter);
        
        let mut spans = Vec::new();
        
        for (style, text) in iter {
            let ratatui_style = self.convert_syntect_style_to_ratatui(style);
            spans.push(Span::styled(text.to_string(), ratatui_style));
        }
        
        Ok(spans)
    }
    
    /// Convert syntect style to ratatui style
    fn convert_syntect_style_to_ratatui(&self, style: syntect::highlighting::Style) -> Style {
        let fg_color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
        
        let mut ratatui_style = Style::default().fg(fg_color);
        
        // Handle text attributes
        if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        
        if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::ITALIC);
        }
        
        if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
            ratatui_style = ratatui_style.add_modifier(ratatui::style::Modifier::UNDERLINED);
        }
        
        ratatui_style
    }
    
    /// Find syntax definition for a given language
    fn find_syntax_for_language(&self, language: &str) -> Option<&syntect::parsing::SyntaxReference> {
        // Try exact name match first
        if let Some(syntax) = self.syntax_set.find_syntax_by_name(language) {
            return Some(syntax);
        }
        
        // Try extension match
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(language) {
            return Some(syntax);
        }
        
        // Try case-insensitive search
        let language_lower = language.to_lowercase();
        
        // Common language aliases
        let alias = match language_lower.as_str() {
            "js" | "javascript" => "JavaScript",
            "ts" | "typescript" => "TypeScript",
            "py" | "python" => "Python",
            "rs" | "rust" => "Rust",
            "sh" | "bash" | "shell" => "Bourne Again Shell (bash)",
            "c" => "C",
            "cpp" | "c++" | "cxx" => "C++",
            "cs" | "csharp" | "c#" => "C#",
            "java" => "Java",
            "go" => "Go",
            "php" => "PHP",
            "rb" | "ruby" => "Ruby",
            "swift" => "Swift",
            "kotlin" | "kt" => "Kotlin",
            "scala" => "Scala",
            "clj" | "clojure" => "Clojure",
            "hs" | "haskell" => "Haskell",
            "ml" | "ocaml" => "OCaml",
            "fs" | "fsharp" | "f#" => "F#",
            "elm" => "Elm",
            "lua" => "Lua",
            "perl" => "Perl",
            "r" => "R",
            "sql" => "SQL",
            "html" => "HTML",
            "css" => "CSS",
            "scss" | "sass" => "Sass",
            "less" => "LESS",
            "xml" => "XML",
            "json" => "JSON",
            "yaml" | "yml" => "YAML",
            "toml" => "TOML",
            "ini" => "INI",
            "dockerfile" | "docker" => "Dockerfile",
            "makefile" | "make" => "Makefile",
            "cmake" => "CMake",
            "gradle" => "Gradle",
            "maven" | "pom" => "Maven POM",
            "tex" | "latex" => "LaTeX",
            "md" | "markdown" => "Markdown",
            "git" | "diff" | "patch" => "Diff",
            "log" => "Log file",
            _ => return None,
        };
        
        self.syntax_set.find_syntax_by_name(alias)
    }
    
    /// Get syntax highlighting statistics for debugging
    pub fn get_stats(&self) -> SyntaxStats {
        SyntaxStats {
            total_syntaxes: self.syntax_set.syntaxes().len(),
            available_themes: self.theme_set.themes.len(),
            current_theme: self.current_theme.clone(),
            cache_size: self.syntax_cache.len(),
        }
    }
}

/// Statistics about the syntax highlighter
#[derive(Debug)]
pub struct SyntaxStats {
    pub total_syntaxes: usize,
    pub available_themes: usize,
    pub current_theme: String,
    pub cache_size: usize,
}

/// Language detection utilities
pub struct LanguageDetector;

impl LanguageDetector {
    /// Attempt to detect language from code content
    pub fn detect_language(code: &str) -> Option<String> {
        let trimmed = code.trim();
        
        // Check for shebangs
        if trimmed.starts_with("#!") {
            if trimmed.contains("python") {
                return Some("python".to_string());
            }
            if trimmed.contains("bash") || trimmed.contains("sh") {
                return Some("bash".to_string());
            }
            if trimmed.contains("node") {
                return Some("javascript".to_string());
            }
        }
        
        // Check for common patterns
        if trimmed.contains("function ") && trimmed.contains("=>") {
            return Some("javascript".to_string());
        }
        
        if trimmed.contains("fn ") && trimmed.contains("->") {
            return Some("rust".to_string());
        }
        
        if trimmed.contains("def ") && trimmed.contains(":") {
            return Some("python".to_string());
        }
        
        if trimmed.contains("package ") && trimmed.contains("import ") {
            return Some("java".to_string());
        }
        
        if trimmed.contains("use ") && trimmed.contains("std::") {
            return Some("rust".to_string());
        }
        
        if trimmed.contains("#include") && trimmed.contains("<iostream>") {
            return Some("cpp".to_string());
        }
        
        if trimmed.contains("#include") && trimmed.contains("<stdio.h>") {
            return Some("c".to_string());
        }
        
        if trimmed.contains("SELECT") || trimmed.contains("FROM") || trimmed.contains("WHERE") {
            return Some("sql".to_string());
        }
        
        if trimmed.starts_with("{") && trimmed.ends_with("}") && trimmed.contains("\"") {
            return Some("json".to_string());
        }
        
        None
    }
    
    /// Get language suggestions based on partial input
    pub fn suggest_languages(partial: &str, highlighter: &SyntaxHighlighter) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let mut suggestions = Vec::new();
        
        for lang in highlighter.supported_languages() {
            if lang.to_lowercase().starts_with(&partial_lower) {
                suggestions.push(lang);
            }
        }
        
        suggestions.sort();
        suggestions.dedup();
        suggestions.truncate(10); // Limit suggestions
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syntax_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert!(highlighter.is_ok());
    }
    
    #[test]
    fn test_language_support() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        assert!(highlighter.supports_language("rust"));
        assert!(highlighter.supports_language("python"));
        assert!(highlighter.supports_language("javascript"));
        assert!(!highlighter.supports_language("nonexistent-language"));
    }
    
    #[test]
    fn test_language_detection() {
        assert_eq!(LanguageDetector::detect_language("fn main() -> () {}"), Some("rust".to_string()));
        assert_eq!(LanguageDetector::detect_language("def hello():\n    pass"), Some("python".to_string()));
        assert_eq!(LanguageDetector::detect_language("function test() { return 42; }"), Some("javascript".to_string()));
        assert_eq!(LanguageDetector::detect_language("{\"key\": \"value\"}"), Some("json".to_string()));
    }
    
    #[test]
    fn test_highlight_rust_code() {
        let highlighter = SyntaxHighlighter::new().unwrap();
        let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let result = highlighter.highlight_code(code, "rust");
        assert!(result.is_ok());
        let spans = result.unwrap();
        assert!(!spans.is_empty());
        assert!(!spans[0].is_empty());
    }
}