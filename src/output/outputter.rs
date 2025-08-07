use crate::output::message::Message;
use crate::output::streaming::{StreamingRenderer, StreamingConfig, stream_smart_content};
use crate::output::syntax::SyntaxHighlighter;
use crate::output::themes::ThemeManager;
use crate::llm::common::model::role::Role;
use colored::*;
use anyhow::Result;

/// Enhanced outputter with streaming, themes, and better formatting
pub struct EnhancedOutputter {
    streaming_renderer: StreamingRenderer,
    syntax_highlighter: SyntaxHighlighter,
    theme_manager: ThemeManager,
    enable_streaming: bool,
}

impl EnhancedOutputter {
    pub fn new() -> Self {
        Self {
            streaming_renderer: StreamingRenderer::new(StreamingConfig::default()),
            syntax_highlighter: SyntaxHighlighter::new(),
            theme_manager: ThemeManager::new(),
            enable_streaming: true,
        }
    }

    /// Print messages with enhanced formatting
    pub async fn print_messages(&mut self, messages: Vec<Message>) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        println!();

        for (i, message) in messages.iter().enumerate() {
            // Create role prefix with theme
            let role_text = message.role.to_string().to_uppercase();
            let role_prefix = self.theme_manager.format_role(&role_text, message.role.clone()).to_string();
            
            if self.enable_streaming {
                // Use streaming renderer for better UX
                stream_smart_content(
                    &mut self.streaming_renderer,
                    &message.message,
                    Some(&format!("{}: ", role_prefix))
                ).await?;
            } else {
                // Fallback to instant display
                print!("{}: ", role_prefix);
                self.print_formatted_content(&message.message, &message.role).await?;
            }

            // Add separator between messages (except for last)
            if i < messages.len() - 1 {
                println!("{}", self.theme_manager.separator(50).dimmed());
                println!();
            }
        }

