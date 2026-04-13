//! Codex API request/response models for OpenAI Codex backend
//!
//! The Codex API uses a slightly different format than the standard OpenAI API,
//! as it goes through the chatgpt.com backend.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Codex API request structure
#[derive(Serialize, Debug, Clone)]
pub struct CodexRequest {
    /// The model to use (e.g., "gpt-5.2-codex")
    pub model: String,

    /// Instructions/system prompt for the model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Input messages for the current conversation turn
    pub input: Vec<CodexMessage>,

    /// Stream the response (must always be true for Codex API)
    pub stream: bool,

    /// Temperature for sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Whether to store the conversation (must be false for Codex)
    pub store: bool,
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
    Reasoning { id: String, summary: Vec<String> },
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
    /// Create a request from conversation messages
    pub fn from_messages(model: String, messages: Vec<CodexMessage>) -> Self {
        Self {
            model,
            instructions: None,
            input: messages,
            stream: true,
            temperature: None,
            store: false,
        }
    }

    /// Set the system instructions
    pub fn with_instructions(mut self, instructions: String) -> Self {
        self.instructions = Some(instructions);
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

#[cfg(test)]
mod tests {
    use super::{CodexMessage, CodexRequest};

    #[test]
    fn test_request_serializes_input_as_message_array() {
        let request = CodexRequest::from_messages(
            "gpt-5.4".to_string(),
            vec![CodexMessage {
                role: "user".to_string(),
                content: "hey".to_string(),
            }],
        )
        .with_instructions("Be concise.".to_string());

        let json = serde_json::to_value(&request).unwrap();

        assert!(json["input"].is_array());
        assert_eq!(json["input"][0]["role"], "user");
        assert_eq!(json["input"][0]["content"], "hey");
    }
}
