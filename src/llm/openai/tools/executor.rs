use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::path::{Component, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Default timeout for command execution (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum output size to prevent memory issues (1MB)
const MAX_OUTPUT_SIZE: usize = 1024 * 1024;

/// Timeout for web requests (15 seconds)
const WEB_TIMEOUT_SECS: u64 = 15;

/// Maximum characters returned from web tools before truncation
const MAX_WEB_OUTPUT_CHARS: usize = 50_000;

/// Environment variable that skips the interactive bash confirmation prompt
const AUTO_APPROVE_ENV: &str = "TERMAI_TOOL_AUTO_APPROVE";

/// Result of executing a tool
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

/// Tool executor that handles running tools within a working directory
pub struct ToolExecutor {
    working_directory: PathBuf,
}

#[derive(Deserialize)]
struct BashArgs {
    command: String,
}

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
}

#[derive(Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct ListFilesArgs {
    directory: String,
}

#[derive(Deserialize)]
struct WebSearchArgs {
    query: String,
}

#[derive(Deserialize)]
struct WebFetchArgs {
    url: String,
}

impl ToolExecutor {
    /// Create a new tool executor with the specified working directory
    pub fn new(working_directory: PathBuf) -> Self {
        Self { working_directory }
    }

    /// Execute a tool by name with the given JSON arguments
    pub async fn execute(&self, tool_name: &str, arguments: &str) -> Result<ToolResult> {
        match tool_name {
            "bash" => self.execute_bash(arguments).await,
            "read_file" => self.execute_read_file(arguments),
            "write_file" => self.execute_write_file(arguments),
            "list_files" => self.execute_list_files(arguments),
            "web_search" => self.execute_web_search(arguments).await,
            "web_fetch" => self.execute_web_fetch(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    /// Execute a bash command with per-command confirmation, timeout and output truncation
    async fn execute_bash(&self, arguments: &str) -> Result<ToolResult> {
        let args: BashArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid bash arguments: {}", e))?;

        // Security gate: require explicit user confirmation before running
        // model-provided shell commands. TERMAI_TOOL_AUTO_APPROVE=1 skips the
        // prompt for non-interactive use (tests, CI).
        if !Self::auto_approve_enabled() {
            println!("\nThe AI wants to run the following command:");
            println!("  {}", args.command);
            let approved = dialoguer::Confirm::new()
                .with_prompt("Allow this command to run?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !approved {
                return Ok(ToolResult {
                    success: false,
                    output: "Command execution declined by user".to_string(),
                });
            }
        }

        let command_future = async {
            let output = Command::new("bash")
                .arg("-c")
                .arg(&args.command)
                .current_dir(&self.working_directory)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str("[stderr]\n");
                result.push_str(&stderr);
            }

            Ok::<(bool, String), std::io::Error>((output.status.success(), result))
        };

        match timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), command_future).await {
            Ok(Ok((success, output))) => Ok(ToolResult {
                success,
                output: truncate_output(output),
            }),
            Ok(Err(e)) => Ok(ToolResult {
                success: false,
                output: format!("Command execution failed: {}", e),
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                output: format!("Command timed out after {} seconds", DEFAULT_TIMEOUT_SECS),
            }),
        }
    }

    /// Read file contents with path validation
    fn execute_read_file(&self, arguments: &str) -> Result<ToolResult> {
        let args: ReadFileArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid read_file arguments: {}", e))?;

        let path = self.resolve_path(&args.path)?;

        match std::fs::read_to_string(&path) {
            Ok(content) => Ok(ToolResult {
                success: true,
                output: truncate_output(content),
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: format!("Failed to read file '{}': {}", args.path, e),
            }),
        }
    }

