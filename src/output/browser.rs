use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tiny_http::{Server, Response, Header};
use colored::*;

use crate::output::export::{ConversationExporter, ExportConfig};
use crate::session::model::message::Message;

/// Browser preview server configuration
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub port: u16,
    pub host: String,
    pub auto_open: bool,
    pub auto_refresh: bool,
    pub custom_css: Option<String>,
    pub template_path: Option<PathBuf>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            auto_open: true,
            auto_refresh: true,
            custom_css: None,
            template_path: None,
        }
    }
}

/// Browser preview server for TermAI conversations
pub struct BrowserPreview {
    config: BrowserConfig,
    temp_dir: PathBuf,
    server_running: Arc<AtomicBool>,
    current_content: Arc<std::sync::Mutex<String>>,
}

impl BrowserPreview {
    pub fn new(config: BrowserConfig) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("termai_preview");
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            config,
            temp_dir,
            server_running: Arc::new(AtomicBool::new(false)),
            current_content: Arc::new(std::sync::Mutex::new(String::new())),
        })
    }

    /// Start the preview server and display content
    pub async fn preview_messages(
        &mut self,
        messages: &[Message],
        title: Option<&str>,
    ) -> Result<String> {
        // Generate HTML content
        let html_content = self.generate_html_content(messages, title)?;
        
        // Update current content
        {
            let mut content = self.current_content.lock().unwrap();
            *content = html_content;
        }

        // Start server if not running
        if !self.server_running.load(Ordering::Relaxed) {
            self.start_server().await?;
        }

        let url = format!("http://{}:{}", self.config.host, self.config.port);
        
        // Auto-open browser if configured
        if self.config.auto_open {
            self.open_browser(&url)?;
        }

        Ok(url)
    }

    /// Preview a single response in browser
    pub async fn preview_response(
        &mut self,
        content: &str,
        role: &crate::llm::common::model::role::Role,
        title: Option<&str>,
    ) -> Result<String> {
        let message = Message::new(
            uuid::Uuid::new_v4().to_string(),
            role.clone(),
            content.to_string(),
        );

        self.preview_messages(&[message], title).await
    }

    /// Start the HTTP server
    async fn start_server(&mut self) -> Result<()> {
        let address = format!("{}:{}", self.config.host, self.config.port);
        let server = Server::http(&address)
            .map_err(|e| anyhow!("Failed to start server on {}: {}", address, e))?;

        let server_running = Arc::clone(&self.server_running);
        let current_content = Arc::clone(&self.current_content);
        let config = self.config.clone();

        server_running.store(true, Ordering::Relaxed);

        println!("{}", format!("🌐 Starting preview server at http://{}", address).bright_cyan());

        // Spawn server thread
        thread::spawn(move || {
            for request in server.incoming_requests() {
                if !server_running.load(Ordering::Relaxed) {
                    break;
                }

                let _method = request.method().to_string();
                let url = request.url().to_string();

                let response = match url.as_str() {
                    "/" => {
                        let content = current_content.lock().unwrap().clone();
                        if content.is_empty() {
                            Response::from_string(Self::default_page())
                                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                        } else {
                            Response::from_string(content)
                                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                        }
                    }
                    "/api/refresh" => {
                        // Refresh endpoint for live updates
                        let content = current_content.lock().unwrap().clone();
                        Response::from_string(content)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
                            .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
                    }
                    "/static/refresh.js" => {
                        let js = if config.auto_refresh {
                            Self::refresh_script()
                        } else {
                            "// Auto-refresh disabled".to_string()
                        };
                        Response::from_string(js)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..]).unwrap())
                    }
                    "/favicon.ico" => {
                        Response::from_string("")
                            .with_status_code(404)
                    }
                    _ => {
                        Response::from_string("Not Found")
                            .with_status_code(404)
                    }
                };

                if let Err(e) = request.respond(response) {
                    eprintln!("Error responding to request: {}", e);
                }
            }
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Generate HTML content from messages
    fn generate_html_content(&self, messages: &[Message], title: Option<&str>) -> Result<String> {
        let exporter = ConversationExporter::new(ExportConfig {
            include_timestamps: true,
            include_metadata: true,
            custom_css: self.get_enhanced_css().into(),
            template: None,
            syntax_theme: "base16-ocean.dark".to_string(),
            include_thinking: false,
        });

        let mut html = exporter.messages_to_html(messages, title)?;

        // Add auto-refresh script if enabled
        if self.config.auto_refresh {
            html = html.replace(
                "</head>",
                "<script src=\"/static/refresh.js\"></script>\n</head>"
            );
        }

        // Add interactive features
        html = self.add_interactive_features(html);

        Ok(html)
    }

    /// Add interactive features to HTML
    fn add_interactive_features(&self, mut html: String) -> String {
        // Add copy buttons to code blocks
        html = html.replace(
            "<pre>",
            r#"<div class="code-container"><pre>"#
        );
        html = html.replace(
            "</pre>",
            r#"</pre><button class="copy-btn" onclick="copyCode(this)">📋 Copy</button></div>"#
        );

        // Add JavaScript for interactive features
        let interactive_js = r#"
<script>
function copyCode(button) {
    const codeBlock = button.previousElementSibling;
    const code = codeBlock.innerText;
    
    if (navigator.clipboard) {
        navigator.clipboard.writeText(code).then(() => {
            button.textContent = '✅ Copied!';
            setTimeout(() => {
                button.textContent = '📋 Copy';
            }, 2000);
        });
    } else {
        // Fallback for older browsers
        const textarea = document.createElement('textarea');
        textarea.value = code;
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        document.body.removeChild(textarea);
        
        button.textContent = '✅ Copied!';
        setTimeout(() => {
            button.textContent = '📋 Copy';
        }, 2000);
    }
}

// Keyboard shortcuts
document.addEventListener('keydown', function(e) {
    // Ctrl/Cmd + R to refresh
    if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
        e.preventDefault();
        location.reload();
    }
    
    // Escape to close (if running in electron or similar)
    if (e.key === 'Escape') {
        if (window.close) {
            window.close();
        }
    }
});

