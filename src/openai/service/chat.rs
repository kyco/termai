use crate::openai::{
    adapter::open_ai_adapter,
    model::{
        chat_completion_request::ChatCompletionRequest, chat_message::ChatMessage, model::Model,
        role::Role,
    },
};
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use anyhow::Result;

pub const SYSTEM_PROMPT: &str = "
You're an assistant in the terminal.
You will keep your answers brief as the user is chatting to you from the command line.
You will never output markdown, only ASCII text or ASCII art.
You will limit your line length to 80 characters.
You will not replace any UUIDs that you find in the text, these are required by the application for replacements later.";

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
                redaction_mapping: None,
            });
        }
    }

    Ok(())
}