    /// Write content to a file with path validation
    fn execute_write_file(&self, arguments: &str) -> Result<ToolResult> {
        let args: WriteFileArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid write_file arguments: {}", e))?;

        let path = self.resolve_path(&args.path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Ok(ToolResult {
                        success: false,
                        output: format!("Failed to create parent directories: {}", e),
                    });
                }
            }
        }

        match std::fs::write(&path, &args.content) {
            Ok(()) => Ok(ToolResult {
                success: true,
                output: format!(
                    "Successfully wrote {} bytes to '{}'",
                    args.content.len(),
                    args.path
                ),
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: format!("Failed to write file '{}': {}", args.path, e),
            }),
        }
    }

    /// List directory contents with path validation
    fn execute_list_files(&self, arguments: &str) -> Result<ToolResult> {
        let args: ListFilesArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid list_files arguments: {}", e))?;

        let path = self.resolve_path(&args.directory)?;

        match std::fs::read_dir(&path) {
            Ok(entries) => {
                let mut files = Vec::new();
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let file_type = if entry.path().is_dir() { "dir" } else { "file" };
                            files.push(format!("[{}] {}", file_type, name));
                        }
                        Err(e) => {
                            files.push(format!("[error] {}", e));
                        }
                    }
                }
                files.sort();
                Ok(ToolResult {
                    success: true,
                    output: if files.is_empty() {
                        "(empty directory)".to_string()
                    } else {
                        files.join("\n")
                    },
                })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: format!("Failed to list directory '{}': {}", args.directory, e),
            }),
        }
    }

    /// Resolve a path relative to the working directory and validate it.
    ///
    /// Security: walk up to the deepest EXISTING ancestor, canonicalize it
    /// (resolving symlinks and `..`), verify that ancestor is inside the
    /// canonicalized working directory, and require that all remaining
    /// (not-yet-existing) components are plain path segments (no `..`/`.`).
    /// This prevents traversal escapes through nonexistent parents such as
    /// `newdir/../../outside/file`.
    fn resolve_path(&self, path_str: &str) -> Result<PathBuf> {
        let path = PathBuf::from(path_str);

        // If path is absolute, use it directly but validate it's within working directory
        let resolved = if path.is_absolute() {
            path
        } else {
            self.working_directory.join(&path)
        };

        let canonical_working = self.working_directory.canonicalize()?;
        let denied = || {
            anyhow!(
                "Path '{}' is outside the working directory. Access denied for security.",
                path_str
            )
        };

        // Find the deepest existing ancestor prefix of the resolved path
        let components: Vec<Component> = resolved.components().collect();
        let mut existing_prefix = PathBuf::new();
        let mut split_idx = 0;
        for (i, comp) in components.iter().enumerate() {
            let candidate = existing_prefix.join(comp);
            if candidate.exists() {
                existing_prefix = candidate;
                split_idx = i + 1;
            } else {
                break;
            }
        }

        if split_idx == 0 {
            return Err(denied());
        }

        // Canonicalize the existing ancestor (resolves symlinks and any `..`)
        let canonical_prefix = existing_prefix.canonicalize()?;
        if !canonical_prefix.starts_with(&canonical_working) {
            return Err(denied());
        }

        // The remaining, not-yet-existing components must all be normal
        // segments: no `..`, `.`, root or prefix components allowed.
        let mut result = canonical_prefix;
        for comp in &components[split_idx..] {
            match comp {
                Component::Normal(segment) => result.push(segment),
                _ => return Err(denied()),
            }
        }

        Ok(result)
    }

    /// Check whether the auto-approve escape hatch is enabled
    fn auto_approve_enabled() -> bool {
        std::env::var(AUTO_APPROVE_ENV).as_deref() == Ok("1")
    }

    /// Build an HTTP client for web tools (15s timeout, redirects, termai UA)
    fn web_client() -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder()
            .timeout(Duration::from_secs(WEB_TIMEOUT_SECS))
            .user_agent("termai")
            .build()?)
    }

    /// Fetch a URL and return its contents as text (HTML converted to plain text)
    async fn execute_web_fetch(&self, arguments: &str) -> Result<ToolResult> {
        let args: WebFetchArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid web_fetch arguments: {}", e))?;

        if let Err(message) = validate_web_url(&args.url) {
            return Ok(ToolResult {
                success: false,
                output: message,
            });
        }

        let client = Self::web_client()?;
        let response = match client.get(&args.url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: format!("Failed to fetch '{}': {}", args.url, e),
                })
            }
        };

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: format!("Failed to read response body from '{}': {}", args.url, e),
                })
            }
        };

        // Cap the body at 1MB before conversion
        let body = &bytes[..bytes.len().min(MAX_OUTPUT_SIZE)];

        let text = if content_type.contains("html") {
            html2text::from_read(body, 100)
        } else {
            String::from_utf8_lossy(body).to_string()
        };

        Ok(ToolResult {
            success: status.is_success(),
            output: truncate_web_output(format!("[HTTP {}] {}\n\n{}", status, args.url, text)),
        })
    }

    /// Search the web via the DuckDuckGo HTML endpoint (PoC)
    async fn execute_web_search(&self, arguments: &str) -> Result<ToolResult> {
        let args: WebSearchArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid web_search arguments: {}", e))?;

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            url_encode(&args.query)
        );

        let client = Self::web_client()?;
        let response = match client.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: format!(
                        "Web search failed (network error while contacting DuckDuckGo): {}",
                        e
                    ),
                })
            }
        };

        let html = match response.text().await {
            Ok(html) => html,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: format!("Web search failed (could not read response): {}", e),
                })
            }
        };

        let results = parse_search_results(&html);
        if results.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: format!("No results found for '{}'", args.query),
            });
        }

        let mut output = format!("Search results for '{}':\n", args.query);
        for (i, result) in results.iter().take(8).enumerate() {
            output.push_str(&format!(
                "\n{}. {}\n   {}\n",
                i + 1,
                result.title,
                result.url
            ));
            if !result.snippet.is_empty() {
                output.push_str(&format!("   {}\n", result.snippet));
            }
        }

        Ok(ToolResult {
            success: true,
            output: truncate_web_output(output),
        })
    }
}