// Add smooth scrolling
document.documentElement.style.scrollBehavior = 'smooth';
</script>
"#;

        html = html.replace("</body>", &format!("{}\n</body>", interactive_js));

        html
    }

    /// Get enhanced CSS for browser preview
    fn get_enhanced_css(&self) -> String {
        if let Some(custom_css) = &self.config.custom_css {
            return custom_css.clone();
        }

        r#"
/* Enhanced TermAI Browser Preview CSS */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif;
    line-height: 1.7;
    color: #2d3748;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
}

.container {
    background: white;
    border-radius: 12px;
    padding: 40px;
    box-shadow: 0 20px 40px rgba(0,0,0,0.1);
    backdrop-filter: blur(10px);
    position: relative;
}

.container::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: linear-gradient(90deg, #667eea, #764ba2);
    border-radius: 12px 12px 0 0;
}

header {
    border-bottom: 2px solid #e2e8f0;
    padding-bottom: 30px;
    margin-bottom: 40px;
    text-align: center;
    position: relative;
}

header h1 {
    background: linear-gradient(135deg, #667eea, #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin: 0;
    font-size: 2.5em;
    font-weight: 700;
}

.export-info {
    color: #718096;
    font-size: 0.95em;
    margin: 15px 0 0 0;
    opacity: 0.8;
}

/* Message styling */
h2 {
    color: #4a5568;
    border-left: 4px solid #667eea;
    padding-left: 20px;
    margin: 40px 0 20px 0;
    font-size: 1.3em;
    display: flex;
    align-items: center;
    gap: 10px;
}

h2::before {
    font-size: 1.2em;
}

h3 {
    color: #2d3748;
    margin-top: 30px;
}

/* Code styling */
.code-container {
    position: relative;
    margin: 20px 0;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 6px rgba(0,0,0,0.1);
}

pre {
    background: linear-gradient(135deg, #1a202c, #2d3748);
    border: none;
    border-radius: 8px;
    padding: 20px;
    overflow-x: auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
    font-size: 0.9em;
    line-height: 1.5;
    color: #e2e8f0;
    margin: 0;
}

code {
    background: #f7fafc;
    padding: 3px 6px;
    border-radius: 4px;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 0.9em;
    color: #e53e3e;
    border: 1px solid #e2e8f0;
}

.copy-btn {
    position: absolute;
    top: 10px;
    right: 10px;
    background: rgba(255,255,255,0.9);
    border: none;
    border-radius: 6px;
    padding: 8px 12px;
    font-size: 0.8em;
    cursor: pointer;
    transition: all 0.2s ease;
    backdrop-filter: blur(10px);
    font-family: inherit;
}

.copy-btn:hover {
    background: rgba(255,255,255,1);
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.1);
}

.copy-btn:active {
    transform: translateY(0);
}

/* Tables */
table {
    border-collapse: separate;
    border-spacing: 0;
    width: 100%;
    margin: 20px 0;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 4px 6px rgba(0,0,0,0.1);
}

table th {
    background: linear-gradient(135deg, #667eea, #764ba2);
    color: white;
    font-weight: 600;
    padding: 15px;
    text-align: left;
    border: none;
}

table td {
    background: #f7fafc;
    padding: 12px 15px;
    border: none;
    border-bottom: 1px solid #e2e8f0;
}

table tr:last-child td {
    border-bottom: none;
}

table tr:nth-child(even) td {
    background: #edf2f7;
}

/* Blockquotes */
blockquote {
    border-left: 4px solid #667eea;
    padding: 20px;
    margin: 20px 0;
    background: #f7fafc;
    font-style: italic;
    color: #4a5568;
    border-radius: 0 8px 8px 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

/* Lists */
ul, ol {
    padding-left: 30px;
    margin: 15px 0;
}

li {
    margin: 8px 0;
    line-height: 1.6;
}

/* Links */
a {
    color: #667eea;
    text-decoration: none;
    border-bottom: 1px solid transparent;
    transition: border-color 0.2s ease;
}

a:hover {
    border-bottom-color: #667eea;
}

/* Separators */
hr {
    border: none;
    border-top: 2px solid #e2e8f0;
    margin: 40px 0;
    border-radius: 1px;
}

/* Footer */
footer {
    border-top: 1px solid #e2e8f0;
    padding-top: 30px;
    margin-top: 50px;
    text-align: center;
    color: #718096;
    font-size: 0.9em;
}

footer a {
    color: #667eea;
    font-weight: 500;
}

/* Responsive design */
@media (max-width: 768px) {
    body {
        padding: 10px;
    }
    
    .container {
        padding: 20px;
    }
    
    header h1 {
        font-size: 2em;
    }
    
    pre {
        padding: 15px;
        font-size: 0.8em;
    }
    
    .copy-btn {
        position: static;
        margin-top: 10px;
        width: 100%;
    }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
    body {
        background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
    }
    
    .container {
        background: #2d3748;
        color: #e2e8f0;
    }
    
    h2, h3 {
        color: #e2e8f0;
    }
    
    code {
        background: #4a5568;
        color: #fbb6ce;
        border-color: #4a5568;
    }
    
    table td {
        background: #4a5568;
        border-color: #2d3748;
        color: #e2e8f0;
    }
    
    table tr:nth-child(even) td {
        background: #3a424e;
    }
    
    blockquote {
        background: #4a5568;
        color: #e2e8f0;
    }
}

/* Animation */
.container {
    animation: fadeIn 0.5s ease-out;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(20px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* Print styles */
@media print {
    body {
        background: white !important;
    }
    
    .container {
        box-shadow: none !important;
        background: white !important;
    }
    
    .copy-btn {
        display: none !important;
    }
    
    pre {
        background: #f8f9fa !important;
        color: #333 !important;
    }
}
"#.to_string()
    }

    /// Generate auto-refresh JavaScript
    fn refresh_script() -> String {
        r#"
// Auto-refresh script for TermAI preview
let refreshInterval = 2000; // 2 seconds
let lastContent = '';

function checkForUpdates() {
    fetch('/api/refresh')
        .then(response => response.text())
        .then(content => {
            if (content !== lastContent && lastContent !== '') {
                // Content has changed, reload the page
                location.reload();
            }
            lastContent = content;
        })
        .catch(error => {
            console.log('Auto-refresh error:', error);
            // Increase interval on error
            refreshInterval = Math.min(refreshInterval * 1.5, 10000);
        });
}

// Start auto-refresh
setInterval(checkForUpdates, refreshInterval);

// Also check when page becomes visible again
document.addEventListener('visibilitychange', function() {
    if (!document.hidden) {
        checkForUpdates();
    }
});
"#.to_string()
    }

    /// Generate default page content
    fn default_page() -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TermAI Preview Server</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            margin: 0;
            padding: 40px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            background: white;
            border-radius: 12px;
            padding: 40px;
            text-align: center;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2d3748;
            margin-bottom: 20px;
        }
        p {
            color: #718096;
            line-height: 1.6;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 TermAI Preview Server</h1>
        <p>Server is running and waiting for content...</p>
        <p>Use TermAI to preview conversations here.</p>
    </div>
</body>
</html>"#.to_string()
    }

    /// Open browser to the preview URL
    fn open_browser(&self, url: &str) -> Result<()> {
        println!("{}", format!("🚀 Opening browser at {}", url).bright_green());

        let command = if cfg!(target_os = "windows") {
            ("cmd", vec!["/c", "start", url])
        } else if cfg!(target_os = "macos") {
            ("open", vec![url])
        } else {
            // Linux and other Unix-like systems
            ("xdg-open", vec![url])
        };

        Command::new(command.0)
            .args(&command.1)
            .spawn()
            .map_err(|e| anyhow!("Failed to open browser: {}", e))?;

        Ok(())
    }

    /// Stop the preview server
    pub fn stop_server(&self) {
        self.server_running.store(false, Ordering::Relaxed);
        println!("{}", "🛑 Preview server stopped".bright_yellow());
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.server_running.load(Ordering::Relaxed)
    }

    /// Export current content to file
    pub fn export_to_file(&self, output_path: &Path) -> Result<()> {
        let content = self.current_content.lock().unwrap().clone();
        fs::write(output_path, content)?;
        Ok(())
    }
}

impl Drop for BrowserPreview {
    fn drop(&mut self) {
        self.stop_server();
        // Clean up temp directory
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::common::model::role::Role;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.auto_open);
    }

    #[tokio::test]
    async fn test_html_generation() {
        let config = BrowserConfig::default();
        let preview = BrowserPreview::new(config).unwrap();

        let messages = vec![
            Message::new(
                "1".to_string(),
                Role::User,
                "Hello, world!".to_string(),
            ),
            Message::new(
                "2".to_string(),
                Role::Assistant,
                "Hello! How can I help you?".to_string(),
            ),
        ];

        let html = preview.generate_html_content(&messages, Some("Test Chat")).unwrap();
        
        assert!(html.contains("Test Chat"));
        assert!(html.contains("Hello, world!"));
        assert!(html.contains("Hello! How can I help you?"));
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("copy-btn"));
    }

    #[test]
    fn test_interactive_features() {
        let config = BrowserConfig::default();
        let preview = BrowserPreview::new(config).unwrap();

        let html = "<pre>some code</pre>".to_string();
        let enhanced = preview.add_interactive_features(html);

        assert!(enhanced.contains("code-container"));
        assert!(enhanced.contains("copy-btn"));
        assert!(enhanced.contains("copyCode"));
    }
}