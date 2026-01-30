/// Handler for the Ask command - one-shot questions with optional context
/// Integrates with existing LLM infrastructure for direct question answering
use crate::args::AskArgs;
use crate::config::model::keys::ConfigKeys;
use crate::config::service::config_service;
use crate::llm::common::constants::SYSTEM_PROMPT;
use crate::llm::common::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
use crate::repository::db::SqliteRepository;
use crate::session::model::message::Message;
use crate::session::model::session::Session;
use crate::session::service::sessions_service;
use anyhow::{Context, Result};
use colored::*;
use uuid::Uuid;

/// Handle the ask command for one-shot questions
pub async fn handle_ask_command(args: &AskArgs, repo: &SqliteRepository) -> Result<()> {
    // Apply environment variable fallbacks
    let args = args.clone().with_env_fallbacks();

    let question = &args.question;
    let directory = &args.directory;
    let directories = &args.directories;
    let exclude = &args.exclude;
    let session_name = &args.session;
    let system_prompt = &args.system_prompt;
    let smart_context = args.smart_context;

    println!("{}", "🤖 Ask Command".bright_cyan().bold());
    println!("{}", "═════════════".white().dimmed());
    println!();
    println!(
        "{}  {}",
        "Question:".bright_green().bold(),
        question.bright_white()
    );

    // Check if we have API configuration (with environment fallback)
    let provider = config_service::fetch_with_env_fallback(repo, &ConfigKeys::ProviderKey.to_key())
        .context("No provider configured. Run 'termai setup' to configure API keys or set TERMAI_PROVIDER environment variable.")?;

    // Extract context files if directory is provided
    let mut context_files = Vec::<Files>::new();
    let mut context_included = false;

    if directory.is_some() || !directories.is_empty() {
        println!();
        println!("{}", "🔍 Extracting context...".bright_yellow());

        context_files = extract_content(directory, directories, exclude)
            .context("Failed to extract context from specified directories")?;

        if !context_files.is_empty() {
            context_included = true;
            println!("   📁 {} file(s) included as context", context_files.len());
            for file in &context_files {
                println!("   • {}", file.path.bright_blue());
            }
        }
    }

    // Handle smart context discovery
    if smart_context {
        println!();
        println!(
            "{}",
            "⚠️  Smart context discovery is not yet fully implemented".yellow()
        );
        println!("💡 Use directory flags (-d) to include specific files as context");
    }

    // Prepare the system prompt
    let effective_system_prompt = system_prompt
        .clone()
        .unwrap_or_else(|| SYSTEM_PROMPT.to_string());

    // Create messages for the request
    let mut messages = vec![Message::new(
        Uuid::new_v4().to_string(),
        Role::System,
        effective_system_prompt,
    )];

    // Add context if available
    if context_included && !context_files.is_empty() {
        let mut context_content = String::new();
        context_content.push_str("Here is the relevant code context:\n\n");

        for file in &context_files {
            context_content.push_str(&format!("# File: {}\n", file.path));
            context_content.push_str("```\n");
            context_content.push_str(&file.content);
            context_content.push_str("\n```\n\n");
        }

        context_content
            .push_str("Please answer the following question with reference to this context.\n\n");

        messages.push(Message::new(
            Uuid::new_v4().to_string(),
            Role::User,
            context_content,
        ));
    }

    // Add the main question
    messages.push(Message::new(
        Uuid::new_v4().to_string(),
        Role::User,
        question.to_string(),
    ));

    println!();
    println!("{}", "💭 Processing your question...".bright_yellow());

    // Get or create session if specified
    let _session = if let Some(name) = session_name {
        println!("   📋 Saving to session: {}", name.bright_cyan());
        sessions_service::session(repo, repo, name)?
    } else {
        Session::new_temporary()
    };

    // Route to appropriate LLM based on provider
    let response = match provider.value.as_str() {
        "claude" => call_claude_api(repo, messages)
            .await
            .context("Failed to get response from Claude API")?,
        "openai" => call_openai_api(repo, messages)
            .await
            .context("Failed to get response from OpenAI API")?,
        "openai-codex" | "openai_codex" | "codex" => call_codex_api(repo, messages)
            .await
            .context("Failed to get response from Codex API")?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown provider: {}. Run 'termai config show' to check configuration.",
                provider.value
            ));
        }
    };

    // Display the response
    println!();
    println!("{}", "🤖 Response:".bright_green().bold());
    println!("{}", "═══════════".white().dimmed());
    println!();
    println!("{}", response.bright_white());

    // Save to session if specified
    if let Some(name) = session_name {
        println!();
        println!(
            "{}",
            format!("💾 Conversation saved to session: {}", name).green()
        );
        println!(
            "   {}",
            format!("Continue with: termai chat --session {}", name).cyan()
        );
    }

    println!();
    println!("{}", "💡 Quick actions:".bright_yellow().bold());
    println!(
        "   {}                      # Start interactive chat",
        "termai chat".cyan()
    );
    println!(
        "   {}         # View configuration",
        "termai config show".cyan()
    );
    if session_name.is_none() {
        println!(
            "   {}  # Save future questions to session",
            "termai ask --session <name> \"question\"".cyan()
        );
    }

    Ok(())
}

