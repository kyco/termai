//! Codex chat service using OAuth authentication
//!
//! This service provides chat functionality using the Codex API
//! with OAuth access tokens from ChatGPT Plus/Pro subscriptions.

use crate::llm::common::model::role::Role;
use crate::llm::openai::{
    adapter::codex_adapter::CodexAdapter,
    model::codex_api::{CodexContentItem, CodexMessage, CodexOutput, CodexRequest},
};
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::{anyhow, Result};

/// Default model for Codex API
const DEFAULT_CODEX_MODEL: &str = "gpt-5.4";

fn build_request(session: &Session, model_param: Option<&str>) -> Result<CodexRequest> {
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
    let instructions =
        instructions.unwrap_or_else(|| "You are a helpful AI assistant.".to_string());

    if chat_messages.is_empty() {
        return Err(anyhow!(
            "No user or assistant messages available for Codex request."
        ));
    }

    // Check total input size
    let total_input_size: usize =
        chat_messages.iter().map(|m| m.content.len()).sum::<usize>() + instructions.len();

    if total_input_size > 500_000 {
        return Err(anyhow!(
            "Input too large ({} characters). Please reduce input size to under 500,000 characters.",
            total_input_size
        ));
    }

    let request = CodexRequest::from_messages(model, chat_messages).with_instructions(instructions);

    Ok(request)
}

/// Chat using the Codex API with OAuth authentication
pub async fn chat(
    access_token: &str,
    session: &mut Session,
    model_param: Option<&str>,
) -> Result<()> {
    let request = build_request(session, model_param)?;

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
        session
            .messages
            .push(Message::new(String::new(), Role::Assistant, text));
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

#[cfg(test)]
mod tests {
    use super::build_request;
    use crate::llm::common::model::role::Role;
    use crate::session::model::message::Message;
    use crate::session::model::session::Session;

    #[test]
    fn test_build_request_serializes_single_turn_input_as_array() {
        let mut session = Session::new_temporary();
        session
            .messages
            .push(Message::new(String::new(), Role::User, "hey".to_string()));

        let request = build_request(&session, Some("gpt-5.4")).unwrap();
        let json = serde_json::to_value(&request).unwrap();

        assert!(json["input"].is_array());
        assert_eq!(json["input"][0]["role"], "user");
        assert_eq!(json["input"][0]["content"], "hey");
    }

    #[test]
    fn test_build_request_preserves_multi_turn_conversation_and_instructions() {
        let mut session = Session::new_temporary();
        session.messages.push(Message::new(
            String::new(),
            Role::System,
            "Be concise.".to_string(),
        ));
        session.messages.push(Message::new(
            String::new(),
            Role::User,
            "First question".to_string(),
        ));
        session.messages.push(Message::new(
            String::new(),
            Role::Assistant,
            "First answer".to_string(),
        ));
        session.messages.push(Message::new(
            String::new(),
            Role::User,
            "Follow-up".to_string(),
        ));

        let request = build_request(&session, Some("gpt-5.4")).unwrap();
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["instructions"], "Be concise.");
        assert!(json["input"].is_array());
        assert_eq!(json["input"].as_array().unwrap().len(), 3);
        assert_eq!(json["input"][0]["role"], "user");
        assert_eq!(json["input"][1]["role"], "assistant");
        assert_eq!(json["input"][2]["content"], "Follow-up");
    }
}