        println!();
        Ok(())
    }

    /// Print a single message with enhanced formatting
    pub async fn print_message(&mut self, message: &Message) -> Result<()> {
        self.print_messages(vec![message.clone()]).await
    }

    /// Print formatted content with smart syntax detection
    async fn print_formatted_content(&mut self, content: &str, role: &Role) -> Result<()> {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            
            if line.trim_start().starts_with("```") {
                // Handle code blocks
                let language = if line.len() > 3 {
                    let lang = line.trim_start().strip_prefix("```").unwrap_or("").trim();
                    if lang.is_empty() { None } else { Some(lang) }
                } else {
                    None
                };
                
                let mut code_content = Vec::new();
                i += 1;
                
                while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                    code_content.push(lines[i]);
                    i += 1;
                }
                
                let code = code_content.join("\n");
                self.print_code_block(&code, language).await?;
                
                if i < lines.len() {
                    i += 1; // Skip closing ```
                }
            } else if self.is_table_line(line) {
                // Handle tables
                let mut table_lines = Vec::new();
                while i < lines.len() && (self.is_table_line(lines[i]) || lines[i].trim().is_empty()) {
                    if !lines[i].trim().is_empty() {
                        table_lines.push(lines[i]);
                    }
                    i += 1;
                }
                
                if !table_lines.is_empty() {
                    self.print_table(&table_lines).await?;
                }
            } else {
                // Regular text
                println!("{}", self.format_text_line(line, role));
                i += 1;
            }
        }

        Ok(())
    }

    /// Print a code block with syntax highlighting
    async fn print_code_block(&mut self, code: &str, language: Option<&str>) -> Result<()> {
        let box_chars = self.theme_manager.box_chars();
        
        // Code block header
        let header = if let Some(lang) = language {
            format!("{} {} {}", 
                box_chars.top_left, 
                lang.bright_cyan().bold(),
                box_chars.horizontal.to_string().repeat(40_usize.saturating_sub(lang.len() + 2))
            )
        } else {
            format!("{} Code {}", 
                box_chars.top_left,
                box_chars.horizontal.to_string().repeat(43)
            )
        };
        
        println!("{}", header.bright_black());

        // Highlight and print code
        match self.syntax_highlighter.highlight(code, language) {
            Ok(highlighted) => {
                for line in highlighted.lines() {
                    println!("{} {}", box_chars.vertical.to_string().bright_black(), line);
                }
            }
            Err(_) => {
                // Fallback to unhighlighted code
                for line in code.lines() {
                    println!("{} {}", box_chars.vertical.to_string().bright_black(), line);
                }
            }
        }

        // Bottom border
        println!("{}{}", 
            box_chars.bottom_left.to_string().bright_black(),
            box_chars.horizontal.to_string().repeat(50).bright_black()
        );

        Ok(())
    }

    /// Print a table
    async fn print_table(&mut self, table_lines: &[&str]) -> Result<()> {
        // Simple table parsing
        if table_lines.len() < 2 {
            return Ok(());
        }

        // Parse header
        let headers: Vec<String> = table_lines[0]
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if headers.is_empty() {
            return Ok(());
        }

        // Parse data rows (skip separator row if present)
        let start_row = if table_lines.len() > 1 && table_lines[1].contains("---") {
            2
        } else {
            1
        };

        let mut rows = Vec::new();
        for line in table_lines.iter().skip(start_row) {
            let row: Vec<String> = line
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            if !row.is_empty() && row.len() >= headers.len() {
                rows.push(row);
            }
        }

        // Use streaming renderer for table
        if self.enable_streaming {
            self.streaming_renderer.stream_table(&headers, &rows).await?;
        } else {
            // Fallback table printing
            self.print_simple_table(&headers, &rows).await?;
        }

        Ok(())
    }

    /// Simple table printing fallback
    async fn print_simple_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<()> {
        // Calculate column widths
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Add padding
        for width in &mut col_widths {
            *width += 2;
        }

        let box_chars = self.theme_manager.box_chars();

        // Top border
        print!("{}", box_chars.top_left);
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", box_chars.horizontal.to_string().repeat(*width));
            if i < col_widths.len() - 1 {
                print!("{}", box_chars.t_down);
            }
        }
        println!("{}", box_chars.top_right);

        // Headers
        print!("{}", box_chars.vertical);
        for (i, header) in headers.iter().enumerate() {
            let width = col_widths[i];
            print!(" {:^width$} ", header.bright_blue().bold(), width = width - 2);
            if i < headers.len() - 1 {
                print!("{}", box_chars.vertical);
            }
        }
        println!("{}", box_chars.vertical);

        // Header separator
        print!("{}", box_chars.t_right);
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", box_chars.horizontal.to_string().repeat(*width));
            if i < col_widths.len() - 1 {
                print!("{}", box_chars.cross);
            }
        }
        println!("{}", box_chars.t_left);

        // Data rows
        for row in rows {
            print!("{}", box_chars.vertical);
            for (i, cell) in row.iter().enumerate() {
                let width = col_widths.get(i).unwrap_or(&10);
                print!(" {:<width$} ", cell, width = width - 2);
                if i < row.len() - 1 {
                    print!("{}", box_chars.vertical);
                }
            }
            println!("{}", box_chars.vertical);
        }

        // Bottom border
        print!("{}", box_chars.bottom_left);
        for (i, width) in col_widths.iter().enumerate() {
            print!("{}", box_chars.horizontal.to_string().repeat(*width));
            if i < col_widths.len() - 1 {
                print!("{}", box_chars.t_up);
            }
        }
        println!("{}", box_chars.bottom_right);

        Ok(())
    }

    /// Check if a line is part of a table
    fn is_table_line(&self, line: &str) -> bool {
        line.contains('|') && line.matches('|').count() > 1
    }

    /// Format a regular text line
    fn format_text_line(&self, line: &str, role: &Role) -> ColoredString {
        match role {
            Role::User => line.normal(),
            Role::Assistant => line.white(),
            Role::System => line.dimmed(),
        }
    }

    /// Enable or disable streaming
    pub fn set_streaming(&mut self, enabled: bool) {
        self.enable_streaming = enabled;
    }

    /// Set theme
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        self.theme_manager.set_theme(theme_name)?;
        // Also update syntax highlighter theme if possible
        let _ = self.syntax_highlighter.set_theme(theme_name);
        Ok(())
    }

    /// Get available themes
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_manager.available_themes()
    }

    /// Print success message
    pub fn print_success(&self, message: &str) {
        println!("{}", self.theme_manager.format_success(message));
    }

    /// Print warning message
    pub fn print_warning(&self, message: &str) {
        println!("{}", self.theme_manager.format_warning(message));
    }

    /// Print error message
    pub fn print_error(&self, message: &str) {
        println!("{}", self.theme_manager.format_error(message));
    }

    /// Print info message
    pub fn print_info(&self, message: &str) {
        println!("{}", self.theme_manager.format_info(message));
    }
}

impl Default for EnhancedOutputter {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy function for backward compatibility
#[allow(dead_code)]
pub async fn print(messages: Vec<Message>) -> Result<()> {
    let mut outputter = EnhancedOutputter::new();
    outputter.print_messages(messages).await
}

// Convenience functions
pub async fn print_success(message: &str) {
    let outputter = EnhancedOutputter::new();
    outputter.print_success(message);
}

pub async fn print_warning(message: &str) {
    let outputter = EnhancedOutputter::new();
    outputter.print_warning(message);
}

pub async fn print_error(message: &str) {
    let outputter = EnhancedOutputter::new();
    outputter.print_error(message);
}

pub async fn print_info(message: &str) {
    let outputter = EnhancedOutputter::new();
    outputter.print_info(message);
}