/// Call Claude API with the given messages
async fn call_claude_api(repo: &SqliteRepository, messages: Vec<Message>) -> Result<String> {
    let api_key = config_service::fetch_with_env_fallback(repo, &ConfigKeys::ClaudeApiKey.to_key())
        .context("Claude API key not configured. Run 'termai setup' to add your API key or set CLAUDE_API_KEY environment variable.")?;

    // Create a temporary session to use the existing chat service
    let mut session = Session::new_temporary();
    session.messages = messages;

    // Use the existing Claude chat service
    crate::llm::claude::service::chat::chat(&api_key.value, &mut session).await?;

    // Extract the assistant's response from the updated session
    if let Some(last_message) = session.messages.last() {
        if last_message.role == Role::Assistant {
            return Ok(last_message.content.clone());
        }
    }

    Err(anyhow::anyhow!("No response received from Claude API"))
}

/// Call OpenAI API with the given messages
async fn call_openai_api(repo: &SqliteRepository, messages: Vec<Message>) -> Result<String> {
    let api_key = config_service::fetch_with_env_fallback(repo, &ConfigKeys::ChatGptApiKey.to_key())
        .context("OpenAI API key not configured. Run 'termai setup' to add your API key or set OPENAI_API_KEY environment variable.")?;

    // Create a temporary session to use the existing chat service
    let mut session = Session::new_temporary();
    session.messages = messages;

    // Use the existing OpenAI chat service
    crate::llm::openai::service::chat::chat(&api_key.value, &mut session).await?;

    // Extract the assistant's response from the updated session
    if let Some(last_message) = session.messages.last() {
        if last_message.role == Role::Assistant {
            return Ok(last_message.content.clone());
        }
    }

    Err(anyhow::anyhow!("No response received from OpenAI API"))
}

/// Call Codex API with the given messages (OAuth authentication)
async fn call_codex_api(repo: &SqliteRepository, messages: Vec<Message>) -> Result<String> {
    use crate::auth::token_manager::TokenManager;

    // Get valid access token (auto-refreshes if needed)
    let token_manager = TokenManager::new(repo);
    let access_token = token_manager
        .get_valid_token()
        .await
        .context("Failed to get Codex access token")?
        .ok_or_else(|| anyhow::anyhow!(
            "Not authenticated with Codex. Run 'termai config login-codex' to authenticate with your ChatGPT Plus/Pro subscription."
        ))?;

    // Create a temporary session to use the Codex chat service
    let mut session = Session::new_temporary();
    session.messages = messages;

    // Use the Codex chat service
    crate::llm::openai::service::codex::chat(&access_token, &mut session).await?;

    // Extract the assistant's response from the updated session
    if let Some(last_message) = session.messages.last() {
        if last_message.role == Role::Assistant {
            return Ok(last_message.content.clone());
        }
    }

    Err(anyhow::anyhow!("No response received from Codex API"))
}
