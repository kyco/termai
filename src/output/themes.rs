use colored::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Color palette for terminal output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub primary: String,
    pub secondary: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub text: String,
    pub text_dim: String,
    pub background: String,
    pub border: String,
}

/// Syntax highlighting colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub string: String,
    pub comment: String,
    pub function: String,
    pub variable: String,
    pub number: String,
    pub operator: String,
    pub type_name: String,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub description: String,
    pub colors: ColorPalette,
    pub syntax: SyntaxColors,
    pub use_icons: bool,
    pub box_drawing: BoxDrawingStyle,
}

/// Box drawing style options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BoxDrawingStyle {
    Single,
    Double,
    Rounded,
    Thick,
    Minimal,
}

/// Icon sets for different roles and status
#[derive(Debug, Clone)]
pub struct IconSet {
    pub user: &'static str,
    pub assistant: &'static str,
    pub system: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    pub thinking: &'static str,
    pub code: &'static str,
    pub file: &'static str,
    pub directory: &'static str,
}

/// Theme manager for output formatting
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: HashMap<String, Theme>,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            primary: "#7C3AED".to_string(),     // Purple
            secondary: "#3B82F6".to_string(),   // Blue
            success: "#10B981".to_string(),     // Green
            warning: "#F59E0B".to_string(),     // Yellow
            error: "#EF4444".to_string(),       // Red
            info: "#06B6D4".to_string(),        // Cyan
            text: "#F9FAFB".to_string(),        // Light gray
            text_dim: "#9CA3AF".to_string(),    // Dim gray
            background: "#111827".to_string(),   // Dark gray
            border: "#374151".to_string(),       // Medium gray
        }
    }
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            keyword: "#8B5CF6".to_string(),     // Purple
            string: "#10B981".to_string(),      // Green
            comment: "#6B7280".to_string(),     // Gray
            function: "#3B82F6".to_string(),    // Blue
            variable: "#F59E0B".to_string(),    // Yellow
            number: "#EF4444".to_string(),      // Red
            operator: "#06B6D4".to_string(),    // Cyan
            type_name: "#8B5CF6".to_string(),   // Purple
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: "Default TermAI theme with modern colors".to_string(),
            colors: ColorPalette::default(),
            syntax: SyntaxColors::default(),
            use_icons: true,
            box_drawing: BoxDrawingStyle::Rounded,
        }
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_theme: Theme::default(),
            available_themes: HashMap::new(),
        };
        
        manager.load_builtin_themes();
        manager
    }

    /// Load built-in themes
    fn load_builtin_themes(&mut self) {
        // Default theme
        self.available_themes.insert("default".to_string(), Theme::default());

        // Dark theme
        let dark_theme = Theme {
            name: "Dark".to_string(),
            description: "Dark theme optimized for dark terminals".to_string(),
            colors: ColorPalette {
                primary: "#BB86FC".to_string(),
                secondary: "#03DAC6".to_string(),
                success: "#4CAF50".to_string(),
                warning: "#FF9800".to_string(),
                error: "#F44336".to_string(),
                info: "#2196F3".to_string(),
                text: "#FFFFFF".to_string(),
                text_dim: "#AAAAAA".to_string(),
                background: "#121212".to_string(),
                border: "#333333".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#BB86FC".to_string(),
                string: "#4CAF50".to_string(),
                comment: "#888888".to_string(),
                function: "#03DAC6".to_string(),
                variable: "#FF9800".to_string(),
                number: "#F44336".to_string(),
                operator: "#2196F3".to_string(),
                type_name: "#9C27B0".to_string(),
            },
            use_icons: true,
            box_drawing: BoxDrawingStyle::Single,
        };
        self.available_themes.insert("dark".to_string(), dark_theme);

        // Light theme
        let light_theme = Theme {
            name: "Light".to_string(),
            description: "Light theme optimized for light terminals".to_string(),
            colors: ColorPalette {
                primary: "#6366F1".to_string(),
                secondary: "#0891B2".to_string(),
                success: "#059669".to_string(),
                warning: "#D97706".to_string(),
                error: "#DC2626".to_string(),
                info: "#0284C7".to_string(),
                text: "#111827".to_string(),
                text_dim: "#6B7280".to_string(),
                background: "#FFFFFF".to_string(),
                border: "#D1D5DB".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#7C3AED".to_string(),
                string: "#059669".to_string(),
                comment: "#9CA3AF".to_string(),
                function: "#0284C7".to_string(),
                variable: "#D97706".to_string(),
                number: "#DC2626".to_string(),
                operator: "#0891B2".to_string(),
                type_name: "#7C3AED".to_string(),
            },
            use_icons: true,
            box_drawing: BoxDrawingStyle::Single,
        };
        self.available_themes.insert("light".to_string(), light_theme);

        // Minimal theme
        let minimal_theme = Theme {
            name: "Minimal".to_string(),
            description: "Minimal theme with reduced visual elements".to_string(),
            colors: ColorPalette {
                primary: "#FFFFFF".to_string(),
                secondary: "#CCCCCC".to_string(),
                success: "#00FF00".to_string(),
                warning: "#FFFF00".to_string(),
                error: "#FF0000".to_string(),
                info: "#00FFFF".to_string(),
                text: "#FFFFFF".to_string(),
                text_dim: "#AAAAAA".to_string(),
                background: "#000000".to_string(),
                border: "#666666".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#FFFFFF".to_string(),
                string: "#FFFFFF".to_string(),
                comment: "#AAAAAA".to_string(),
                function: "#FFFFFF".to_string(),
                variable: "#FFFFFF".to_string(),
                number: "#FFFFFF".to_string(),
                operator: "#FFFFFF".to_string(),
                type_name: "#FFFFFF".to_string(),
            },
            use_icons: false,
            box_drawing: BoxDrawingStyle::Minimal,
        };
        self.available_themes.insert("minimal".to_string(), minimal_theme);
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        if let Some(theme) = self.available_themes.get(theme_name) {
            self.current_theme = theme.clone();
            Ok(())
        } else {
            anyhow::bail!("Theme '{}' not found", theme_name)
        }
    }

    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// List available themes
    pub fn available_themes(&self) -> Vec<&str> {
        self.available_themes.keys().map(|k| k.as_str()).collect()
    }

    /// Get themed text for different message types
    pub fn format_role(&self, role_text: &str, role: crate::llm::common::model::role::Role) -> ColoredString {
        match role {
            crate::llm::common::model::role::Role::User => {
                if self.current_theme.use_icons {
                    format!("💬 {}", role_text).bright_blue().bold()
                } else {
                    role_text.bright_blue().bold()
                }
            }
            crate::llm::common::model::role::Role::Assistant => {
                if self.current_theme.use_icons {
                    format!("🤖 {}", role_text).bright_green().bold()
                } else {
                    role_text.bright_green().bold()
                }
            }
            crate::llm::common::model::role::Role::System => {
                if self.current_theme.use_icons {
                    format!("⚙️  {}", role_text).bright_yellow().bold()
                } else {
                    role_text.bright_yellow().bold()
                }
            }
        }
    }

    /// Format success message
    pub fn format_success(&self, text: &str) -> ColoredString {
        if self.current_theme.use_icons {
            format!("✅ {}", text).bright_green()
        } else {
            text.bright_green()
        }
    }

    /// Format warning message
    pub fn format_warning(&self, text: &str) -> ColoredString {
        if self.current_theme.use_icons {
            format!("⚠️  {}", text).bright_yellow()
        } else {
            text.bright_yellow()
        }
    }

    /// Format error message
    pub fn format_error(&self, text: &str) -> ColoredString {
        if self.current_theme.use_icons {
            format!("❌ {}", text).bright_red()
        } else {
            text.bright_red()
        }
    }

    /// Format info message
    pub fn format_info(&self, text: &str) -> ColoredString {
        if self.current_theme.use_icons {
            format!("💡 {}", text).bright_cyan()
        } else {
            text.bright_cyan()
        }
    }

    /// Format thinking indicator
    pub fn format_thinking(&self, text: &str) -> ColoredString {
        if self.current_theme.use_icons {
            format!("🤔 {}", text).bright_cyan()
        } else {
            text.bright_cyan()
        }
    }

    /// Get box drawing characters for current theme
    pub fn box_chars(&self) -> BoxChars {
        match self.current_theme.box_drawing {
            BoxDrawingStyle::Single => BoxChars::single(),
            BoxDrawingStyle::Double => BoxChars::double(),
            BoxDrawingStyle::Rounded => BoxChars::rounded(),
            BoxDrawingStyle::Thick => BoxChars::thick(),
            BoxDrawingStyle::Minimal => BoxChars::minimal(),
        }
    }

    /// Create a themed separator line
    pub fn separator(&self, width: usize) -> String {
        let chars = self.box_chars();
        chars.horizontal.to_string().repeat(width)
    }

    /// Create a themed box with title
    pub fn create_box(&self, title: &str, content: &str, width: Option<usize>) -> String {
        let chars = self.box_chars();
        let box_width = width.unwrap_or(60);
        let content_width = box_width.saturating_sub(4);

        let mut result = String::new();

        // Top border with title
        result.push(chars.top_left);
        if !title.is_empty() {
            let title_text = format!(" {} ", title);
            let remaining = box_width.saturating_sub(title_text.len() + 2);
            result.push(chars.horizontal);
            result.push_str(&title_text);
            result.push_str(&chars.horizontal.to_string().repeat(remaining));
        } else {
            result.push_str(&chars.horizontal.to_string().repeat(box_width.saturating_sub(2)));
        }
        result.push(chars.top_right);
        result.push('\n');

        // Content lines
        for line in content.lines() {
            result.push(chars.vertical);
            result.push(' ');
            
            if line.len() <= content_width {
                result.push_str(line);
                result.push_str(&" ".repeat(content_width.saturating_sub(line.len())));
            } else {
                result.push_str(&line[..content_width.saturating_sub(3)]);
                result.push_str("...");
            }
            
            result.push(' ');
            result.push(chars.vertical);
            result.push('\n');
        }

        // Bottom border
        result.push(chars.bottom_left);
        result.push_str(&chars.horizontal.to_string().repeat(box_width.saturating_sub(2)));
        result.push(chars.bottom_right);

        result
    }
}

