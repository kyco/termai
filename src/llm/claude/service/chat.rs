use crate::llm::claude::adapter::claude_adapter;
use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_message::ChatMessage;
use crate::llm::claude::model::content_block::ContentBlock;
use crate::llm::claude::model::thinking::Thinking;
use crate::llm::claude::model::thinking_type::ThinkingType;
use crate::llm::common::model::role::Role;
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::Result;

pub async fn chat(api_key: &str, session: &mut Session) -> Result<()> {
    let model = "claude-opus-4-1-20250805".to_string();

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

    let max_tokens = 32000;
    let budget_tokens = 16000;
    let request = ChatCompletionRequest {
        model,
        max_tokens,
        messages: chat_messages,
        system: system_message,
        thinking: Some(Thinking {
            budget_tokens,
            thinking_type: ThinkingType::Enabled,
        }),
    };

    let (_, response) = claude_adapter::chat(&request, api_key).await?;

    // Handle refusal stop reason introduced in Claude 4
    if response.stop_reason == "refusal" {
        anyhow::bail!("Claude declined to generate content for safety reasons");
    }

    let mut response_text = String::new();
    for block in response.content {
        let content = match block {
            ContentBlock::Text { text } => text,
            _ => continue,
        };
        response_text.push_str(&content);
    }

    session.messages.push(Message {
        id: "".to_string(),
        role: Role::Assistant,
        content: response_text,
    });

    Ok(())
}
