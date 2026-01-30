use anyhow::{Result, anyhow};
use serde_json::Value;
use std::fs;
use std::path::Path;
use comrak::{markdown_to_html, ComrakOptions};
use chrono;

use crate::llm::common::model::role::Role;
use crate::session::model::session::Session;
use crate::session::model::message::Message;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Markdown,
    Html,
    Json,
    Yaml,
    PlainText,
}

impl std::str::FromStr for ExportFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ExportFormat::Markdown),
            "html" | "htm" => Ok(ExportFormat::Html),
            "json" => Ok(ExportFormat::Json),
            "yaml" | "yml" => Ok(ExportFormat::Yaml),
            "text" | "txt" | "plain" => Ok(ExportFormat::PlainText),
            _ => Err(anyhow!("Unsupported export format: {}", s)),
        }
    }
}

/// Configuration for export operations
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Include timestamps in exports
    pub include_timestamps: bool,
    /// Include metadata headers
    pub include_metadata: bool,
    /// Custom CSS for HTML exports
    pub custom_css: Option<String>,
    /// Template for exports
    pub template: Option<String>,
    /// Theme for syntax highlighting
    pub syntax_theme: String,
    /// Whether to include thinking sections
    pub include_thinking: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_timestamps: true,
            include_metadata: true,
            custom_css: None,
            template: None,
            syntax_theme: "base16-ocean.dark".to_string(),
            include_thinking: false,
        }
    }
}

/// Exports conversations and messages to various formats
pub struct ConversationExporter {
    config: ExportConfig,
}

