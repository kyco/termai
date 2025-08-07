use crate::llm::common::model::role::Role;
use crate::llm::openai::model::reasoning_effort::ReasoningEffort;
use crate::llm::openai::{
    adapter::open_ai_adapter,
    model::{
        chat_completion_request::ChatCompletionRequest, chat_message::ChatMessage, model::Model,
    },
};
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::Result;

pub async fn chat(api_key: &str, session: &mut Session) -> Result<()> {
    let model = Model::O3Mini;

    let chat_messages = session
        .messages
        .iter()
        .map(|m| ChatMessage {
            role: m.role.to_string(),
            content: m.content.to_string(),
        })
        .collect::<Vec<ChatMessage>>();

    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: chat_messages,
        reasoning_effort: ReasoningEffort::High,
        verbosity: None,
        tools: None,
        tool_choice: None,
    };
    let response = open_ai_adapter::chat(&request, api_key).await?;

    if let Some(choices) = response.choices {
        for choice in choices {
            let role = choice.message.role;
            let message = choice.message.content;
            session.messages.push(Message {
                id: "".to_string(),
                role: Role::from_str(&role),
                content: message,
            });
        }
    }

    Ok(())
}