/// A single parsed web search result
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

/// Validate that a URL uses the http or https scheme
fn validate_web_url(url: &str) -> std::result::Result<(), String> {
    let lower = url.trim().to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        Ok(())
    } else {
        Err(format!(
            "Refusing to fetch '{}': only http:// and https:// URLs are supported",
            url
        ))
    }
}

/// Parse DuckDuckGo HTML search results (result__a links + result__snippet)
fn parse_search_results(html: &str) -> Vec<SearchResult> {
    let link_re =
        regex::Regex::new(r#"(?s)<a[^>]*class="result__a"[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#)
            .expect("valid link regex");
    let snippet_re = regex::Regex::new(r#"(?s)class="result__snippet"[^>]*>(.*?)</a>"#)
        .expect("valid snippet regex");

    let snippets: Vec<String> = snippet_re
        .captures_iter(html)
        .map(|c| strip_html(&c[1]))
        .collect();

    link_re
        .captures_iter(html)
        .enumerate()
        .map(|(i, c)| SearchResult {
            url: clean_result_url(&c[1]),
            title: strip_html(&c[2]),
            snippet: snippets.get(i).cloned().unwrap_or_default(),
        })
        .collect()
}

/// DuckDuckGo wraps result URLs in a redirect (`/l/?uddg=<encoded>`); unwrap it
fn clean_result_url(href: &str) -> String {
    if let Some(pos) = href.find("uddg=") {
        let rest = &href[pos + 5..];
        let end = rest.find('&').unwrap_or(rest.len());
        percent_decode(&rest[..end])
    } else {
        href.to_string()
    }
}

/// Remove HTML tags and decode common entities
fn strip_html(fragment: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").expect("valid tag regex");
    let text = tag_re.replace_all(fragment, "");
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}

/// Minimal percent-encoding for URL query values
fn url_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// Minimal percent-decoding for unwrapping redirect URLs
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// Truncate web tool output to a manageable size for the model
fn truncate_web_output(output: String) -> String {
    if output.len() <= MAX_WEB_OUTPUT_CHARS {
        return output;
    }
    let mut end = MAX_WEB_OUTPUT_CHARS;
    while !output.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n\n[truncated]", &output[..end])
}

/// Truncate output to prevent memory issues
fn truncate_output(output: String) -> String {
    if output.len() > MAX_OUTPUT_SIZE {
        let truncated = &output[..MAX_OUTPUT_SIZE];
        format!(
            "{}\n\n[Output truncated: {} bytes total, showing first {} bytes]",
            truncated,
            output.len(),
            MAX_OUTPUT_SIZE
        )
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that mutate the TERMAI_TOOL_AUTO_APPROVE env var
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct AutoApproveGuard;

    impl AutoApproveGuard {
        fn enable() -> Self {
            std::env::set_var(AUTO_APPROVE_ENV, "1");
            AutoApproveGuard
        }
    }

    impl Drop for AutoApproveGuard {
        fn drop(&mut self) {
            std::env::remove_var(AUTO_APPROVE_ENV);
        }
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_bash_runs_when_auto_approve_enabled() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = AutoApproveGuard::enable();

        let workdir = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(workdir.path().to_path_buf());
        let args = serde_json::json!({"command": "echo hello"}).to_string();

        let result = executor.execute("bash", &args).await.unwrap();
        assert!(result.success);
        assert!(result.output.contains("hello"));
    }

    #[test]
    fn test_auto_approve_disabled_by_default() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var(AUTO_APPROVE_ENV);
        assert!(!ToolExecutor::auto_approve_enabled());

        let _guard = AutoApproveGuard::enable();
        assert!(ToolExecutor::auto_approve_enabled());
    }

    #[tokio::test]
    async fn test_web_fetch_rejects_non_http_schemes() {
        let workdir = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(workdir.path().to_path_buf());

        for url in [
            "file:///etc/passwd",
            "ftp://example.com/file",
            "javascript:alert(1)",
            "gopher://example.com",
            "example.com/no-scheme",
        ] {
            let args = serde_json::json!({ "url": url }).to_string();
            let result = executor.execute("web_fetch", &args).await.unwrap();
            assert!(!result.success, "scheme should be rejected: {}", url);
            assert!(
                result.output.contains("only http:// and https://"),
                "unexpected rejection message for {}: {}",
                url,
                result.output
            );
        }
    }

    #[test]
    fn test_parse_search_results_from_fixture() {
        let fixture = r#"
<div class="result">
  <a rel="nofollow" class="result__a" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust%2Dlang.org%2F&amp;rut=abc">Rust Programming <b>Language</b></a>
  <a class="result__snippet" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fwww.rust%2Dlang.org%2F">A language empowering everyone to build reliable software.</a>
</div>
<div class="result">
  <a rel="nofollow" class="result__a" href="https://doc.rust-lang.org/book/">The Rust Book</a>
  <a class="result__snippet" href="https://doc.rust-lang.org/book/">Learn Rust &amp; more.</a>
</div>
"#;

        let results = parse_search_results(fixture);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://www.rust-lang.org/");
        assert_eq!(
            results[0].snippet,
            "A language empowering everyone to build reliable software."
        );

        assert_eq!(results[1].title, "The Rust Book");
        assert_eq!(results[1].url, "https://doc.rust-lang.org/book/");
        assert_eq!(results[1].snippet, "Learn Rust & more.");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("rust lang"), "rust+lang");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("safe-chars_.~"), "safe-chars_.~");
    }

    #[test]
    fn test_truncate_web_output() {
        let short = "hello".to_string();
        assert_eq!(truncate_web_output(short.clone()), short);

        let long = "x".repeat(MAX_WEB_OUTPUT_CHARS + 100);
        let truncated = truncate_web_output(long);
        assert!(truncated.ends_with("[truncated]"));
        assert!(truncated.len() <= MAX_WEB_OUTPUT_CHARS + 20);
    }

    #[tokio::test]
    #[ignore] // Ignore by default since these require network access
    async fn test_web_fetch_real_request() {
        let workdir = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(workdir.path().to_path_buf());
        let args = serde_json::json!({"url": "https://example.com"}).to_string();

        let result = executor.execute("web_fetch", &args).await.unwrap();
        assert!(result.success, "fetch failed: {}", result.output);
        assert!(result.output.contains("Example Domain"));
    }

    #[tokio::test]
    #[ignore] // Ignore by default since these require network access
    async fn test_web_search_real_request() {
        let workdir = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(workdir.path().to_path_buf());
        let args = serde_json::json!({"query": "rust programming language"}).to_string();

        let result = executor.execute("web_search", &args).await.unwrap();
        assert!(result.success, "search failed: {}", result.output);
    }

    /// Regression test: writing to a path whose (nonexistent) parent uses `..`
    /// to escape the working directory must be rejected.
    #[tokio::test]
    async fn test_write_file_rejects_traversal_via_nonexistent_parent() {
        let workdir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();

        // Canonicalize so lexical prefix checks can't be masked by symlinks
        // (macOS /var -> /private/var)
        let workdir_path = workdir.path().canonicalize().unwrap();
        let executor = ToolExecutor::new(workdir_path.clone());

        // Path like "missing/../../<outside>/evil.txt" — parent doesn't exist,
        // so the old lexical fallback let it through.
        let escape_target = format!(
            "missing/..{}/pwned/evil.txt",
            "/..".repeat(workdir_path.components().count())
        );
        let args = serde_json::json!({
            "path": escape_target,
            "content": "owned"
        })
        .to_string();

        let result = executor.execute("write_file", &args).await;

        // Must be rejected (either an Err or an unsuccessful ToolResult)
        match result {
            Err(_) => {}
            Ok(r) => assert!(
                !r.success,
                "write_file escaped the working directory: {}",
                r.output
            ),
        }

        // And nothing may have been created outside the working dir
        assert!(
            !PathBuf::from("/pwned").exists(),
            "traversal write created a directory outside the working dir"
        );

        drop(outside);
    }

    #[tokio::test]
    async fn test_write_file_relative_traversal_to_sibling_rejected() {
        let parent = tempfile::tempdir().unwrap();
        let workdir = parent.path().join("work");
        std::fs::create_dir_all(&workdir).unwrap();
        let workdir = workdir.canonicalize().unwrap();

        let executor = ToolExecutor::new(workdir.clone());
        let args = serde_json::json!({
            "path": "newdir/../../escaped/evil.txt",
            "content": "owned"
        })
        .to_string();

        let result = executor.execute("write_file", &args).await;
        match result {
            Err(_) => {}
            Ok(r) => assert!(!r.success, "traversal write succeeded: {}", r.output),
        }
        assert!(
            !workdir.parent().unwrap().join("escaped").exists(),
            "traversal created files outside the working directory"
        );
        drop(parent);
    }

    #[tokio::test]
    async fn test_write_file_to_new_nested_dir_inside_workdir_allowed() {
        let workdir = tempfile::tempdir().unwrap();
        let executor = ToolExecutor::new(workdir.path().canonicalize().unwrap());
        let args = serde_json::json!({
            "path": "sub/dir/file.txt",
            "content": "hello"
        })
        .to_string();

        let result = executor.execute("write_file", &args).await.unwrap();
        assert!(
            result.success,
            "legit nested write failed: {}",
            result.output
        );
        assert!(workdir.path().join("sub/dir/file.txt").exists());
    }
}
