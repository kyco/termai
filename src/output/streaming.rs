use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;
use colored::*;
use crossterm::{
    cursor::MoveToColumn,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

/// Configuration for streaming output behavior
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// Characters to display per batch
    pub chars_per_batch: usize,
    /// Delay between batches in milliseconds
    pub batch_delay_ms: u64,
    /// Show typing indicator while streaming
    pub show_typing_indicator: bool,
    /// Enable smooth scrolling effects
    pub enable_smooth_scrolling: bool,
    /// Maximum content length before enabling streaming
    pub min_content_length: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chars_per_batch: 3,
            batch_delay_ms: 15,
            show_typing_indicator: true,
            enable_smooth_scrolling: true,
            min_content_length: 50,
        }
    }
}

/// Manages streaming output with typing indicators and smooth rendering
pub struct StreamingRenderer {
    config: StreamingConfig,
    stdout: io::Stdout,
    is_cancelled: bool,
}

impl StreamingRenderer {
    pub fn new(config: StreamingConfig) -> Self {
        Self {
            config,
            stdout: io::stdout(),
            is_cancelled: false,
        }
    }

    /// Stream text output with typewriter effect
    pub async fn stream_text(&mut self, content: &str, role_prefix: Option<&str>) -> io::Result<()> {
        if content.len() < self.config.min_content_length {
            // For short content, just print immediately
            if let Some(prefix) = role_prefix {
                print!("{}", prefix);
            }
            println!("{}", content);
            return Ok(());
        }

        // Show typing indicator if enabled
        if self.config.show_typing_indicator {
            if let Some(prefix) = role_prefix {
                print!("{}", prefix);
            }
            self.show_typing_indicator().await?;
        }

        // Clear typing indicator and show role prefix
        if self.config.show_typing_indicator {
            self.clear_typing_indicator()?;
            if let Some(prefix) = role_prefix {
                print!("{}", prefix);
                self.stdout.flush()?;
            }
        } else if let Some(prefix) = role_prefix {
            print!("{}", prefix);
            self.stdout.flush()?;
        }

        // Stream the content
        let mut chars_written = 0;
        let content_chars: Vec<char> = content.chars().collect();
        
        while chars_written < content_chars.len() && !self.is_cancelled {
            let end_pos = std::cmp::min(
                chars_written + self.config.chars_per_batch,
                content_chars.len(),
            );
            
            let chunk: String = content_chars[chars_written..end_pos].iter().collect();
            print!("{}", chunk);
            self.stdout.flush()?;
            
            chars_written = end_pos;
            
            if chars_written < content_chars.len() {
                sleep(Duration::from_millis(self.config.batch_delay_ms)).await;
            }
        }

        if !content.ends_with('\n') {
            println!();
        }

        Ok(())
    }

    /// Stream code block with enhanced formatting
    pub async fn stream_code_block(
        &mut self,
        content: &str,
        language: Option<&str>,
        syntax_highlighter: Option<&dyn Fn(&str, Option<&str>) -> String>,
    ) -> io::Result<()> {
        // Code block header
        let header = if let Some(lang) = language {
            format!("┌─ {} ─{}", lang.bright_cyan().bold(), "─".repeat(40_usize.saturating_sub(lang.len() + 4)))
        } else {
            format!("┌─ Code ─{}", "─".repeat(37))
        };
        
        println!("{}", header.bright_black());

        // Apply syntax highlighting if available
        let highlighted_content = if let Some(highlighter) = syntax_highlighter {
            highlighter(content, language)
        } else {
            content.to_string()
        };

        // Stream the code content (no left border for clean copy-paste)
        let lines: Vec<&str> = highlighted_content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Stream each line character by character for effect
            if line.len() > 20 {
                let chars: Vec<char> = line.chars().collect();
                let mut char_pos = 0;
                
                while char_pos < chars.len() && !self.is_cancelled {
                    let end_pos = std::cmp::min(char_pos + self.config.chars_per_batch, chars.len());
                    let chunk: String = chars[char_pos..end_pos].iter().collect();
                    print!("{}", chunk);
                    self.stdout.flush()?;
                    char_pos = end_pos;
                    
                    if char_pos < chars.len() {
                        sleep(Duration::from_millis(self.config.batch_delay_ms / 2)).await;
                    }
                }
            } else {
                print!("{}", line);
            }
            
            println!();
            
            // Small delay between lines for readability
            if i < lines.len() - 1 {
                sleep(Duration::from_millis(self.config.batch_delay_ms)).await;
            }
        }

        println!("{}", format!("└{}", "─".repeat(50)).bright_black());
        
