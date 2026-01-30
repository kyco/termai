use crate::llm::common::model::role::Role;
use crate::llm::openai::{
    adapter::compact_adapter::CompactAdapter,
    model::compact_api::{CompactInputItem, CompactOutputItem, CompactRequest},
};
use crate::session::model::message::{Message, MessageType};
use crate::session::model::session::Session;
use anyhow::Result;

/// Character threshold for triggering compaction (400KB = 80% of 500KB limit)
const COMPACTION_CHAR_THRESHOLD: usize = 400_000;

/// Check if a session needs compaction based on total message size
pub fn needs_compaction(session: &Session) -> bool {
    let total_size: usize = session.messages.iter()
        .map(|m| m.content.len())
        .sum();
    total_size > COMPACTION_CHAR_THRESHOLD
}

/// Compact the session's conversation history using OpenAI's compact endpoint
///
/// This function will:
/// 1. Convert session messages to the compact API format
/// 2. Call the compact endpoint
/// 3. Replace session messages with compacted output
///
/// If compaction fails, the session is left unchanged (graceful degradation)
pub async fn compact_session(api_key: &str, session: &mut Session, model: &str) -> Result<()> {
    // Convert session messages to compact input format
    let input: Vec<CompactInputItem> = session.messages.iter().map(|m| {
        match &m.message_type {
            MessageType::Standard => {
                CompactInputItem::message(m.role.to_string(), m.content.clone())
            }
            MessageType::Compaction { compaction_id, encrypted_content } => {
                CompactInputItem::compaction(compaction_id.clone(), encrypted_content.clone())
            }
        }
    }).collect();

    // Create the compact request
    let request = CompactRequest::new(model.to_string(), input);

    // Call the compact endpoint
    let response = CompactAdapter::compact(&request, api_key).await?;

    // Convert the response back to session messages
    let mut new_messages = Vec::new();

    for output in response.output {
        match output {
            CompactOutputItem::Message { role, content } => {
                // Extract text from content items
                let text = content.iter()
                    .map(|c| c.text())
                    .collect::<Vec<_>>()
                    .join("");

                if !text.is_empty() {
                    new_messages.push(Message::new(
                        String::new(),
                        Role::from_str(&role),
                        text,
                    ));
                }
            }
            CompactOutputItem::Compaction { id, encrypted_content } => {
                // Create a compaction message
                // The content is a placeholder since the actual content is encrypted
                new_messages.push(Message::new_compaction(
                    String::new(),
                    Role::System, // Compaction items are treated as system context
                    "[Compacted conversation history]".to_string(),
                    id,
                    encrypted_content,
                ));
            }
        }
    }

    // Replace session messages with compacted output
    session.messages = new_messages;

    Ok(())
}

/// Attempt to compact session, logging errors but not failing the operation
pub async fn try_compact_session(api_key: &str, session: &mut Session, model: &str) {
    if let Err(e) = compact_session(api_key, session, model).await {
        eprintln!("Warning: Conversation compaction failed: {}. Continuing with original messages.", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::session::Session;

    #[test]
    fn test_needs_compaction_below_threshold() {
        let mut session = Session::new_temporary();
        session.messages.push(Message::new(
            "1".to_string(),
            Role::User,
            "Hello".to_string(),
        ));
        assert!(!needs_compaction(&session));
    }

    #[test]
    fn test_needs_compaction_above_threshold() {
        let mut session = Session::new_temporary();
        // Create a message larger than 400KB
        let large_content = "x".repeat(COMPACTION_CHAR_THRESHOLD + 1);
        session.messages.push(Message::new(
            "1".to_string(),
            Role::User,
            large_content,
        ));
        assert!(needs_compaction(&session));
    }

    #[test]
    fn test_needs_compaction_cumulative() {
        let mut session = Session::new_temporary();
        // Create multiple messages that together exceed threshold
        let content = "x".repeat(COMPACTION_CHAR_THRESHOLD / 2 + 1);
        session.messages.push(Message::new(
            "1".to_string(),
            Role::User,
            content.clone(),
        ));
        session.messages.push(Message::new(
            "2".to_string(),
            Role::Assistant,
            content,
        ));
        assert!(needs_compaction(&session));
    }
}
