mod args;
mod chat;
mod common;
mod config;
mod context;
mod llm;
mod output;
mod path;
mod redactions;
mod repository;
mod session;
mod setup;
mod ui;

use crate::args::{Args, Commands, ConfigAction, Provider, RedactAction, SessionAction};
use crate::chat::InteractiveSession;
use crate::config::repository::ConfigRepository;
use crate::config::service::provider_config::write_provider_key;
use crate::config::service::{claude_config, open_ai_config, redacted_config};
use crate::llm::common::model::role::Role;
use crate::path::extract::{extract_content, extract_content_with_smart_fallback};
use crate::path::model::Files;
use crate::session::model::message::{contains_system_prompt, messages_with_system_prompt};
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::session::service::sessions_service::session_add_messages;
use crate::setup::SetupWizard;
use crate::ui::timer::ThinkingTimer;
use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use llm::claude;
use llm::openai;
use output::message::Message;
use output::outputter;
use repository::db::SqliteRepository;
use std::fs::create_dir_all;
use std::io::IsTerminal;
use std::io::{self, Read};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let db_path = db_path();
    let repo = SqliteRepository::new(db_path.to_str().unwrap())?;

    // Handle subcommands
    match &args.command {
        Some(Commands::Setup) => {
            let wizard = SetupWizard::new();
            wizard.run(&repo).await?;
            return Ok(());
        }
        Some(Commands::Config { action }) => {
            return handle_config_command(&repo, action, &args);
        }
        Some(Commands::Redact { action }) => {
            return handle_redact_command(&repo, action, &args);
        }
        Some(Commands::Sessions { action }) => {
            return handle_sessions_command(&repo, action);
        }
        Some(Commands::Chat { .. }) | None => {
            // Continue to chat handling below
        }
    }

    // Legacy flag handling for backwards compatibility
    if args.is_chat_gpt_api_key() {
        open_ai_config::write_open_ai_key(&repo, &args)?;
        return Ok(());
    }

    if args.is_claude_api_key() {
        claude_config::write_claude_key(&repo, &args)?;
        return Ok(());
    }

    if args.is_redaction() {
        redacted_config::redaction(&repo, &args)?;
        return Ok(());
    }

    if args.is_sessions_all() {
        sessions_service::fetch_all_sessions(&repo, &repo)?;
        return Ok(());
    }

    if args.is_provider() {
        write_provider_key(&repo, &args)?;
        return Ok(());
    }

    if args.is_config_show() {
        return print_config(&repo);
    }

    // Handle chat functionality
    let mut session = if args.is_session() {
        if let Some(name) = &args.get_chat_session() {
            sessions_service::session(&repo, &repo, name)?
        } else {
            Session::new_temporary()
        }
    } else {
        Session::new_temporary()
    };

    let local_context = if let Some(Commands::Chat {
        smart_context,
        context_query,
        preview_context,
        chunked_analysis,
        chunk_strategy,
        max_context_tokens,
        ..
    }) = &args.command
    {
        if *smart_context {
            extract_content_with_smart_fallback(
                &args.get_chat_directory(),
                &args.get_chat_directories(),
                &args.get_chat_exclude(),
                true,
                context_query.as_deref(),
                *preview_context,
                *chunked_analysis,
                Some(chunk_strategy.as_str()),
                *max_context_tokens,
            )
            .await
        } else {
            extract_content(
                &args.get_chat_directory(),
                &args.get_chat_directories(),
                &args.get_chat_exclude(),
            )
        }
    } else {
        extract_content(
            &args.get_chat_directory(),
            &args.get_chat_directories(),
            &args.get_chat_exclude(),
        )
    };

    // Check if we have direct input for one-shot mode or should start interactive mode
    let input_data = get_input_from_args_or_stdin(&args);

    if let Some(input) = input_data {
        // One-shot mode: process single command and exit
        request_response_from_ai(
            &repo,
            &repo,
            &repo,
            &input,
            &mut session,
            args.get_chat_system_prompt(),
            &local_context,
        )
        .await
    } else {
        // Interactive mode: start chat session
        let context_files = local_context.unwrap_or_default();
        let mut interactive_session =
            InteractiveSession::new(&repo, &repo, &repo, session, context_files)?;

        interactive_session.run().await
    }
}

fn handle_config_command<R: ConfigRepository>(
    repo: &R,
    action: &ConfigAction,
    _args: &Args,
) -> Result<()> {
    match action {
        ConfigAction::Show => print_config(repo),
        ConfigAction::SetOpenai { api_key } => {
            config_service::write_config(repo, &ConfigKeys::ChatGptApiKey.to_key(), api_key)?;
            println!("OpenAI API key updated successfully");
            Ok(())
        }
        ConfigAction::SetClaude { api_key } => {
            config_service::write_config(repo, &ConfigKeys::ClaudeApiKey.to_key(), api_key)?;
            println!("Claude API key updated successfully");
            Ok(())
        }
        ConfigAction::SetProvider { provider } => {
            config_service::write_config(
                repo,
                &ConfigKeys::ProviderKey.to_key(),
                provider.to_str(),
            )?;
            println!("Default provider set to {}", provider.to_str());
            Ok(())
        }
        ConfigAction::Reset => {
            let wizard = SetupWizard::new();
            wizard.reset_configuration(repo)?;
            Ok(())
        }
    }
}

