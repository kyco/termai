//! Codex API request/response models for OpenAI Codex backend
//!
//! The Codex API uses a slightly different format than the standard OpenAI API,
//! as it goes through the chatgpt.com backend.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Codex API request structure
#[derive(Serialize, Debug, Clone)]
pub struct CodexRequest {
    /// The model to use (e.g., "gpt-4", "o1-pro")
    pub model: String,

    /// Instructions/system prompt for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Input can be text or conversation messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<CodexInput>,

    /// Stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// Codex input - can be text or messages
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CodexInput {
    Text(String),
    Messages(Vec<CodexMessage>),
}

/// A message in the conversation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodexMessage {
    pub role: String,
    pub content: String,
}

/// Codex API response structure
#[derive(Deserialize, Debug, Clone)]
pub struct CodexResponse {
    /// Response ID
    pub id: String,

    /// Object type
    pub object: String,

    /// Model used
    pub model: String,

    /// Response status: "completed", "failed", etc.
    pub status: String,

    /// Error information if failed
    pub error: Option<CodexError>,

    /// Output array
    pub output: Vec<CodexOutput>,

    /// Usage statistics
    pub usage: Option<CodexUsage>,
}

/// Error information
#[derive(Deserialize, Debug, Clone)]
pub struct CodexError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

/// Output item in the response
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CodexOutput {
    #[serde(rename = "message")]
    Message {
        id: String,
        status: String,
        role: String,
        content: Vec<CodexContentItem>,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        summary: Vec<String>,
    },
}

/// Content item within a message
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum CodexContentItem {
    #[serde(rename = "output_text")]
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<serde_json::Value>,
    },
}

/// Usage statistics
#[derive(Deserialize, Debug, Clone)]
pub struct CodexUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

impl CodexRequest {
    /// Create a simple text request
    pub fn simple(model: String, input: String) -> Self {
        Self {
            model,
            instructions: None,
            input: Some(CodexInput::Text(input)),
            stream: Some(false),
            max_output_tokens: Some(16000),
            temperature: None,
        }
    }

    /// Create a request from conversation messages
    pub fn from_messages(model: String, messages: Vec<CodexMessage>) -> Self {
        Self {
            model,
            instructions: None,
            input: Some(CodexInput::Messages(messages)),
            stream: Some(false),
            max_output_tokens: Some(16000),
            temperature: None,
        }
    }

    /// Set the system instructions
    pub fn with_instructions(mut self, instructions: String) -> Self {
        self.instructions = Some(instructions);
        self
    }

    /// Set the maximum output tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_tokens);
        self
    }
}

impl CodexResponse {
    /// Check if the response completed successfully
    pub fn is_successful(&self) -> bool {
        self.status == "completed" && self.error.is_none()
    }

    /// Extract the text content from the response
    pub fn extract_text(&self) -> Option<String> {
        for output in &self.output {
            if let CodexOutput::Message { content, .. } = output {
                let texts: Vec<String> = content
                    .iter()
                    .filter_map(|item| match item {
                        CodexContentItem::OutputText { text, .. } => Some(text.clone()),
                    })
                    .collect();
                if !texts.is_empty() {
                    return Some(texts.join("\n"));
                }
            }
        }
        None
    }
}
