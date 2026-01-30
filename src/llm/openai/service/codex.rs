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
const DEFAULT_CODEX_MODEL: &str = "gpt-4o";

/// Chat using the Codex API with OAuth authentication
pub async fn chat(access_token: &str, session: &mut Session, model: Option<&str>) -> Result<()> {
    let model = model.unwrap_or(DEFAULT_CODEX_MODEL).to_string();

    // Convert session messages to Codex format
    let messages: Vec<CodexMessage> = session
        .messages
        .iter()
        .map(|m| CodexMessage {
            role: m.role.to_string(),
            content: m.content.clone(),
        })
        .collect();

    // Check total input size
    let total_input_size: usize = messages.iter().map(|m| m.content.len()).sum();

    if total_input_size > 500_000 {
        return Err(anyhow!(
            "Input too large ({} characters). Please reduce input size to under 500,000 characters.",
            total_input_size
        ));
    }

    // Create the request
    let request = if messages.len() == 1 && messages[0].role == "user" {
        CodexRequest::simple(model, messages[0].content.clone())
    } else {
        CodexRequest::from_messages(model, messages)
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
        .filter_map(|item| match item {
            CodexContentItem::OutputText { text, .. } => Some(text),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
