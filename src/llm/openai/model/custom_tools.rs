use serde::{Serialize, Deserialize};

/// Custom tool definition for GPT-5
/// Allows freeform text input instead of structured JSON
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CustomTool {
    /// Tool type - always "custom"
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Tool name
    pub name: String,
    
    /// Tool description that guides the model on when and how to use it
    pub description: String,
    
    /// Optional context-free grammar to constrain outputs
    pub grammar: Option<String>,
}

impl CustomTool {
    /// Create a new custom tool
    pub fn new(name: String, description: String) -> Self {
        Self {
            tool_type: "custom".to_string(),
            name,
            description,
            grammar: None,
        }
    }

    /// Create a custom tool with grammar constraints
    pub fn with_grammar(name: String, description: String, grammar: String) -> Self {
        Self {
            tool_type: "custom".to_string(),
            name,
            description,
            grammar: Some(grammar),
        }
    }
}

/// Tool choice configuration for allowed tools
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AllowedToolsChoice {
    #[serde(rename = "type")]
    pub choice_type: String, // "allowed_tools"
    
    /// Mode: "auto" (model may pick any) or "required" (model must invoke one)
    pub mode: AllowedToolsMode,
    
    /// Subset of tools that can be used
    pub tools: Vec<AllowedToolReference>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AllowedToolsMode {
    Auto,
    Required,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AllowedToolReference {
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Tool name (for function tools)
    pub name: Option<String>,
    
    /// Server label (for MCP tools)
    pub server_label: Option<String>,
}

#[allow(dead_code)]
impl AllowedToolReference {
    pub fn function(name: String) -> Self {
        Self {
            tool_type: "function".to_string(),
            name: Some(name),
            server_label: None,
        }
    }

    pub fn custom(name: String) -> Self {
        Self {
            tool_type: "custom".to_string(),
            name: Some(name),
            server_label: None,
        }
    }

    pub fn mcp(server_label: String) -> Self {
        Self {
            tool_type: "mcp".to_string(),
            name: None,
            server_label: Some(server_label),
        }
    }

    pub fn image_generation() -> Self {
        Self {
            tool_type: "image_generation".to_string(),
            name: None,
            server_label: None,
        }
    }
}

/// Preamble configuration for tool calls
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PreambleConfig {
    /// Whether to enable preambles (explanations before tool calls)
    pub enabled: bool,
    
    /// Custom instruction for preambles (optional)
    pub instruction: Option<String>,
}

impl Default for PreambleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            instruction: None,
        }
    }
}

#[allow(dead_code)]
impl PreambleConfig {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            instruction: Some("Before you call a tool, explain why you are calling it.".to_string()),
        }
    }

    pub fn with_instruction(instruction: String) -> Self {
        Self {
            enabled: true,
            instruction: Some(instruction),
        }
    }
}