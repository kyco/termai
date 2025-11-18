use serde::{Serialize, Deserialize};
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use crate::llm::openai::model::verbosity::Verbosity;
use crate::llm::openai::model::custom_tools::{CustomTool, AllowedToolsChoice};

/// Request input can be a string or array of messages  
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestInput {
    Text(String),
    Messages(Vec<InputMessage>),
}

/// Input message structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputMessage {
    pub role: String,
    pub content: String,
}

/// Responses API request structure
#[derive(Serialize, Debug, Clone)]
pub struct ResponsesRequest {
    /// Model to use
    pub model: String,
    
    /// Input text or message array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<RequestInput>,
    
    /// System instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    
    /// Reasoning configuration (o-series models only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    
    /// Text generation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    
    /// Tools available to the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    
    /// Tool choice configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    
    /// Store conversation for future reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    
    /// Previous response ID for multi-turn conversations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    
    /// Metadata for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    
    /// Maximum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    
    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Top P
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Streaming
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    
    /// Verbosity level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ReasoningConfig {
    /// Reasoning effort level
    pub effort: ReasoningEffort,
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

/// Responses API response structure
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ResponsesResponse {
    /// Response ID for reference
    pub id: String,
    
    /// Object type
    pub object: String,
    
    /// Model used
    pub model: String,
    
    /// Response status
    pub status: String, // "completed", "failed", etc.
    
    /// Error information if failed
    pub error: Option<ResponseError>,
    
    /// Output array containing messages and tool calls
    pub output: Vec<ResponseOutput>,
    
    /// Usage statistics
    pub usage: Option<ResponseUsage>,
    
    /// Reasoning information for o-series models
    pub reasoning: Option<ResponseReasoning>,
    
    /// Previous response ID
    pub previous_response_id: Option<String>,
    
    /// Whether response was stored
    pub store: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ResponseError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum ResponseOutput {
    #[serde(rename = "message")]
    Message {
        id: String,
        status: String,
        role: String,
        content: Vec<ContentItem>,
    },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        status: String,
        call_type: String,
        function: Option<ToolCallFunction>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        summary: Vec<String>,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum ContentItem {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        annotations: Vec<serde_json::Value>,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct ResponseReasoning {
    pub effort: Option<String>,
    pub summary: Option<String>,
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
    /// Input tokens
    pub input_tokens: u32,
    
    /// Output tokens
    pub output_tokens: u32,
    
    /// Total tokens
    pub total_tokens: u32,
    
    /// Input token details
    pub input_tokens_details: Option<InputTokenDetails>,
    
    /// Output token details
    pub output_tokens_details: Option<OutputTokenDetails>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct InputTokenDetails {
    pub cached_tokens: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct OutputTokenDetails {
    pub reasoning_tokens: Option<u32>,
}

impl ResponsesRequest {
    /// Create a simple text request
    pub fn simple(model: String, input: String) -> Self {
        Self {
            model,
            input: Some(RequestInput::Text(input)),
            instructions: None,
            reasoning: Some(ReasoningConfig {
                effort: ReasoningEffort::High,
            }),
            text: Some(TextConfig {
                verbosity: Verbosity::Medium,
            }),
            tools: None,
            tool_choice: None,
            store: Some(true),
            previous_response_id: None,
            metadata: None,
            max_output_tokens: Some(32000), // Set reasonable output limit
            temperature: None,
            top_p: None,
            stream: Some(false),
            verbosity: None,
        }
    }

    /// Create a request from messages (for conversation)
    pub fn from_messages(model: String, messages: Vec<InputMessage>) -> Self {
        Self {
            model,
            input: Some(RequestInput::Messages(messages)),
            instructions: None,
            reasoning: Some(ReasoningConfig {
                effort: ReasoningEffort::High,
            }),
            text: Some(TextConfig {
                verbosity: Verbosity::Medium,
            }),
            tools: None,
            tool_choice: None,
            store: Some(true),
            previous_response_id: None,
            metadata: None,
            max_output_tokens: Some(32000), // Set reasonable output limit
            temperature: None,
            top_p: None,
            stream: Some(false),
            verbosity: None,
        }
    }

    /// Create a request with custom reasoning effort
    pub fn with_reasoning(model: String, input: String, effort: ReasoningEffort) -> Self {
        let mut request = Self::simple(model, input);
        request.reasoning = Some(ReasoningConfig {
            effort,
        });
        request
    }
}