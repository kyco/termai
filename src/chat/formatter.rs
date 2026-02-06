use crate::llm::common::model::role::Role;
use crate::output::streaming::{StreamingRenderer, StreamingConfig};
use crate::output::syntax::SyntaxHighlighter;
use crate::output::themes::ThemeManager;
use chrono::{DateTime, Local};
use colored::*;

/// Enhanced formatter for chat mode with better UX
pub struct ChatFormatter {
    show_timestamps: bool,
    show_role_labels: bool,
    streaming_renderer: StreamingRenderer,
    syntax_highlighter: SyntaxHighlighter,
    theme_manager: ThemeManager,
    enable_streaming: bool,
    enable_markdown: bool,
}

impl ChatFormatter {
    pub fn new() -> Self {
        Self {
            show_timestamps: true,
            show_role_labels: true,
            streaming_renderer: StreamingRenderer::new(StreamingConfig {
                chars_per_batch: 2,
                batch_delay_ms: 12,
                show_typing_indicator: true,
                enable_smooth_scrolling: true,
                min_content_length: 20,
            }),
            syntax_highlighter: SyntaxHighlighter::new(),
            theme_manager: ThemeManager::new(),
            enable_streaming: true,
            enable_markdown: true,
        }
    }

    /// Format a chat message with enhanced styling
    pub async fn format_message_async(
        &mut self,
        role: &Role,
        content: &str,
        timestamp: Option<DateTime<Local>>,
    ) -> Result<(), std::io::Error> {
        // Add visual separator for AI responses
        if matches!(role, Role::Assistant) {
            println!();
        }

        // Create role prefix with theme
        let role_prefix = if self.show_role_labels {
            let role_text = match role {
                Role::User => "You",
                Role::Assistant => "AI", 
                Role::System => "System",
            };
            
            let formatted_role = self.theme_manager.format_role(role_text, role.clone());
            
            let timestamp_str = if self.show_timestamps {
                if let Some(ts) = timestamp {
                    format!(" {}", ts.format("%H:%M:%S").to_string().dimmed())
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            Some(format!("{}{}: ", formatted_role, timestamp_str))
        } else {
            None
        };

        // Handle different content types based on role
        match role {
            Role::Assistant => {
                if self.enable_markdown {
                    self.format_ai_response_async(content, role_prefix).await?;
                } else {
                    if let Some(prefix) = role_prefix {
                        print!("{}", prefix);
                    }
                    println!("{}", content);
                }
            }
            Role::User => {
                if let Some(prefix) = role_prefix {
                    print!("{}", prefix);
                }
                println!("{}", content.normal());
            }
            Role::System => {
                if let Some(prefix) = role_prefix {
                    print!("{}", prefix);
                }
                println!("{}", content.dimmed());
            }
        }

        // Add spacing after AI responses
        if matches!(role, Role::Assistant) {
            println!();
        }

        Ok(())
    }

    /// Legacy synchronous method for backward compatibility
    pub fn format_message(
        &self,
        role: &Role,
        content: &str,
        timestamp: Option<DateTime<Local>>,
    ) -> String {
        // For backward compatibility, return a simple formatted string
        let role_text = match role {
            Role::User => "💬 You",
            Role::Assistant => "🤖 AI", 
            Role::System => "⚙️ System",
        };
        
        let timestamp_str = if self.show_timestamps {
            if let Some(ts) = timestamp {
                format!(" {}", ts.format("%H:%M:%S").to_string().dimmed())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!("{}{}: {}", role_text, timestamp_str, content)
    }

    /// Format AI response with enhanced markdown and syntax highlighting
    async fn format_ai_response_async(
        &mut self,
        content: &str,
        role_prefix: Option<String>,
    ) -> Result<(), std::io::Error> {
        use crate::output::streaming::stream_smart_content;
        
        if self.enable_streaming {
            // Use our enhanced streaming system
            stream_smart_content(
                &mut self.streaming_renderer,
                content,
                role_prefix.as_deref(),
            ).await?;
        } else {
            // Use synchronous enhanced formatting
            self.format_content_synchronously(content, role_prefix.as_deref()).await?;
        }
        
        Ok(())
    }

    /// Format content synchronously with enhanced markdown support
    async fn format_content_synchronously(
        &self,
        content: &str,
        role_prefix: Option<&str>,
    ) -> Result<(), std::io::Error> {
        if let Some(prefix) = role_prefix {
            print!("{}", prefix);
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            
            // Handle code blocks
            if line.trim_start().starts_with("```") {
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
                self.print_code_block(&code, language)?;
                
                if i < lines.len() {
                    i += 1; // Skip closing ```
                }
            }
            // Handle tables
            else if self.is_table_line(line) {
                let mut table_lines = Vec::new();
                while i < lines.len() && (self.is_table_line(lines[i]) || lines[i].trim().is_empty()) {
                    if !lines[i].trim().is_empty() {
                        table_lines.push(lines[i]);
                    }
                    i += 1;
                }
                
                if !table_lines.is_empty() {
                    self.print_table(&table_lines)?;
                }
            }
            // Handle lists and other markdown
            else {
                let formatted_line = self.format_markdown_line(line);
                println!("{}", formatted_line);
                i += 1;
            }
        }

        Ok(())
    }

    /// Print a code block with syntax highlighting
    fn print_code_block(&self, code: &str, language: Option<&str>) -> Result<(), std::io::Error> {
        
        // Code block header with improved styling
        let header = if let Some(lang) = language {
            let lang_display = match lang.to_lowercase().as_str() {
                "rust" => "🦀 Rust",
                "python" => "🐍 Python", 
                "javascript" => "⚡ JavaScript",
                "typescript" => "📘 TypeScript",
                "java" => "☕ Java",
                "go" => "🐹 Go",
                "c" => "⚙️ C",
                "cpp" | "c++" => "⚙️ C++",
                "bash" | "sh" => "🐚 Shell",
                "json" => "📄 JSON",
                "yaml" => "📋 YAML",
                "html" => "🌐 HTML",
                "css" => "🎨 CSS",
                "sql" => "🗄️ SQL",
                "zig" => "⚡ Zig",
                _ => lang,
            };
            
            let padding_length = 50_usize.saturating_sub(lang_display.len() + 4);
            format!("┌─ {} {}\n", 
                lang_display.bright_cyan().bold(),
                "─".repeat(padding_length).bright_black()
            )
        } else {
            "┌─ Code ─────────────────────────────────────────────\n".bright_black().to_string()
        };
        
        print!("{}", header);

        // Highlight and print code (no left border for clean copy-paste)
        match self.syntax_highlighter.highlight(code, language) {
            Ok(highlighted) => {
                for line in highlighted.lines() {
                    println!("{}", line);
                }
            }
            Err(_) => {
                // Fallback to unhighlighted code
                for line in code.lines() {
                    println!("{}", line);
                }
            }
        }

        // Bottom border
        println!("{}", 
            "└───────────────────────────────────────────────────".bright_black()
        );

        Ok(())
    }

    /// Print a table with enhanced formatting
    fn print_table(&self, table_lines: &[&str]) -> Result<(), std::io::Error> {
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

        self.print_simple_table(&headers, &rows)?;
        Ok(())
    }

    /// Simple table printing with box drawing
    fn print_simple_table(&self, headers: &[String], rows: &[Vec<String>]) -> Result<(), std::io::Error> {
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

    /// Format individual markdown lines
    fn format_markdown_line(&self, line: &str) -> ColoredString {
        let trimmed = line.trim();
        
        // Handle headers - improved styling with better visual separation
        if trimmed.starts_with("### ") {
            let title = &trimmed[4..]; // Remove "### "
            return format!("🔷 {}", title).bright_cyan().bold().to_string().into();
        } else if trimmed.starts_with("##") {
            let title = if trimmed.starts_with("## ") {
                &trimmed[3..] // Remove "## "
            } else {
                &trimmed[2..] // Remove "##"
            };
            return format!("🔵 {}", title.trim()).bright_blue().bold().to_string().into();
        } else if trimmed.starts_with("# ") {
            let title = &trimmed[2..]; // Remove "# "
            return format!("🟢 {}", title).bright_green().bold().to_string().into();
        }
        
        // Handle lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            return format!("  • {}", &trimmed[2..]).bright_yellow().to_string().into();
        }
        
        // Handle numbered lists
        if let Some(captures) = regex::Regex::new(r"^(\d+)\. (.*)").ok().and_then(|r| r.captures(trimmed)) {
            if let (Some(num), Some(content)) = (captures.get(1), captures.get(2)) {
                return format!("  {}. {}", num.as_str().bright_yellow().bold(), content.as_str()).into();
            }
        }
        
        // Handle blockquotes
        if trimmed.starts_with("> ") {
            return format!("│ {}", &trimmed[2..]).bright_magenta().italic().to_string().into();
        }
        
        // Handle bold and italic
        let mut formatted = line.to_string();
        
        // Handle inline code first
        if formatted.contains("`") {
            let re = regex::Regex::new(r"`([^`]+)`").unwrap();
            formatted = re.replace_all(&formatted, |caps: &regex::Captures| {
                format!("{}", caps[1].on_black().bright_white().bold())
            }).to_string();
        }
        
        // Handle bold text **text**
        if formatted.contains("**") {
            let re = regex::Regex::new(r"\*\*([^*]+)\*\*").unwrap();
            formatted = re.replace_all(&formatted, |caps: &regex::Captures| {
                format!("{}", caps[1].bold())
            }).to_string();
        }
        
        // Handle italic text *text*
        if formatted.contains("*") && !formatted.contains("**") {
            let re = regex::Regex::new(r"\*([^*]+)\*").unwrap();
            formatted = re.replace_all(&formatted, |caps: &regex::Captures| {
                format!("{}", caps[1].italic())
            }).to_string();
        }
        
        formatted.normal()
    }

    /// Enable or disable streaming
    #[allow(dead_code)]
    pub fn set_streaming(&mut self, enabled: bool) {
        self.enable_streaming = enabled;
    }

    /// Enable or disable markdown formatting
    #[allow(dead_code)]
    pub fn set_markdown(&mut self, enabled: bool) {
        self.enable_markdown = enabled;
    }

    /// Set theme
    #[allow(dead_code)]
    pub fn set_theme(&mut self, theme_name: &str) -> Result<(), String> {
        self.theme_manager.set_theme(theme_name).map_err(|e| e.to_string())
    }

    /// Get available themes
    #[allow(dead_code)]
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_manager.available_themes()
    }

    /// Format a system message (commands, status updates, etc.)
    #[allow(dead_code)]
    pub fn format_system_message(&self, message: &str) -> String {
        format!("💡 {}", message.bright_cyan())
    }

    /// Format an error message
    pub fn format_error(&self, error: &str) -> String {
        format!("❌ {}", error.bright_red())
    }

    /// Format a success message
    pub fn format_success(&self, message: &str) -> String {
        format!("✅ {}", message.bright_green())
    }

    /// Format a warning message
    pub fn format_warning(&self, message: &str) -> String {
        format!("⚠️  {}", message.bright_yellow())
    }

    /// Format the welcome message for chat mode
    pub fn format_welcome(&self) -> String {
        let width = 46; // Internal width
        let mut lines = Vec::new();

        // Top border
        lines.push(format!("┌{}┐", "─".repeat(width)));

        // Title
        let title = "TermAI Interactive Chat Mode";
        let padding = (width - title.len()) / 2;
        lines.push(format!(
            "│{}{}{}│",
            " ".repeat(padding),
            title,
            " ".repeat(width - title.len() - padding)
        ));

        // Separator
        lines.push(format!("├{}┤", "─".repeat(width)));

        // Content lines
        let content = vec![
            "Type your message and press Enter to chat",
            "Type ? or /commands to open command palette",
            "Ctrl+C twice to exit gracefully",
        ];

        for line in content {
            lines.push(format!(
                "│ {}{} │",
                line,
                " ".repeat(width - line.len() - 2)
            ));
        }

        // Bottom border
        lines.push(format!("└{}┘", "─".repeat(width)));

        lines.join("\n")
    }

    /// Format help text for slash commands (compact view from /help)
    pub fn format_help(&self, commands: &[(&str, &str)]) -> String {
        let mut help = String::new();
        help.push_str("┌────────────────────────────────────────────────────────────┐\n");
        help.push_str("│                    Available Commands                      │\n");
        help.push_str("├────────────────────────────────────────────────────────────┤\n");

        let inner_width = 58;
        for (command, description) in commands {
            let formatted_line = format!(" {} - {}", command, description);
            let display_len = formatted_line.len();
            if display_len <= inner_width {
                help.push_str(&format!(
                    "│{}{}│\n",
                    formatted_line,
                    " ".repeat(inner_width - display_len)
                ));
            } else {
                let truncated = &formatted_line[..inner_width - 3];
                help.push_str(&format!("│{}...│\n", truncated));
            }
        }

        help.push_str("├────────────────────────────────────────────────────────────┤\n");
        help.push_str("│ Tip: Type ? or /commands for the full command palette      │\n");
        help.push_str("└────────────────────────────────────────────────────────────┘");
        help
    }

    /// Format the rich command palette (from /commands or ?)
    /// Groups commands by category with a box-drawing UI
    pub fn format_command_palette(
        &self,
        palette: &[crate::chat::commands::CommandEntry],
    ) -> String {
        use crate::chat::commands::CommandCategory;

        let outer_width = 62;
        let inner_width = outer_width - 2; // inside the box borders

        let mut out = String::new();

        // Top border and title
        out.push_str(&format!("┌{}┐\n", "─".repeat(inner_width)));
        let title = "Command Palette";
        let pad_l = (inner_width - title.len()) / 2;
        let pad_r = inner_width - title.len() - pad_l;
        out.push_str(&format!(
            "│{}{}{}│\n",
            " ".repeat(pad_l),
            title,
            " ".repeat(pad_r)
        ));
        out.push_str(&format!("╞{}╡\n", "═".repeat(inner_width)));

        for cat in CommandCategory::all() {
            // Category header
            let cat_header = format!(" {} {}", cat.icon(), cat.label());
            let cat_pad = inner_width - cat_header.len();
            out.push_str(&format!("│{}{}│\n", cat_header, " ".repeat(cat_pad)));
            out.push_str(&format!("├{}┤\n", "┄".repeat(inner_width)));

            // Commands in this category
            let entries: Vec<&crate::chat::commands::CommandEntry> =
                palette.iter().filter(|e| e.category == *cat).collect();

            for entry in &entries {
                // Command column (left-aligned, fixed width)
                let cmd_col_width = 24;
                let cmd_text = if entry.aliases.is_empty() {
                    entry.command.to_string()
                } else {
                    format!("{}, {}", entry.command, entry.aliases)
                };

                let cmd_display = if cmd_text.len() <= cmd_col_width {
                    format!("{}{}", cmd_text, " ".repeat(cmd_col_width - cmd_text.len()))
                } else {
                    format!("{}...", &cmd_text[..cmd_col_width - 3])
                };

                // Description column (fills remaining width)
                let desc_width = inner_width - cmd_col_width - 4; // 4 = "│ " + " │"
                let desc_display = if entry.description.len() <= desc_width {
                    format!(
                        "{}{}",
                        entry.description,
                        " ".repeat(desc_width - entry.description.len())
                    )
                } else {
                    format!("{}...", &entry.description[..desc_width - 3])
                };

                out.push_str(&format!("│ {} {} │\n", cmd_display, desc_display));
            }

            // Separator between categories (skip after last)
            if *cat != *CommandCategory::all().last().unwrap() {
                out.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));
            }
        }

        // Footer
        out.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));
        let tip = " Tip: Commands accept abbreviated aliases (/h, /s, /m)";
        let tip_pad = inner_width - tip.len();
        if tip_pad > 0 {
            out.push_str(&format!("│{}{}│\n", tip, " ".repeat(tip_pad)));
        } else {
            out.push_str(&format!("│{}│\n", &tip[..inner_width]));
        }
        let tip2 = " Type ? anytime to reopen this palette";
        let tip2_pad = inner_width - tip2.len();
        if tip2_pad > 0 {
            out.push_str(&format!("│{}{}│\n", tip2, " ".repeat(tip2_pad)));
        }
        out.push_str(&format!("└{}┘", "─".repeat(inner_width)));

