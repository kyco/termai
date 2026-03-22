use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::util::as_24_bit_terminal_escaped;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Enhanced syntax highlighter with support for 20+ languages
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    language_mappings: HashMap<String, String>,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let theme_set = ThemeSet::load_defaults();
        
        // Choose the best available theme for code highlighting
        let current_theme = if theme_set.themes.contains_key("Solarized (dark)") {
            "Solarized (dark)".to_string()
        } else if theme_set.themes.contains_key("base16-ocean.dark") {
            "base16-ocean.dark".to_string()
        } else if theme_set.themes.contains_key("InspiredGitHub") {
            "InspiredGitHub".to_string()
        } else {
            // Use the first available theme as fallback
            theme_set.themes.keys().next().unwrap_or(&"".to_string()).clone()
        };
        
        let mut highlighter = Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set,
            current_theme,
            language_mappings: HashMap::new(),
        };
        
        highlighter.setup_language_mappings();
        highlighter
    }

    /// Set up language name mappings for better detection
    fn setup_language_mappings(&mut self) {
        let mappings = [
            // Common aliases
            ("rs", "Rust"),
            ("py", "Python"),
            ("js", "JavaScript"),
            ("ts", "TypeScript"),
            ("tsx", "TypeScript"),
            ("jsx", "JavaScript (JSX)"),
            ("cpp", "C++"),
            ("c", "C"),
            ("h", "C"),
            ("hpp", "C++"),
            ("cxx", "C++"),
            ("cc", "C++"),
            ("java", "Java"),
            ("kt", "Kotlin"),
            ("scala", "Scala"),
            ("go", "Go"),
            ("php", "PHP"),
            ("rb", "Ruby"),
            ("swift", "Swift"),
            ("sh", "Bash"),
            ("bash", "Bash"),
            ("zsh", "Bash"),
            ("fish", "fish"),
            ("ps1", "PowerShell"),
            ("psm1", "PowerShell"),
            ("psd1", "PowerShell"),
            ("html", "HTML"),
            ("htm", "HTML"),
            ("xml", "XML"),
            ("css", "CSS"),
            ("scss", "SCSS"),
            ("sass", "Sass"),
            ("less", "LESS"),
            ("json", "JSON"),
            ("yaml", "YAML"),
            ("yml", "YAML"),
            ("toml", "TOML"),
            ("ini", "INI"),
            ("cfg", "INI"),
            ("conf", "Apache Conf"),
            ("sql", "SQL"),
            ("md", "Markdown"),
            ("markdown", "Markdown"),
            ("tex", "LaTeX"),
            ("dockerfile", "Dockerfile"),
            ("makefile", "Makefile"),
            ("cmake", "CMake"),
            ("r", "R"),
            ("m", "Objective-C"),
            ("mm", "Objective-C++"),
            ("pl", "Perl"),
            ("lua", "Lua"),
            ("vim", "VimL"),
            ("asm", "Assembly"),
            ("s", "Assembly"),
            ("clj", "Clojure"),
            ("cljs", "Clojure"),
            ("hs", "Haskell"),
            ("elm", "Elm"),
            ("erl", "Erlang"),
            ("ex", "Elixir"),
            ("exs", "Elixir"),
            ("fs", "F#"),
            ("fsx", "F#"),
            ("ml", "OCaml"),
            ("mli", "OCaml"),
            ("jl", "Julia"),
            ("dart", "Dart"),
            ("nim", "Nim"),
            ("zig", "Zig"),
            ("v", "V"),
        ];

        for (alias, language) in mappings {
            self.language_mappings.insert(alias.to_string(), language.to_string());
        }
    }

    /// Detect language from code content
    pub fn detect_language(&self, code: &str, hint: Option<&str>) -> Option<&SyntaxReference> {
        // First try the hint if provided
        if let Some(hint) = hint {
            if let Some(syntax) = self.find_syntax_by_name(hint) {
                return Some(syntax);
            }
        }

        // Try to detect from content patterns
        if let Some(detected) = self.detect_from_content(code) {
            return Some(detected);
        }

        // Fallback to plain text
        self.syntax_set.find_syntax_by_name("Plain Text")
    }

    /// Find syntax by name with fuzzy matching
    fn find_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        let name_lower = name.to_lowercase();
        
        // Try direct mapping first
        if let Some(mapped_name) = self.language_mappings.get(&name_lower) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_name(mapped_name) {
                return Some(syntax);
            }
        }

        // Try exact match
        if let Some(syntax) = self.syntax_set.find_syntax_by_name(name) {
            return Some(syntax);
        }

        // Try case-insensitive match
        for syntax in self.syntax_set.syntaxes() {
            if syntax.name.to_lowercase() == name_lower {
                return Some(syntax);
            }
        }

        // Try partial match
        for syntax in self.syntax_set.syntaxes() {
            if syntax.name.to_lowercase().contains(&name_lower) {
                return Some(syntax);
            }
        }

        // Try file extension
        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(&name_lower) {
            return Some(syntax);
        }

        None
    }

    /// Detect language from code content patterns
    fn detect_from_content(&self, code: &str) -> Option<&SyntaxReference> {
        let lines: Vec<&str> = code.lines().take(10).collect(); // Check first 10 lines
        let first_line = lines.first().unwrap_or(&"").trim();

        // Shebang detection
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return self.syntax_set.find_syntax_by_name("Python");
            } else if first_line.contains("node") || first_line.contains("nodejs") {
                return self.syntax_set.find_syntax_by_name("JavaScript");
            } else if first_line.contains("bash") || first_line.contains("sh") {
                return self.syntax_set.find_syntax_by_name("Bash");
            } else if first_line.contains("ruby") {
                return self.syntax_set.find_syntax_by_name("Ruby");
            } else if first_line.contains("perl") {
                return self.syntax_set.find_syntax_by_name("Perl");
            }
        }

        // Language-specific patterns
        let code_sample = lines.join(" ");
        
        // Rust patterns
        if code_sample.contains("fn main()") || 
           code_sample.contains("use std::") ||
           code_sample.contains("impl ") ||
           code_sample.contains("match ") {
            return self.syntax_set.find_syntax_by_name("Rust");
        }

        // Python patterns
        if code_sample.contains("def ") || 
           code_sample.contains("import ") ||
           code_sample.contains("from ") ||
           first_line.starts_with("import ") ||
           first_line.starts_with("from ") {
            return self.syntax_set.find_syntax_by_name("Python");
        }

        // JavaScript/TypeScript patterns
        if code_sample.contains("function ") ||
           code_sample.contains("const ") ||
           code_sample.contains("let ") ||
           code_sample.contains("var ") ||
           code_sample.contains("=> ") ||
           code_sample.contains("console.log") {
            if code_sample.contains(": string") || 
               code_sample.contains(": number") ||
               code_sample.contains("interface ") ||
               code_sample.contains("type ") {
                return self.syntax_set.find_syntax_by_name("TypeScript");
            }
            return self.syntax_set.find_syntax_by_name("JavaScript");
        }

        // Java patterns
        if code_sample.contains("public class") ||
           code_sample.contains("public static void main") ||
           code_sample.contains("System.out.println") {
            return self.syntax_set.find_syntax_by_name("Java");
        }

        // C/C++ patterns
        if code_sample.contains("#include") {
            if code_sample.contains("std::") || 
               code_sample.contains("using namespace") ||
               code_sample.contains("cout") {
                return self.syntax_set.find_syntax_by_name("C++");
            }
            return self.syntax_set.find_syntax_by_name("C");
        }

        // Go patterns
        if code_sample.contains("package ") ||
           code_sample.contains("func ") ||
           code_sample.contains("import (") {
            return self.syntax_set.find_syntax_by_name("Go");
        }

        // HTML patterns
        if code_sample.contains("<!DOCTYPE") || 
           code_sample.contains("<html") ||
           code_sample.contains("<head>") ||
           code_sample.contains("<body>") {
            return self.syntax_set.find_syntax_by_name("HTML");
        }

        // CSS patterns
        if code_sample.contains("{") && code_sample.contains("}") && 
           (code_sample.contains(":") && code_sample.contains(";")) {
            return self.syntax_set.find_syntax_by_name("CSS");
        }

        // SQL patterns
        if code_sample.to_uppercase().contains("SELECT ") ||
           code_sample.to_uppercase().contains("INSERT ") ||
           code_sample.to_uppercase().contains("UPDATE ") ||
           code_sample.to_uppercase().contains("DELETE ") ||
           code_sample.to_uppercase().contains("CREATE TABLE") {
            return self.syntax_set.find_syntax_by_name("SQL");
        }

        // JSON pattern
        if ((code_sample.trim().starts_with('{') && code_sample.trim().ends_with('}')) ||
           (code_sample.trim().starts_with('[') && code_sample.trim().ends_with(']')))
            && code_sample.contains("\":") && code_sample.contains(",") {
                return self.syntax_set.find_syntax_by_name("JSON");
            }

        // YAML patterns
        if code_sample.contains("---") || 
           (code_sample.contains(":") && !code_sample.contains("{")) {
            return self.syntax_set.find_syntax_by_name("YAML");
        }

        None
    }

    /// Highlight code with the specified language
    pub fn highlight(&self, code: &str, language: Option<&str>) -> Result<String> {
        let syntax = self.detect_language(code, language)
            .ok_or_else(|| anyhow!("Could not determine language for syntax highlighting"))?;
        
        let theme = self.theme_set.themes.get(&self.current_theme)
            .ok_or_else(|| anyhow!("Theme '{}' not found", self.current_theme))?;

        let mut highlighted_lines = Vec::new();
        let mut highlighter = HighlightLines::new(syntax, theme);

        for line in code.lines() {
            let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &self.syntax_set)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            highlighted_lines.push(escaped);
        }

        Ok(highlighted_lines.join("\n"))
    }

    /// Highlight code with line numbers
    pub fn highlight_with_line_numbers(&self, code: &str, language: Option<&str>, start_line: usize) -> Result<String> {
        let highlighted = self.highlight(code, language)?;
        let lines: Vec<&str> = highlighted.lines().collect();
        
        // Calculate padding for line numbers
        let max_line_num = start_line + lines.len() - 1;
        let padding = max_line_num.to_string().len();

        let mut result = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let line_num = start_line + i;
            let line_num_str = format!("{:width$}", line_num, width = padding);
            result.push(format!("{} │ {}", line_num_str, line));
        }

        Ok(result.join("\n"))
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if self.theme_set.themes.contains_key(theme_name) {
            self.current_theme = theme_name.to_string();
            Ok(())
        } else {
            Err(anyhow!("Theme '{}' not found", theme_name))
        }
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_set.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<&str> {
        self.syntax_set.syntaxes()
            .iter()
            .map(|s| s.name.as_str())
            .collect()
    }

    /// Create a diff highlighting for code changes
    pub fn highlight_diff(&self, old_code: &str, new_code: &str, language: Option<&str>) -> Result<String> {
        let old_lines: Vec<&str> = old_code.lines().collect();
        let new_lines: Vec<&str> = new_code.lines().collect();
        
        let mut result = Vec::new();
        let max_lines = old_lines.len().max(new_lines.len());
        
        for i in 0..max_lines {
            let old_line = old_lines.get(i).copied().unwrap_or("");
            let new_line = new_lines.get(i).copied().unwrap_or("");
            
            if old_line != new_line {
                if !old_line.is_empty() {
                    let highlighted = self.highlight(old_line, language)?;
                    result.push(format!("- {}", highlighted));
                }
                if !new_line.is_empty() {
                    let highlighted = self.highlight(new_line, language)?;
                    result.push(format!("+ {}", highlighted));
                }
            } else if !old_line.is_empty() {
                let highlighted = self.highlight(old_line, language)?;
                result.push(format!("  {}", highlighted));
            }
        }
        
        Ok(result.join("\n"))
    }

    /// Get language statistics for code
    pub fn analyze_code(&self, code: &str, language: Option<&str>) -> CodeAnalysis {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        let non_empty_lines = lines.iter().filter(|line| !line.trim().is_empty()).count();
        let comment_lines = self.count_comment_lines(&lines, language);
        
        let estimated_lang = self.detect_language(code, language).map(|detected| detected.name.clone());

        CodeAnalysis {
            total_lines,
            non_empty_lines,
            comment_lines,
            blank_lines: total_lines - non_empty_lines,
            estimated_language: estimated_lang,
            complexity_estimate: self.estimate_complexity(&lines),
        }
    }

    /// Count comment lines based on language
    fn count_comment_lines(&self, lines: &[&str], language: Option<&str>) -> usize {
        let comment_markers = match language {
            Some(lang) => self.get_comment_markers(lang),
            None => vec!["//", "#", "*", "<!--"], // Common comment markers
        };

        lines.iter()
            .filter(|line| {
                let trimmed = line.trim();
                comment_markers.iter().any(|marker| trimmed.starts_with(marker))
            })
            .count()
    }

    /// Get comment markers for a language
    fn get_comment_markers(&self, language: &str) -> Vec<&'static str> {
        match language.to_lowercase().as_str() {
            "rust" | "javascript" | "typescript" | "java" | "c" | "cpp" | "c++" => vec!["//", "/*"],
            "python" | "ruby" | "bash" | "yaml" => vec!["#"],
            "html" | "xml" => vec!["<!--"],
            "css" => vec!["/*"],
            "sql" => vec!["--", "/*"],
            _ => vec!["//", "#", "/*", "<!--"],
        }
    }

    /// Estimate code complexity
    fn estimate_complexity(&self, lines: &[&str]) -> usize {
        let complexity_keywords = [
            "if", "else", "elif", "for", "while", "match", "case", "switch", 
            "try", "catch", "except", "finally", "async", "await", "fn", "function", "def"
        ];

        lines.iter()
            .map(|line| {
                let line_lower = line.to_lowercase();
                complexity_keywords.iter()
                    .filter(|keyword| line_lower.contains(*keyword))
                    .count()
            })
            .sum()
    }
}

