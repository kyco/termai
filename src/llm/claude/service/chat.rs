use crate::llm::claude::adapter::claude_adapter;
use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_message::ChatMessage;
use crate::llm::common::model::role::Role;
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::Result;

pub async fn chat(api_key: &str, session: &mut Session) -> Result<()> {
    let model = "claude-3-7-sonnet-20250219".to_string();

    let chat_messages = session
        .messages
        .iter()
        .filter(|m| m.role != Role::System)
        .map(|m| ChatMessage {
            role: m.role.to_string(),
            content: m.content.to_string(),
        })
        .collect::<Vec<ChatMessage>>();

    let system_message = session
        .messages
        .iter()
        .filter(|m| m.role == Role::System)
        .map(|m| m.content.to_string())
        .collect::<Vec<String>>();
    let system_message = system_message.first().map(|m| m.to_string());

    let request = ChatCompletionRequest {
        model,
        max_tokens: 64000,
        messages: chat_messages,
        system: system_message,
    };

    let response = claude_adapter::chat(&request, api_key).await?;

    let mut response_text = String::new();
    for block in response.content {
        response_text.push_str(&block.text);
    }

    session.messages.push(Message {
        id: "".to_string(),
        role: Role::Assistant,
        content: response_text,
    });

    Ok(())
}
