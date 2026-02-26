//! Codex chat service using OAuth authentication
//!
//! This service provides chat functionality using the Codex API
//! with OAuth access tokens from ChatGPT Plus/Pro subscriptions.

use crate::llm::common::model::role::Role;
use crate::llm::openai::{
    adapter::codex_adapter::CodexAdapter,
    model::codex_api::{CodexMessage, CodexRequest, CodexContentItem, CodexOutput},
};
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::{anyhow, Result};

/// Default model for Codex API
const DEFAULT_CODEX_MODEL: &str = "gpt-5.3-codex";

/// Chat using the Codex API with OAuth authentication
pub async fn chat(access_token: &str, session: &mut Session, model_param: Option<&str>) -> Result<()> {
    let model = model_param.unwrap_or(DEFAULT_CODEX_MODEL).to_string();

    // Extract system messages for instructions, and non-system messages for input
    let mut instructions: Option<String> = None;
    let mut chat_messages: Vec<CodexMessage> = Vec::new();

    for m in &session.messages {
        if m.role == Role::System {
            // Combine system messages into instructions
            if let Some(ref mut inst) = instructions {
                inst.push_str("\n\n");
                inst.push_str(&m.content);
            } else {
                instructions = Some(m.content.clone());
            }
        } else {
            chat_messages.push(CodexMessage {
                role: m.role.to_string(),
                content: m.content.clone(),
            });
        }
    }

    // Provide default instructions if none found
    let instructions = instructions.unwrap_or_else(|| {
        "You are a helpful AI assistant.".to_string()
    });

    // Check total input size
    let total_input_size: usize = chat_messages.iter().map(|m| m.content.len()).sum::<usize>()
        + instructions.len();

    if total_input_size > 500_000 {
        return Err(anyhow!(
            "Input too large ({} characters). Please reduce input size to under 500,000 characters.",
            total_input_size
        ));
    }

    // Create the request with instructions
    let request = if chat_messages.len() == 1 && chat_messages[0].role == "user" {
        CodexRequest::simple(model, chat_messages[0].content.clone())
            .with_instructions(instructions)
    } else {
        CodexRequest::from_messages(model, chat_messages)
            .with_instructions(instructions)
    };

    // Make the request
    let response = CodexAdapter::chat(&request, access_token).await?;

    // Check if request was successful
    if !response.is_successful() {
        if let Some(error) = response.error {
            return Err(anyhow!("Codex API error: {}", error.message));
        } else {
            return Err(anyhow!("Request failed with status: {}", response.status));
        }
    }

    // Extract text from the response
    if let Some(text) = response.extract_text() {
        session.messages.push(Message::new(
            String::new(),
            Role::Assistant,
            text,
        ));
    } else {
        // Try to extract from output directly
        for output in response.output {
            if let CodexOutput::Message { role, content, .. } = output {
                let message_text = extract_text_from_content(content);
                if !message_text.is_empty() {
                    session.messages.push(Message::new(
                        String::new(),
                        Role::from_str(&role),
                        message_text,
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Extract text content from content items
fn extract_text_from_content(content: Vec<CodexContentItem>) -> String {
    content
        .into_iter()
        .map(|item| match item {
            CodexContentItem::OutputText { text, .. } => text,
        })
        .collect::<Vec<String>>()
        .join("\n")
}