impl ConversationExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self { config }
    }

    /// Export a session to the specified format
    pub fn export_session(
        &self,
        session: &Session,
        format: ExportFormat,
        output_path: &Path,
    ) -> Result<()> {
        let content = match format {
            ExportFormat::Markdown => self.session_to_markdown(session)?,
            ExportFormat::Html => self.session_to_html(session)?,
            ExportFormat::Json => self.session_to_json(session)?,
            ExportFormat::Yaml => self.session_to_yaml(session)?,
            ExportFormat::PlainText => self.session_to_plain_text(session)?,
        };

        fs::write(output_path, content)?;
        Ok(())
    }

    /// Export a single conversation to the specified format
    pub fn export_messages(
        &self,
        messages: &[Message],
        format: ExportFormat,
        output_path: &Path,
        title: Option<&str>,
    ) -> Result<()> {
        let content = match format {
            ExportFormat::Markdown => self.messages_to_markdown(messages, title)?,
            ExportFormat::Html => self.messages_to_html(messages, title)?,
            ExportFormat::Json => self.messages_to_json(messages, title)?,
            ExportFormat::Yaml => self.messages_to_yaml(messages, title)?,
            ExportFormat::PlainText => self.messages_to_plain_text(messages, title)?,
        };

        fs::write(output_path, content)?;
        Ok(())
    }

    /// Convert session to Markdown format
    fn session_to_markdown(&self, session: &Session) -> Result<String> {
        let mut content = String::new();

        // Header with metadata
        if self.config.include_metadata {
            content.push_str("# TermAI Conversation Export\n\n");
            content.push_str(&format!("**Session ID:** {}\n", session.id));
            content.push_str(&format!("**Session Name:** {}\n", session.name));
            content.push_str(&format!("**Expires At:** {}\n", session.expires_at.format("%Y-%m-%d %H:%M:%S")));
            content.push_str(&format!("**Message Count:** {}\n\n", session.messages.len()));
            content.push_str("---\n\n");
        }

        // Messages
        content.push_str(&self.messages_to_markdown(&session.messages, None)?);

        Ok(content)
    }

    /// Convert messages to Markdown format
    fn messages_to_markdown(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let mut content = String::new();

        if let Some(title) = title {
            content.push_str(&format!("# {}\n\n", title));
        }

        for (i, message) in messages.iter().enumerate() {
            // Role header
            let role_emoji = match message.role {
                Role::User => "💬",
                Role::Assistant => "🤖",
                Role::System => "⚙️",
            };

            content.push_str(&format!("## {} {}\n\n", role_emoji, message.role.to_string().to_uppercase()));

            if self.config.include_timestamps {
                content.push_str(&format!("*Timestamp: {}*\n\n", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
            }

            // Clean and format message content
            let cleaned_content = self.clean_message_content(&message.content);
            content.push_str(&cleaned_content);
            content.push_str("\n\n");

            // Add separator between messages (except for last)
            if i < messages.len() - 1 {
                content.push_str("---\n\n");
            }
        }

        Ok(content)
    }

    /// Convert session to HTML format
    fn session_to_html(&self, session: &Session) -> Result<String> {
        let markdown_content = self.session_to_markdown(session)?;
        self.markdown_to_html(&markdown_content, Some(&session.name))
    }

    /// Convert messages to HTML format
    pub fn messages_to_html(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let markdown_content = self.messages_to_markdown(messages, title)?;
        self.markdown_to_html(&markdown_content, title)
    }

    /// Convert Markdown to HTML with custom styling
    fn markdown_to_html(&self, markdown: &str, title: Option<&str>) -> Result<String> {
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.tagfilter = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.header_ids = Some("".to_string());
        options.extension.footnotes = true;
        options.extension.description_lists = true;
        options.render.github_pre_lang = true;
        options.render.unsafe_ = false; // Security: disable raw HTML

        let html_body = markdown_to_html(markdown, &options);

        let css = self.get_html_css();
        let page_title = title.unwrap_or("TermAI Export");

        let full_html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <meta name="generator" content="TermAI">
    <style>
{}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>TermAI Conversation Export</h1>
            <p class="export-info">Exported on {}</p>
        </header>
        <main>
{}
        </main>
        <footer>
            <p>Generated by TermAI</p>
        </footer>
    </div>
</body>
</html>"#,
            page_title,
            css,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            html_body
        );

        Ok(full_html)
    }

    /// Convert session to JSON format
    fn session_to_json(&self, session: &Session) -> Result<String> {
        let mut json_data = serde_json::Map::new();
        
        // Metadata
        json_data.insert("export_info".to_string(), serde_json::json!({
            "exported_at": chrono::Local::now().to_rfc3339(),
            "exported_by": "TermAI",
            "version": "1.0"
        }));

        // Session data
        json_data.insert("session".to_string(), serde_json::json!({
            "id": session.id,
            "name": session.name,
            "expires_at": session.expires_at.and_utc().to_rfc3339(),
            "current": session.current,
            "message_count": session.messages.len()
        }));

        // Messages
        let messages_json: Vec<Value> = session.messages
            .iter()
            .map(|msg| serde_json::json!({
                "id": msg.id,
                "role": msg.role.to_string(),
                "content": msg.content,
                "timestamp": chrono::Local::now().to_rfc3339()
            }))
            .collect();

        json_data.insert("messages".to_string(), Value::Array(messages_json));

        Ok(serde_json::to_string_pretty(&json_data)?)
    }

    /// Convert messages to JSON format
    fn messages_to_json(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let mut json_data = serde_json::Map::new();
        
        json_data.insert("export_info".to_string(), serde_json::json!({
            "title": title.unwrap_or("TermAI Messages Export"),
            "exported_at": chrono::Local::now().to_rfc3339(),
            "exported_by": "TermAI",
            "version": "1.0"
        }));

        let messages_json: Vec<Value> = messages
            .iter()
            .map(|msg| serde_json::json!({
                "id": msg.id,
                "role": msg.role.to_string(),
                "content": msg.content
            }))
            .collect();

        json_data.insert("messages".to_string(), Value::Array(messages_json));

        Ok(serde_json::to_string_pretty(&json_data)?)
    }

    /// Convert session to YAML format
    fn session_to_yaml(&self, session: &Session) -> Result<String> {
        let json_content = self.session_to_json(session)?;
        let json_value: Value = serde_json::from_str(&json_content)?;
        Ok(serde_yaml::to_string(&json_value)?)
    }

    /// Convert messages to YAML format
    fn messages_to_yaml(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let json_content = self.messages_to_json(messages, title)?;
        let json_value: Value = serde_json::from_str(&json_content)?;
        Ok(serde_yaml::to_string(&json_value)?)
    }

    /// Convert session to plain text format
    fn session_to_plain_text(&self, session: &Session) -> Result<String> {
        let mut content = String::new();

        if self.config.include_metadata {
            content.push_str("TERMAI CONVERSATION EXPORT\n");
            content.push_str(&"=".repeat(50));
            content.push_str("\n\n");
            content.push_str(&format!("Session ID: {}\n", session.id));
            content.push_str(&format!("Session Name: {}\n", session.name));
            content.push_str(&format!("Expires At: {}\n", session.expires_at.format("%Y-%m-%d %H:%M:%S")));
            content.push_str(&format!("Messages: {}\n\n", session.messages.len()));
            content.push_str(&"-".repeat(50));
            content.push_str("\n\n");
        }

        content.push_str(&self.messages_to_plain_text(&session.messages, None)?);
        Ok(content)
    }

    /// Convert messages to plain text format
    fn messages_to_plain_text(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let mut content = String::new();

        if let Some(title) = title {
            content.push_str(&title.to_uppercase());
            content.push('\n');
            content.push_str(&"=".repeat(title.len()));
            content.push_str("\n\n");
        }

        for (i, message) in messages.iter().enumerate() {
            // Role header
            content.push_str(&format!("[{}]", message.role.to_string().to_uppercase()));
            
            if self.config.include_timestamps {
                content.push_str(&format!(" - {}", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
            }
            content.push_str("\n");

            // Convert markdown code blocks to plain text
            let plain_content = self.markdown_to_plain_text(&message.content);
            content.push_str(&plain_content);
            content.push_str("\n\n");

            if i < messages.len() - 1 {
                content.push_str(&"-".repeat(50));
                content.push_str("\n\n");
            }
        }

        Ok(content)
    }

    /// Clean message content for export
    fn clean_message_content(&self, content: &str) -> String {
        // Remove any TermAI-specific thinking blocks if not included
        if !self.config.include_thinking {
            // This would remove <thinking> blocks if they exist
            // For now, just return the content as-is
            content.to_string()
        } else {
            content.to_string()
        }
    }

    /// Convert markdown to plain text
    fn markdown_to_plain_text(&self, markdown: &str) -> String {
        // Simple markdown to text conversion
        let mut text = markdown.to_string();
        
        // Remove code block markers
        text = text.replace("```rust", "");
        text = text.replace("```python", "");
        text = text.replace("```javascript", "");
        text = text.replace("```", "");
        
        // Remove markdown formatting
        text = text.replace("**", "");
        text = text.replace("*", "");
        text = text.replace("__", "");
        text = text.replace("_", "");
        
        // Clean up headers
        text = text.replace("### ", "");
        text = text.replace("## ", "");
        text = text.replace("# ", "");
        
        text
    }

    /// Get default CSS for HTML exports
    fn get_html_css(&self) -> String {
        if let Some(custom_css) = &self.config.custom_css {
            custom_css.clone()
        } else {
            r#"
body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    background-color: #fafafa;
}

.container {
    background: white;
    border-radius: 8px;
    padding: 30px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

header {
    border-bottom: 2px solid #e1e5e9;
    padding-bottom: 20px;
    margin-bottom: 30px;
    text-align: center;
}

header h1 {
    color: #2c3e50;
    margin: 0;
    font-size: 2.2em;
}

.export-info {
    color: #7f8c8d;
    font-size: 0.9em;
    margin: 10px 0 0 0;
}

h2 {
    color: #3498db;
    border-left: 4px solid #3498db;
    padding-left: 15px;
    margin-top: 30px;
}

h3 {
    color: #2c3e50;
}

pre {
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 4px;
    padding: 15px;
    overflow-x: auto;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
}

code {
    background: #f8f9fa;
    padding: 2px 6px;
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
}

blockquote {
    border-left: 4px solid #bdc3c7;
    padding-left: 15px;
    margin-left: 0;
    font-style: italic;
    color: #7f8c8d;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 20px 0;
}

table th, table td {
    border: 1px solid #ddd;
    padding: 12px;
    text-align: left;
}

table th {
    background-color: #f8f9fa;
    font-weight: bold;
}

table tr:nth-child(even) {
    background-color: #f9f9f9;
}

footer {
    border-top: 1px solid #e1e5e9;
    padding-top: 20px;
    margin-top: 40px;
    text-align: center;
    color: #7f8c8d;
    font-size: 0.9em;
}

footer a {
    color: #3498db;
    text-decoration: none;
}

hr {
    border: none;
    border-top: 2px solid #e1e5e9;
    margin: 30px 0;
}

.message-separator {
    text-align: center;
    margin: 30px 0;
    color: #bdc3c7;
}
"#.to_string()
        }
    }
}

/// Export a single response/message quickly
pub fn quick_export_message(
    content: &str,
    role: &Role,
    format: ExportFormat,
    output_path: &Path,
) -> Result<()> {
    let message = Message::new(
        uuid::Uuid::new_v4().to_string(),
        role.clone(),
        content.to_string(),
    );

    let exporter = ConversationExporter::new(ExportConfig::default());
    exporter.export_messages(&[message], format, output_path, Some("TermAI Response"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_markdown_export() {
        let messages = vec![
            Message::new(
                "1".to_string(),
                Role::User,
                "Hello, world!".to_string(),
            ),
            Message::new(
                "2".to_string(),
                Role::Assistant,
                "Hello! How can I help you today?".to_string(),
            ),
        ];

        let exporter = ConversationExporter::new(ExportConfig::default());
        let result = exporter.messages_to_markdown(&messages, Some("Test Conversation"));
        
        assert!(result.is_ok());
        let markdown = result.unwrap();
        assert!(markdown.contains("# Test Conversation"));
        assert!(markdown.contains("💬 USER"));
        assert!(markdown.contains("🤖 ASSISTANT"));
        assert!(markdown.contains("Hello, world!"));
        assert!(markdown.contains("Hello! How can I help you today?"));
    }

    #[test]
    fn test_json_export() {
        let messages = vec![
            Message::new(
                "1".to_string(),
                Role::User,
                "Test message".to_string(),
            ),
        ];

        let exporter = ConversationExporter::new(ExportConfig::default());
        let result = exporter.messages_to_json(&messages, Some("Test"));
        
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("Test message"));
        assert!(json.contains("user"));
    }

    #[test]
    fn test_export_formats() {
        let temp_dir = TempDir::new().unwrap();
        
        let message = Message::new(
            "1".to_string(),
            Role::Assistant,
            "Test content with `code`".to_string(),
        );

        for format in [ExportFormat::Markdown, ExportFormat::Json, ExportFormat::PlainText].iter().cloned() {
            let filename = format!("test.{}", match format {
                ExportFormat::Markdown => "md",
                ExportFormat::Json => "json",
                ExportFormat::PlainText => "txt",
                _ => "txt",
            });
            
            let output_path = temp_dir.path().join(filename);
            let exporter = ConversationExporter::new(ExportConfig::default());
            let result = exporter.export_messages(&[message.clone()], format, &output_path, Some("Test"));
            
            assert!(result.is_ok(), "Failed to export format: {:?}", format);
            assert!(output_path.exists(), "Export file was not created");
            
            let content = fs::read_to_string(&output_path).unwrap();
            assert!(!content.is_empty(), "Export file is empty");
        }
    }
}