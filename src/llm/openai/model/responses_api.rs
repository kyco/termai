use serde::{Serialize, Deserialize};
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use crate::llm::openai::model::verbosity::Verbosity;
use crate::llm::openai::model::custom_tools::{CustomTool, AllowedToolsChoice};

/// GPT-5 Responses API request structure
/// Optimized for reasoning models with chain of thought support
#[derive(Serialize, Debug, Clone)]
pub struct ResponsesRequest {
    /// Model to use
    pub model: String,
    
    /// Input text or message
    pub input: String,
    
    /// Reasoning configuration
    pub reasoning: Option<ReasoningConfig>,
    
    /// Text generation configuration
    pub text: Option<TextConfig>,
    
    /// Tools available to the model
    pub tools: Option<Vec<Tool>>,
    
    /// Tool choice configuration
    pub tool_choice: Option<ToolChoice>,
    
    /// Store conversation for future reference
    pub store: Option<bool>,
    
    /// Previous response ID for multi-turn conversations
    pub previous_response_id: Option<String>,
    
    /// Metadata for the request
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ReasoningConfig {
    /// Reasoning effort level
    pub effort: ReasoningEffort,
    
    /// Include encrypted reasoning content (for ZDR mode)
    pub encrypted_content: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct TextConfig {
    /// Verbosity level for the response
    pub verbosity: Verbosity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Tool {
    Custom(CustomTool),
    Function(FunctionTool),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionTool {
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    pub function: FunctionDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ToolChoice {
    Auto(String), // "auto"
    None(String), // "none" 
    Required(String), // "required"
    AllowedTools(AllowedToolsChoice),
}

/// GPT-5 Responses API response structure
#[derive(Deserialize, Debug, Clone)]
pub struct ResponsesResponse {
    /// Response ID for reference
    pub id: String,
    
    /// Object type
    pub object: String,
    
    /// Model used
    pub model: String,
    
    /// Response choices
    pub choices: Vec<ResponseChoice>,
    
    /// Usage statistics
    pub usage: Option<ResponseUsage>,
    
    /// Reasoning items for multi-turn conversations
    pub reasoning: Option<Vec<ReasoningItem>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseChoice {
    /// Choice index
    pub index: usize,
    
    /// Response content
    pub message: ResponseMessage,
    
    /// Finish reason
    pub finish_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ResponseMessage {
    /// Message role
    pub role: String,
    
    /// Message content
    pub content: Option<String>,
    
    /// Tool calls if any
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    
    /// Function details (for function tools)
    pub function: Option<ToolCallFunction>,
    
    /// Custom tool input (for custom tools)
    pub input: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ResponseUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    
    /// Completion tokens
    pub completion_tokens: u32,
    
    /// Total tokens
    pub total_tokens: u32,
    
    /// Reasoning tokens (for reasoning models)
    pub reasoning_tokens: Option<u32>,
    
    /// Completion token details
    pub completion_tokens_details: Option<ResponseCompletionTokenDetails>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ResponseCompletionTokenDetails {
    /// Reasoning tokens used
    pub reasoning_tokens: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ReasoningItem {
    /// Reasoning content
    pub content: Option<String>,
    
    /// Encrypted reasoning content (for ZDR mode)
    pub encrypted_content: Option<String>,
    
    /// Item type
    #[serde(rename = "type")]
    pub item_type: Option<String>,
}

#[allow(dead_code)]
impl ResponsesRequest {
    /// Create a simple text request
    pub fn simple(model: String, input: String) -> Self {
        Self {
            model,
            input,
            reasoning: Some(ReasoningConfig {
                effort: ReasoningEffort::Medium,
                encrypted_content: None,
            }),
            text: Some(TextConfig {
                verbosity: Verbosity::Medium,
            }),
            tools: None,
            tool_choice: None,
            store: Some(false),
            previous_response_id: None,
            metadata: None,
        }
    }

    /// Create a request with custom reasoning effort
    pub fn with_reasoning(model: String, input: String, effort: ReasoningEffort) -> Self {
        Self {
            model,
            input,
            reasoning: Some(ReasoningConfig {
                effort,
                encrypted_content: None,
            }),
            text: Some(TextConfig {
                verbosity: Verbosity::Medium,
            }),
            tools: None,
            tool_choice: None,
            store: Some(false),
            previous_response_id: None,
            metadata: None,
        }
    }

    /// Create a request with custom verbosity
    pub fn with_verbosity(model: String, input: String, verbosity: Verbosity) -> Self {
        Self {
            model,
            input,
            reasoning: Some(ReasoningConfig {
                effort: ReasoningEffort::Medium,
                encrypted_content: None,
            }),
            text: Some(TextConfig {
                verbosity,
            }),
            tools: None,
            tool_choice: None,
            store: Some(false),
            previous_response_id: None,
            metadata: None,
        }
    }

    /// Create a request with tools
    pub fn with_tools(model: String, input: String, tools: Vec<Tool>) -> Self {
        Self {
            model,
            input,
            reasoning: Some(ReasoningConfig {
                effort: ReasoningEffort::Medium,
                encrypted_content: None,
            }),
            text: Some(TextConfig {
                verbosity: Verbosity::Medium,
            }),
            tools: Some(tools),
            tool_choice: Some(ToolChoice::Auto("auto".to_string())),
            store: Some(false),
            previous_response_id: None,
            metadata: None,
        }
    }
}