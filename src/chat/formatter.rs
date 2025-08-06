use crate::llm::common::model::role::Role;
use chrono::{DateTime, Local};
use colored::*;

/// Enhanced formatter for chat mode with better UX
pub struct ChatFormatter {
    show_timestamps: bool,
    show_role_labels: bool,
}

impl ChatFormatter {
    pub fn new() -> Self {
        Self {
            show_timestamps: true,
            show_role_labels: true,
        }
    }

    /// Format a chat message with enhanced styling
    pub fn format_message(
        &self,
        role: &Role,
        content: &str,
        timestamp: Option<DateTime<Local>>,
    ) -> String {
        let mut output = String::new();

        // Add visual separator for AI responses
        if matches!(role, Role::Assistant) {
            output.push('\n');
        }

        // Add role label with color coding and timestamp
        if self.show_role_labels {
            let (role_label, role_icon) = match role {
                Role::User => ("You".bright_blue().bold(), "💬"),
                Role::Assistant => ("AI".bright_green().bold(), "🤖"),
                Role::System => ("System".bright_yellow().bold(), "⚙️"),
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

            output.push_str(&format!("{} {}{}: ", role_icon, role_label, timestamp_str));
        }

        // Format the content based on role
        let formatted_content = match role {
            Role::User => content.normal(),
            Role::Assistant => self.format_ai_response(content),
            Role::System => content.dimmed(),
        };

        output.push_str(&formatted_content.to_string());

        // Add spacing after AI responses
        if matches!(role, Role::Assistant) {
            output.push('\n');
        }

        output
    }

    /// Format AI response with syntax highlighting hints
    fn format_ai_response(&self, content: &str) -> ColoredString {
        // For now, just return normal formatting
        // TODO: Add code block detection and syntax highlighting
        content.normal()
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
            "/help - Show available slash commands",
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

    /// Format help text for slash commands
    pub fn format_help(&self, commands: &[(&str, &str)]) -> String {
        let mut help = String::new();
        help.push_str("┌────────────────────────────────────────────────┐\n");
        help.push_str("│              📚 Available Commands             │\n");
        help.push_str("├────────────────────────────────────────────────┤\n");

        for (command, description) in commands {
            let formatted_line = format!(" {} - {}", command, description);
            if formatted_line.len() <= 46 {
                help.push_str(&format!(
                    "│{}{}│\n",
                    formatted_line,
                    " ".repeat(48 - formatted_line.len())
                ));
            } else {
                // Truncate if too long
                let truncated = &formatted_line[..43];
                help.push_str(&format!("│{}... │\n", truncated));
            }
        }

        help.push_str("├────────────────────────────────────────────────┤\n");
        help.push_str("│ 💡 Tip: Commands can be abbreviated (/h, /s)  │\n");
        help.push_str("└────────────────────────────────────────────────┘");
        help
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
        assert!(welcome.contains("/help"));
        assert!(welcome.contains("┌")); // Check for proper box formatting
        assert!(welcome.contains("└"));
    }
}