/// Box drawing characters for different styles
#[derive(Debug, Clone)]
pub struct BoxChars {
    pub horizontal: char,
    pub vertical: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub cross: char,
    pub t_up: char,
    pub t_down: char,
    pub t_left: char,
    pub t_right: char,
}

impl BoxChars {
    pub fn single() -> Self {
        Self {
            horizontal: '─',
            vertical: '│',
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            cross: '┼',
            t_up: '┴',
            t_down: '┬',
            t_left: '┤',
            t_right: '├',
        }
    }

    pub fn double() -> Self {
        Self {
            horizontal: '═',
            vertical: '║',
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            cross: '╬',
            t_up: '╩',
            t_down: '╦',
            t_left: '╣',
            t_right: '╠',
        }
    }

    pub fn rounded() -> Self {
        Self {
            horizontal: '─',
            vertical: '│',
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            cross: '┼',
            t_up: '┴',
            t_down: '┬',
            t_left: '┤',
            t_right: '├',
        }
    }

    pub fn thick() -> Self {
        Self {
            horizontal: '━',
            vertical: '┃',
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            cross: '╋',
            t_up: '┻',
            t_down: '┳',
            t_left: '┫',
            t_right: '┣',
        }
    }

    pub fn minimal() -> Self {
        Self {
            horizontal: '-',
            vertical: '|',
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            cross: '+',
            t_up: '+',
            t_down: '+',
            t_left: '+',
            t_right: '+',
        }
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert!(manager.available_themes.len() > 0);
        assert!(manager.available_themes.contains_key("default"));
        assert!(manager.available_themes.contains_key("dark"));
        assert!(manager.available_themes.contains_key("light"));
    }

