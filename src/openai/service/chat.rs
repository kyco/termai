use crate::openai::{
    adapter::open_ai_adapter,
    model::{
        chat_completion_request::ChatCompletionRequest, message::Message, model::Model, role::Role,
    },
};
use anyhow::Result;

const SYSTEM_PROMPT: &str = "
You're an assistant in the terminal.
You will keep your answers brief as the user is chatting to you from the command line.
You will never output markdown, only ASCII text.
The user also loves seeing ASCII art where appropriate
 (only use it to visually explain a concept or when the user requests something that can only be represented in ASCII).
You will limit your line length to 80 characters.";

pub async fn chat(api_key: &str, data: &str) -> Result<Vec<Message>> {
    let model = Model::O1Mini;
    let user_message = Message {
        role: Role::User.to_string(),
        content: data.to_string(),
    };
    let request = ChatCompletionRequest {
        model: model.to_string(),
        messages: create_message_content(&model, &user_message),
    };
    let response = open_ai_adapter::chat(&request, api_key).await?;

    let mut messages: Vec<Message> = vec![user_message.clone().remove_from_content(SYSTEM_PROMPT)];
    if let Some(choices) = response.choices {
        for choice in choices {
            let role = choice.message.role;
            let message = choice.message.content;
            messages.push(Message::new(&role, &message));
        }
    }

    Ok(messages)
}

fn create_message_content(model: &Model, user_message: &Message) -> Vec<Message> {
    match model {
        Model::O1Mini | Model::O1Preview => vec![user_message.prepend_content(SYSTEM_PROMPT)],
        _ => vec![
            Message {
                role: Role::System.to_string(),
                content: SYSTEM_PROMPT.to_string(),
            },
            user_message.clone(),
        ],
    }
}
