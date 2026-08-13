use crate::llm::openai::model::responses_api::{FunctionTool, Tool};
use serde_json::json;

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
        description: "List the contents of a directory. Returns file and directory names."
            .to_string(),
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

/// Create the web_search tool definition for searching the web
pub fn web_search_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "web_search".to_string(),
        description:
            "Search the web for information. Returns a list of result titles, URLs and snippets."
                .to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                }
            },
            "required": ["query"]
        }),
    }
}

/// Create the web_fetch tool definition for fetching a URL
pub fn web_fetch_tool() -> FunctionTool {
    FunctionTool {
        tool_type: "function".to_string(),
        name: "web_fetch".to_string(),
        description: "Fetch the contents of a web page by URL (http/https only). HTML is converted to plain text.".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The http or https URL to fetch"
                }
            },
            "required": ["url"]
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
        Tool::Function(web_search_tool()),
        Tool::Function(web_fetch_tool()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_search_tool_definition() {
        let tool = web_search_tool();
        assert_eq!(tool.name, "web_search");
        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.parameters["required"][0], "query");
        assert!(tool.parameters["properties"]["query"].is_object());
    }

    #[test]
    fn test_web_fetch_tool_definition() {
        let tool = web_fetch_tool();
        assert_eq!(tool.name, "web_fetch");
        assert_eq!(tool.tool_type, "function");
        assert_eq!(tool.parameters["required"][0], "url");
        assert!(tool.parameters["properties"]["url"].is_object());
    }

    #[test]
    fn test_enabled_tools_include_web_tools() {
        let tools = get_enabled_tools();
        let names: Vec<String> = tools
            .iter()
            .filter_map(|t| match t {
                Tool::Function(f) => Some(f.name.clone()),
                _ => None,
            })
            .collect();
        for expected in [
            "bash",
            "read_file",
            "write_file",
            "list_files",
            "web_search",
            "web_fetch",
        ] {
            assert!(
                names.contains(&expected.to_string()),
                "missing {}",
                expected
            );
        }
    }
}
