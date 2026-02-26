use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Default timeout for command execution (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum output size to prevent memory issues (1MB)
const MAX_OUTPUT_SIZE: usize = 1024 * 1024;

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
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    /// Execute a bash command with timeout and output truncation
    async fn execute_bash(&self, arguments: &str) -> Result<ToolResult> {
        let args: BashArgs = serde_json::from_str(arguments)
            .map_err(|e| anyhow!("Invalid bash arguments: {}", e))?;

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
                output: format!("Successfully wrote {} bytes to '{}'", args.content.len(), args.path),
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
                            let file_type = if entry.path().is_dir() {
                                "dir"
                            } else {
                                "file"
                            };
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

    /// Resolve a path relative to the working directory and validate it
    fn resolve_path(&self, path_str: &str) -> Result<PathBuf> {
        let path = PathBuf::from(path_str);

        // If path is absolute, use it directly but validate it's within working directory
        let resolved = if path.is_absolute() {
            path
        } else {
            self.working_directory.join(&path)
        };

        // Canonicalize to resolve .. and symlinks (if the path exists)
        // For new files, we check the parent directory
        let canonical = if resolved.exists() {
            resolved.canonicalize()?
        } else if let Some(parent) = resolved.parent() {
            if parent.exists() {
                let canonical_parent = parent.canonicalize()?;
                canonical_parent.join(resolved.file_name().unwrap_or_default())
            } else {
                resolved
            }
        } else {
            resolved
        };

        // Security check: ensure path is within or equal to working directory
        let canonical_working = self.working_directory.canonicalize()?;
        if !canonical.starts_with(&canonical_working) {
            return Err(anyhow!(
                "Path '{}' is outside the working directory. Access denied for security.",
                path_str
            ));
        }

        Ok(canonical)
    }
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