fn handle_redact_command<R: ConfigRepository>(
    repo: &R,
    action: &RedactAction,
    _args: &Args,
) -> Result<()> {
    // Create a temporary Args struct with the appropriate field set for the redaction function
    let mut temp_args = Args {
        command: None,
        chat_gpt_api_key: None,
        claude_api_key: None,
        system_prompt: None,
        redact_add: None,
        redact_remove: None,
        redact_list: false,
        print_config: false,
        sessions_all: false,
        session: None,
        data: None,
        directory: None,
        exclude: vec![],
        provider: None,
        directories: vec![],
        smart_context: false,
        max_context_tokens: None,
        preview_context: false,
        chunked_analysis: false,
    };

    match action {
        RedactAction::Add { pattern } => {
            temp_args.redact_add = Some(pattern.clone());
            redacted_config::redaction(repo, &temp_args)?;
            println!("Added redaction pattern: {pattern}");
            Ok(())
        }
        RedactAction::Remove { pattern } => {
            temp_args.redact_remove = Some(pattern.clone());
            redacted_config::redaction(repo, &temp_args)?;
            println!("Removed redaction pattern: {pattern}");
            Ok(())
        }
        RedactAction::List => {
            temp_args.redact_list = true;
            redacted_config::redaction(repo, &temp_args)?;
            Ok(())
        }
    }
}

fn handle_sessions_command<R: ConfigRepository + SessionRepository + MessageRepository>(
    repo: &R,
    action: &SessionAction,
) -> Result<()> {
    match action {
        SessionAction::List => {
            sessions_service::fetch_all_sessions(repo, repo)?;
            Ok(())
        }
    }
}

fn db_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let default_dir = home_dir.join(".config/termai");
    create_dir_all(&default_dir).expect("Failed to create default directory");
    default_dir.join("app.db")
}

fn print_config<R: ConfigRepository>(repo: &R) -> Result<()> {
    match config_service::fetch_config(repo) {
        Ok(configs) => {
            configs
                .iter()
                .for_each(|config| println!("{:} -> {:}", config.key, config.value));
            Ok(())
        }
        Err(_) => {
            println!("failed to fetch config");
            Ok(())
        }
    }
}

async fn request_response_from_ai<
    R: ConfigRepository,
    SR: SessionRepository,
    MR: MessageRepository,
>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
    input: &String,
    session: &mut Session,
    user_defined_system_prompt: Option<String>,
    local_context: &Option<Vec<Files>>,
) -> Result<()> {
    let provider = config_service::fetch_by_key(repo, &ConfigKeys::ProviderKey.to_key())?;
    let provider = Provider::new(&provider.value);
    let provider_api_key = match provider {
        Provider::Claude => config_service::fetch_by_key(repo, &ConfigKeys::ClaudeApiKey.to_key())?,
        Provider::Openapi => {
            config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?
        }
    };

    let contains_system_prompt = contains_system_prompt(&session.messages);
    if !contains_system_prompt {
        session.messages =
            messages_with_system_prompt(user_defined_system_prompt, &session.messages);
    }

    let input_with_local_context = match local_context {
        Some(files) => {
            let local_context: Vec<String> = files
                .iter()
                .map(|file| {
                    let file_path = file.path.clone();
                    let file_content = file.content.clone();
                    format!("{file_path}\n```\n{file_content}```")
                })
                .collect();
            format!("{}\n{}", input, local_context.join("\n"))
        }
        None => input.clone(),
    };

    session.add_raw_message(input_with_local_context, Role::User);
    session.redact(repo);

    let mut timer = ThinkingTimer::new();
    timer.start();

    match provider {
        Provider::Claude => {
            if let Err(err) = claude::service::chat::chat(&provider_api_key.value, session).await {
                println!("{err:#?}");
                timer.stop();
                return Err(err);
            }
        }
        Provider::Openapi => {
            if let Err(err) = openai::service::chat::chat(&provider_api_key.value, session).await {
                println!("{err:#?}");
                timer.stop();
                return Err(err);
            }
        }
    };
    timer.stop();

    session_add_messages(session_repository, message_repository, session)
        .expect("could not write new messages to repo");

    session.unredact();
    let output_messages = session
        .messages
        .iter()
        .filter(|message| message.role != Role::System)
        .map(|message| Message {
            role: message.role.clone(),
            message: message.content.to_string(),
        })
        .collect::<Vec<Message>>();

    outputter::print(output_messages);
    Ok(())
}

fn get_input_from_args_or_stdin(args: &Args) -> Option<String> {
    let mut input = String::new();

    // Check for command line input
    if let Some(ref data_arg) = args.get_chat_data() {
        input.push_str(data_arg);
    }

    // Check for piped input
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() {
            if !input.is_empty() {
                input.push('\n');
                input.push('\n');
            }
            input.push_str(buffer.trim());
        }
    }

    // Return input if we have any, otherwise None for interactive mode
    if input.is_empty() {
        None
    } else {
        Some(input)
    }
}