    #[test]
    fn test_theme_switching() {
        let mut manager = ThemeManager::new();
        
        assert_eq!(manager.current_theme.name, "Default");
        
        let result = manager.set_theme("dark");
        assert!(result.is_ok());
        assert_eq!(manager.current_theme.name, "Dark");
        
        let result = manager.set_theme("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_box_drawing() {
        let manager = ThemeManager::new();
        let box_content = manager.create_box("Test", "Hello\nWorld", Some(30));
        
        assert!(box_content.contains("Test"));
        assert!(box_content.contains("Hello"));
        assert!(box_content.contains("World"));
        assert!(box_content.contains('┌') || box_content.contains('╭'));
    }

    #[test]
    fn test_message_formatting() {
        let manager = ThemeManager::new();
        
        let user_msg = manager.format_role("User", crate::llm::common::model::role::Role::User);
        let assistant_msg = manager.format_role("Assistant", crate::llm::common::model::role::Role::Assistant);
        
        assert!(user_msg.to_string().contains("User"));
        assert!(assistant_msg.to_string().contains("Assistant"));
    }

    #[test]
    fn test_status_formatting() {
        let manager = ThemeManager::new();
        
        let success = manager.format_success("Success message");
        let warning = manager.format_warning("Warning message");
        let error = manager.format_error("Error message");
        let info = manager.format_info("Info message");
        
        assert!(success.to_string().contains("Success message"));
        assert!(warning.to_string().contains("Warning message"));
        assert!(error.to_string().contains("Error message"));
        assert!(info.to_string().contains("Info message"));
    }
}