        out
    }

    /// Format a settings overview showing all current chat settings
    pub fn format_settings_overview(
        &self,
        provider: &str,
        model: &str,
        tools_enabled: bool,
        streaming_enabled: bool,
        context_file_count: usize,
        session_name: &str,
    ) -> String {
        let inner_width = 58;
        let mut out = String::new();

        out.push_str(&format!("┌{}┐\n", "─".repeat(inner_width)));
        let title = "Current Settings";
        let pad_l = (inner_width - title.len()) / 2;
        let pad_r = inner_width - title.len() - pad_l;
        out.push_str(&format!(
            "│{}{}{}│\n",
            " ".repeat(pad_l),
            title,
            " ".repeat(pad_r)
        ));
        out.push_str(&format!("├{}┤\n", "─".repeat(inner_width)));

        let rows = [
            ("Provider", provider),
            ("Model", model),
            (
                "Tools",
                if tools_enabled { "enabled" } else { "disabled" },
            ),
            (
                "Streaming",
                if streaming_enabled {
                    "enabled"
                } else {
                    "disabled"
                },
            ),
            ("Session", session_name),
        ];

        for (label, value) in &rows {
            let row = format!(" {:<16} {}", label, value);
            let pad = inner_width - row.len();
            out.push_str(&format!("│{}{}│\n", row, " ".repeat(pad)));
        }

        // Context count
        let ctx_row = format!(" {:<16} {} file(s)", "Context", context_file_count);
        let ctx_pad = inner_width - ctx_row.len();
        out.push_str(&format!("│{}{}│\n", ctx_row, " ".repeat(ctx_pad)));

        out.push_str(&format!("└{}┘", "─".repeat(inner_width)));
        out
    }

    /// Format context information
    pub fn format_context_info(&self, context_size: usize, files: &[String]) -> String {
        let mut info = String::new();
        info.push_str(
            &format!("📁 Context Information:\n")
                .bright_blue()
                .bold()
                .to_string(),
        );
        info.push_str(&format!(
            "   Total files: {}\n",
            context_size.to_string().bright_cyan()
        ));

        if !files.is_empty() {
            info.push_str("   Files:\n");
            for file in files.iter().take(10) {
                // Show max 10 files
                info.push_str(&format!("     • {}\n", file.normal()));
            }
            if files.len() > 10 {
                info.push_str(&format!(
                    "     ... and {} more\n",
                    (files.len() - 10).to_string().dimmed()
                ));
            }
        }

        info
    }

    /// Format a progress indicator
    #[allow(dead_code)]
    pub fn format_thinking(&self) -> String {
        "🤔 AI is thinking...".bright_cyan().to_string()
    }

    /// Format session save confirmation
    pub fn format_session_saved(&self, session_name: &str) -> String {
        format!("💾 Session saved as '{}'", session_name)
            .bright_green()
            .to_string()
    }

    /// Format conversation cleared message
    pub fn format_conversation_cleared(&self) -> String {
        "🗑️  Conversation history cleared"
            .bright_yellow()
            .to_string()
    }
}

impl Default for ChatFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_message_formatting() {
        let formatter = ChatFormatter::new();
        let now = Local::now();

        let user_msg = formatter.format_message(&Role::User, "Hello world", Some(now));
        assert!(user_msg.contains("💬 You"));
        assert!(user_msg.contains("Hello world"));

        let ai_msg = formatter.format_message(&Role::Assistant, "Hi there!", Some(now));
        assert!(ai_msg.contains("🤖 AI"));
        assert!(ai_msg.contains("Hi there!"));
    }

    #[test]
    fn test_system_messages() {
        let formatter = ChatFormatter::new();

        let success = formatter.format_success("Operation completed");
        assert!(success.contains("✅"));

        let error = formatter.format_error("Something went wrong");
        assert!(error.contains("❌"));

        let warning = formatter.format_warning("Be careful");
        assert!(warning.contains("⚠️"));
    }

    #[test]
    fn test_welcome_message() {
        let formatter = ChatFormatter::new();
        let welcome = formatter.format_welcome();
        assert!(welcome.contains("TermAI Interactive Chat Mode"));
        assert!(welcome.contains("/commands"));
        assert!(welcome.contains("┌")); // Check for proper box formatting
        assert!(welcome.contains("└"));
    }
}