/// Code analysis results
#[derive(Debug, Clone)]
pub struct CodeAnalysis {
    pub total_lines: usize,
    pub non_empty_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub estimated_language: Option<String>,
    pub complexity_estimate: usize,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let highlighter = SyntaxHighlighter::new();

        // Test Rust detection
        let rust_code = "fn main() {\n    println!(\"Hello, world!\");\n}";
        let detected = highlighter.detect_language(rust_code, None);
        assert!(detected.is_some());
        assert_eq!(detected.unwrap().name, "Rust");

        // Test Python detection
        let python_code = "def hello():\n    print(\"Hello, world!\")\n\nif __name__ == \"__main__\":\n    hello()";
        let detected = highlighter.detect_language(python_code, None);
        assert!(detected.is_some());
        assert_eq!(detected.unwrap().name, "Python");

        // Test JavaScript detection
        let js_code = "function hello() {\n    console.log(\"Hello, world!\");\n}";
        let detected = highlighter.detect_language(js_code, None);
        assert!(detected.is_some());
        assert_eq!(detected.unwrap().name, "JavaScript");
    }

    #[test]
    fn test_syntax_highlighting() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    println!(\"Hello!\");\n}";
        
        let result = highlighter.highlight(code, Some("rust"));
        assert!(result.is_ok());
        
        let highlighted = result.unwrap();
        assert!(!highlighted.is_empty());
        assert!(highlighted.contains("main"));
    }

    #[test]
    fn test_line_numbers() {
        let highlighter = SyntaxHighlighter::new();
        let code = "line 1\nline 2\nline 3";
        
        let result = highlighter.highlight_with_line_numbers(code, None, 1);
        assert!(result.is_ok());
        
        let numbered = result.unwrap();
        assert!(numbered.contains("1 │"));
        assert!(numbered.contains("2 │"));
        assert!(numbered.contains("3 │"));
    }

    #[test]
    fn test_code_analysis() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() {\n    // This is a comment\n    if true {\n        println!(\"Hello!\");\n    }\n}";
        
        let analysis = highlighter.analyze_code(code, Some("rust"));
        assert!(analysis.total_lines > 0);
        assert!(analysis.comment_lines > 0);
        assert_eq!(analysis.estimated_language, Some("Rust".to_string()));
    }

    #[test]
    fn test_theme_switching() {
        let mut highlighter = SyntaxHighlighter::new();
        let themes = highlighter.available_themes();
        
        assert!(!themes.is_empty());
        assert!(themes.contains(&"base16-ocean.dark"));
        
        if themes.contains(&"base16-ocean.light") {
            let result = highlighter.set_theme("base16-ocean.light");
            assert!(result.is_ok());
            assert_eq!(highlighter.current_theme, "base16-ocean.light");
        }
    }

    #[test]
    fn test_supported_languages() {
        let highlighter = SyntaxHighlighter::new();
        let languages = highlighter.supported_languages();
        
        assert!(!languages.is_empty());
        assert!(languages.contains(&"Rust"));
        assert!(languages.contains(&"Python"));
        assert!(languages.contains(&"JavaScript"));
    }
}