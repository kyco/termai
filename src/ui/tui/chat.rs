use crate::args::Provider;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use crate::config::model::keys::ConfigKeys;
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service::session_add_messages;
use crate::llm::{claude, openai};
use crate::llm::common::model::role::Role;
use anyhow::Result;

pub async fn send_message<R: ConfigRepository, SR: SessionRepository, MR: MessageRepository>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
    session: &mut Session,
    message: String,
) -> Result<()> {
    // Add user message to session
    session.add_raw_message(message, Role::User);
    session.redact(repo);

    // Get provider configuration
    let provider = config_service::fetch_by_key(repo, &ConfigKeys::ProviderKey.to_key())?;
    let provider = Provider::new(&provider.value);
    let provider_api_key = match provider {
        Provider::Claude => config_service::fetch_by_key(repo, &ConfigKeys::ClaudeApiKey.to_key())?,
        Provider::Openapi => {
            config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?
        }
    };

    // Send to AI
    match provider {
        Provider::Claude => {
            claude::service::chat::chat(&provider_api_key.value, session).await?;
        }
        Provider::Openapi => {
            openai::service::chat::chat(&provider_api_key.value, session).await?;
        }
    }

    // Save messages to database
    session_add_messages(session_repository, message_repository, session)?;
    session.unredact();

    Ok(())
}

// New async function that doesn't add the user message (it's already added in the UI)
pub async fn send_message_async<R: ConfigRepository, SR: SessionRepository, MR: MessageRepository>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
    session: &mut Session,
    _message: String,
) -> Result<String> {
    // The user message is already added to the session in the UI
    // We just need to redact it and send to AI
    session.redact(repo);

    // Get provider configuration
    let provider = config_service::fetch_by_key(repo, &ConfigKeys::ProviderKey.to_key())?;
    let provider = Provider::new(&provider.value);
    let provider_api_key = match provider {
        Provider::Claude => config_service::fetch_by_key(repo, &ConfigKeys::ClaudeApiKey.to_key())?,
        Provider::Openapi => {
            config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?
        }
    };

    // Send to AI
    match provider {
        Provider::Claude => {
            claude::service::chat::chat(&provider_api_key.value, session).await?;
        }
        Provider::Openapi => {
            openai::service::chat::chat(&provider_api_key.value, session).await?;
        }
    }

    // Save messages to database
    session_add_messages(session_repository, message_repository, session)?;
    session.unredact();

    // Return the AI response (last message in the session)
    let ai_response = session.messages.last()
        .filter(|msg| msg.role == Role::Assistant)
        .map(|msg| msg.content.clone())
        .unwrap_or_else(|| "No response received".to_string());

    Ok(ai_response)
} 