        Ok(())
    }

    /// Show typing indicator animation
    async fn show_typing_indicator(&mut self) -> io::Result<()> {
        let indicators = ["⌨️  AI is thinking...", "⌨️  AI is typing...", "⌨️  AI is responding..."];
        let dots = ["", ".", "..", "..."];
        
        // Show for about 1 second with animation
        for cycle in 0..8 {
            let indicator = indicators[cycle % indicators.len()];
            let dot_pattern = dots[cycle % dots.len()];
            
            print!("\r{}{}", indicator.bright_cyan(), dot_pattern);
            self.stdout.flush()?;
            sleep(Duration::from_millis(150)).await;
        }
        
        Ok(())
    }

    /// Clear the typing indicator line
    fn clear_typing_indicator(&mut self) -> io::Result<()> {
        self.stdout.execute(MoveToColumn(0))?;
        self.stdout.execute(Clear(ClearType::CurrentLine))?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Stream a markdown table with enhanced formatting
    pub async fn stream_table(&mut self, headers: &[String], rows: &[Vec<String>]) -> io::Result<()> {
        if headers.is_empty() || rows.is_empty() {
            return Ok(());
        }

        // Calculate column widths
        let mut col_widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Add some padding
        for width in &mut col_widths {
            *width += 2;
        }

        // Top border
        let top_border = format!(
            "┌{}┐",
            col_widths
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("┬")
        );
        println!("{}", top_border.bright_black());

        // Headers
        print!("│");
        for (i, header) in headers.iter().enumerate() {
            let width = col_widths.get(i).unwrap_or(&10);
            print!("{:^width$}", header.bright_blue().bold(), width = width);
            if i < headers.len() - 1 {
                print!("│");
            }
        }
        println!("│");

        // Header separator
        let header_sep = format!(
            "├{}┤",
            col_widths
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("┼")
        );
        println!("{}", header_sep.bright_black());

        // Data rows with streaming effect
        for (row_idx, row) in rows.iter().enumerate() {
            print!("│");
            for (i, cell) in row.iter().enumerate() {
                let width = col_widths.get(i).unwrap_or(&10);
                
                // Stream longer cells for effect
                if cell.len() > 15 {
                    let chars: Vec<char> = cell.chars().collect();
                    let mut char_pos = 0;
                    
                    while char_pos < chars.len() && !self.is_cancelled {
                        let end_pos = std::cmp::min(char_pos + 2, chars.len());
                        let chunk: String = chars[char_pos..end_pos].iter().collect();
                        print!("{}", chunk);
                        self.stdout.flush()?;
                        char_pos = end_pos;
                        
                        if char_pos < chars.len() {
                            sleep(Duration::from_millis(10)).await;
                        }
                    }
                    
                    // Pad to width
                    let padding = width.saturating_sub(cell.len());
                    print!("{}", " ".repeat(padding));
                } else {
                    print!("{:<width$}", cell, width = width);
                }
                
                if i < row.len() - 1 {
                    print!("│");
                }
            }
            println!("│");
            
            // Small delay between rows
            if row_idx < rows.len() - 1 {
                sleep(Duration::from_millis(50)).await;
            }
        }

        // Bottom border
        let bottom_border = format!(
            "└{}┘",
            col_widths
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("┴")
        );
        println!("{}", bottom_border.bright_black());

        Ok(())
    }

    /// Cancel ongoing streaming
    pub fn cancel(&mut self) {
        self.is_cancelled = true;
    }

    /// Check if streaming was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    /// Reset cancellation state
    pub fn reset(&mut self) {
        self.is_cancelled = false;
    }
}

/// Stream content with smart content detection
pub async fn stream_smart_content(
    renderer: &mut StreamingRenderer,
    content: &str,
    role_prefix: Option<&str>,
) -> io::Result<()> {
    // Detect content type and stream accordingly
    if content.contains("```") {
        stream_markdown_content(renderer, content, role_prefix).await
    } else if content.contains('|') && content.lines().any(|line| line.matches('|').count() > 1) {
        // Might be a table, try to parse it
        if let Some((headers, rows)) = parse_simple_table(content) {
            if let Some(prefix) = role_prefix {
                print!("{}", prefix);
            }
            renderer.stream_table(&headers, &rows).await
        } else {
            renderer.stream_text(content, role_prefix).await
        }
    } else {
        renderer.stream_text(content, role_prefix).await
    }
}

/// Stream markdown content with special handling for code blocks
async fn stream_markdown_content(
    renderer: &mut StreamingRenderer,
    content: &str,
    role_prefix: Option<&str>,
) -> io::Result<()> {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut first_content = true;

    while i < lines.len() {
        let line = lines[i];
        
        if line.trim_start().starts_with("```") {
            // Extract language if specified
            let language = if line.len() > 3 {
                let lang = line.trim_start().strip_prefix("```").unwrap_or("").trim();
                if lang.is_empty() { None } else { Some(lang) }
            } else {
                None
            };
            
            // Find the closing ```
            let mut code_content = Vec::new();
            i += 1;
            
            while i < lines.len() && !lines[i].trim_start().starts_with("```") {
                code_content.push(lines[i]);
                i += 1;
            }
            
            if first_content && role_prefix.is_some() {
                print!("{}", role_prefix.unwrap());
                first_content = false;
            }
            
            renderer.stream_code_block(&code_content.join("\n"), language, None).await?;
            
            if i < lines.len() {
                i += 1; // Skip closing ```
            }
        } else {
            // Regular text content
            if first_content {
                renderer.stream_text(line, role_prefix).await?;
                first_content = false;
            } else {
                renderer.stream_text(line, None).await?;
            }
            i += 1;
        }
    }

    Ok(())
}

/// Simple table parser for markdown-style tables
fn parse_simple_table(content: &str) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 {
        return None;
    }

    // Look for table header pattern
    for (idx, line) in lines.iter().enumerate() {
        if line.contains('|') && line.matches('|').count() > 1 {
            // This might be a header row
            let headers: Vec<String> = line
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            if headers.len() < 2 {
                continue;
            }

            // Look for separator row (optional)
            let start_row = if idx + 1 < lines.len() 
                && lines[idx + 1].contains("---") 
                && lines[idx + 1].contains('|') {
                idx + 2
            } else {
                idx + 1
            };

            // Parse data rows
            let mut rows = Vec::new();
            for line in lines.iter().skip(start_row) {
                if line.contains('|') && line.matches('|').count() > 1 {
                    let row: Vec<String> = line
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    
                    if row.len() >= headers.len() {
                        rows.push(row);
                    }
                }
            }

            if !rows.is_empty() {
                return Some((headers, rows));
            }
        }
    }

    None
}