use crate::args::Provider;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use crate::config::model::keys::ConfigKeys;
use crate::session::model::session::Session;
use crate::session::model::message::Message;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service::session_add_messages;
use crate::llm::{claude, openai};
use crate::llm::common::model::role::Role;
use anyhow::Result;
use chrono::Utc;


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

// Background title generation for new sessions
pub async fn generate_session_title_async<R: ConfigRepository>(
    repo: &R,
    session: &Session,
) -> Result<String> {
    // Only generate titles for sessions with at least 2 messages (user + assistant)
    if session.messages.len() < 2 {
        return Err(anyhow::anyhow!("Not enough messages for title generation"));
    }

    // Create a temporary session with just the first exchange for title generation
    let mut title_session = Session {
        id: session.id.clone(),
        name: "title_generation".to_string(),
        expires_at: session.expires_at,
        current: false,
        messages: vec![
            Message {
                id: "system".to_string(),
                role: Role::System,
                content: "You are a helpful assistant that generates concise, descriptive titles for conversations. Create a title that captures the main topic or question from the user's message and assistant's response. The title should be 2-6 words, clear, and specific. Do not include quotes or extra formatting. Just respond with the title text only.".to_string(),
            },
            Message {
                id: "user".to_string(),
                role: Role::User,
                content: format!("Please generate a short, descriptive title for this conversation:\n\nUser: {}\nAssistant: {}", 
                    session.messages.get(0).map(|m| &m.content).map_or("", |v| v),
                    session.messages.get(1).map(|m| &m.content).map_or("", |v| v)),
            }
        ],
        temporary: true,
        redaction_mapping: None,
    };

    // Get provider configuration
    let provider = config_service::fetch_by_key(repo, &ConfigKeys::ProviderKey.to_key())?;
    let provider = Provider::new(&provider.value);
    let provider_api_key = match provider {
        Provider::Claude => config_service::fetch_by_key(repo, &ConfigKeys::ClaudeApiKey.to_key())?,
        Provider::Openapi => config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?,
    };

    // Send to AI for title generation
    match provider {
        Provider::Claude => {
            claude::service::chat::chat(&provider_api_key.value, &mut title_session).await?;
        }
        Provider::Openapi => {
            openai::service::chat::chat(&provider_api_key.value, &mut title_session).await?;
        }
    }

    // Extract the generated title from the AI response
    let title = title_session.messages.last()
        .filter(|msg| msg.role == Role::Assistant)
        .map(|msg| msg.content.trim().to_string())
        .unwrap_or_else(|| "New Chat".to_string());

    // Clean up the title (remove quotes, limit length, etc.)
    let clean_title = title
        .trim_matches('"')
        .trim_matches('\'')
        .chars()
        .take(50) // Limit to 50 characters
        .collect::<String>()
        .trim()
        .to_string();

    Ok(if clean_title.is_empty() { "New Chat".to_string() } else { clean_title })
}

// Function to convert temporary session to permanent with generated title
pub async fn convert_temporary_session_to_permanent<R: ConfigRepository, SR: SessionRepository, MR: MessageRepository>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
    session: &mut Session,
) -> Result<()> {
    if !session.temporary || session.messages.len() < 2 {
        return Ok(()); // Not a temporary session or not enough messages
    }

    // Generate title
    let title = match generate_session_title_async(repo, session).await {
        Ok(title) => title,
        Err(_) => "New Chat".to_string(), // Fallback title if generation fails
    };

    // Convert to permanent session
    session.temporary = false;
    session.name = title.clone();

    // Save to database using the existing session_add_messages logic
    // First, we need to create the session in the database
    let now = Utc::now().naive_utc();
    let expires_at = now + chrono::Duration::hours(24);
    
    match session_repository.add_session(&session.id, &title, expires_at, session.current) {
        Ok(_) => {},
        Err(_) => return Err(anyhow::anyhow!("Failed to create session in database")),
    }
    
    // Now save all messages using the existing logic
    match session_add_messages(session_repository, message_repository, session) {
        Ok(_) => {},
        Err(_) => return Err(anyhow::anyhow!("Failed to save messages to database")),
    }

    Ok(())
} 