use crate::llm::openai::model::chat_message::ChatMessage;
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use crate::llm::openai::model::verbosity::Verbosity;
use crate::llm::openai::model::custom_tools::{CustomTool, AllowedToolsChoice};
use serde::Serialize;

#[derive(Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub reasoning_effort: ReasoningEffort,
    
    // GPT-5 new features for Chat Completions API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<Verbosity>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ChatTool>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ChatToolChoice>,
}

#[derive(Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ChatTool {
    Custom { 
        #[serde(rename = "type")]
        tool_type: String,
        custom: CustomTool,
    },
    Function {
        #[serde(rename = "type")]
        tool_type: String,
        function: FunctionDefinition,
    },
}

#[derive(Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum ChatToolChoice {
    Auto(String), // "auto"
    None(String), // "none"
    Required(String), // "required"
    AllowedTools(AllowedToolsChoice),
}

#[allow(dead_code)]
impl ChatCompletionRequest {
    pub fn simple(model: String, messages: Vec<ChatMessage>) -> Self {
        Self {
            model,
            messages,
            reasoning_effort: ReasoningEffort::Medium,
            verbosity: None,
            tools: None,
            tool_choice: None,
        }
    }

    pub fn with_reasoning(model: String, messages: Vec<ChatMessage>, effort: ReasoningEffort) -> Self {
        Self {
            model,
            messages,
            reasoning_effort: effort,
            verbosity: None,
            tools: None,
            tool_choice: None,
        }
    }

    pub fn with_verbosity(model: String, messages: Vec<ChatMessage>, verbosity: Verbosity) -> Self {
        Self {
            model,
            messages,
            reasoning_effort: ReasoningEffort::Medium,
            verbosity: Some(verbosity),
            tools: None,
            tool_choice: None,
        }
    }
}
