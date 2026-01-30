use serde_json::json;
use crate::llm::openai::model::responses_api::{FunctionTool, Tool};

/// Create the bash tool definition for executing shell commands
pub fn bash_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "bash".to_string(),
        description: "Execute a bash command and return stdout/stderr. Use this to run shell commands, scripts, or system utilities.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        }),
    }
}

/// Create the read_file tool definition for reading file contents
pub fn read_file_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "read_file".to_string(),
        description: "Read the contents of a file at the specified path. Returns the file contents as a string.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read (relative to working directory or absolute)"
                }
            },
            "required": ["path"]
        }),
    }
}

/// Create the write_file tool definition for writing file contents
pub fn write_file_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "write_file".to_string(),
        description: "Write content to a file at the specified path. Creates the file if it doesn't exist, or overwrites if it does.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write (relative to working directory or absolute)"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

/// Create the list_files tool definition for listing directory contents
pub fn list_files_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "list_files".to_string(),
        description: "List the contents of a directory. Returns file and directory names.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "directory": {
                    "type": "string",
                    "description": "The directory path to list (relative to working directory or absolute). Use '.' for current directory."
                }
            },
            "required": ["directory"]
        }),
    }
}

/// Get all enabled tools as a vector
pub fn get_enabled_tools() -> Vec<Tool> {
    vec![
        Tool::Function(bash_tool()),
        Tool::Function(read_file_tool()),
        Tool::Function(write_file_tool()),
        Tool::Function(list_files_tool()),
    ]
}
