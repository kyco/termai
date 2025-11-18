use crate::llm::common::model::role::Role;
use crate::llm::openai::{
    adapter::responses_adapter::ResponsesAdapter,
    model::{
        responses_api::{ResponsesRequest, InputMessage, ResponseOutput, ContentItem},
        model::Model,
    },
};
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::{Result, anyhow};

pub async fn chat(api_key: &str, session: &mut Session) -> Result<()> {
    let model = Model::Gpt5; // Default to GPT-5.1 for best performance

    // Convert session messages to input messages
    let input_messages = session
        .messages
        .iter()
        .map(|m| InputMessage {
            role: m.role.to_string(),
            content: m.content.to_string(),
        })
        .collect::<Vec<InputMessage>>();

    // Check total input size to prevent hanging on extremely large inputs
    let total_input_size: usize = input_messages
        .iter()
        .map(|m| m.content.len())
        .sum();
    
    if total_input_size > 500_000 { // 500KB limit
        return Err(anyhow!(
            "Input too large ({} characters). Please reduce input size to under 500,000 characters.",
            total_input_size
        ));
    }

    // Create the request
    let request = if input_messages.len() == 1 && input_messages[0].role == "user" {
        // For single user message, use simple text input
        ResponsesRequest::simple(model.to_string(), input_messages[0].content.clone())
    } else {
        // For conversation, use messages format
        ResponsesRequest::from_messages(model.to_string(), input_messages)
    };

    // Make the request
    let response = ResponsesAdapter::chat(&request, api_key).await?;

    // Check if request was successful
    if response.status != "completed" {
        if let Some(error) = response.error {
            return Err(anyhow!("OpenAI API error: {}", error.message));
        } else {
            return Err(anyhow!("Request failed with status: {}", response.status));
        }
    }

    // Process the output
    for output in response.output {
        match output {
            ResponseOutput::Message { role, content, .. } => {
                // Extract text from content items
                let message_text = extract_text_from_content(content);
                if !message_text.is_empty() {
                    session.messages.push(Message {
                        id: "".to_string(),
                        role: Role::from_str(&role),
                        content: message_text,
                    });
                }
            }
            ResponseOutput::ToolCall { .. } => {
                // Handle tool calls if needed in the future
                // For now, we'll skip them as they're not used in basic chat
            }
            ResponseOutput::Reasoning { .. } => {
                // Skip reasoning output - it's metadata, not user-facing content
            }
        }
    }

    Ok(())
}

/// Extract text content from content items
fn extract_text_from_content(content: Vec<ContentItem>) -> String {
    content
        .into_iter()
        .filter_map(|item| match item {
            ContentItem::OutputText { text, .. } => Some(text),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
