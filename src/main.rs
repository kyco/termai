mod args;
mod common;
mod config;
mod llm;
mod output;
mod path;
mod redactions;
mod repository;
mod session;
mod ui;

use crate::args::{Args, Provider};
use crate::config::repository::ConfigRepository;
use crate::config::service::provider_config::write_provider_key;
use crate::config::service::{claude_config, open_ai_config, redacted_config};
use crate::llm::common::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
use crate::session::model::message::{contains_system_prompt, messages_with_system_prompt};
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::session::service::sessions_service::session_add_messages;
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

    if args.print_config {
        return print_config(&repo);
    }

    if args.is_print_session() {
        return print_session(&repo, &repo, &args);
    }

    if args.is_ui() {
        return ui::tui::runner::run_tui(&repo, &repo, &repo).await;
    }

    // Check if we should use CLI mode (when input is provided)
    let has_input = args.data.is_some() || !io::stdin().is_terminal();
    
    if !has_input {
        // No input provided, start UI by default
        return ui::tui::runner::run_tui(&repo, &repo, &repo).await;
    }

    let mut session = if args.is_session() {
        if let Some(name) = &args.session {
            sessions_service::session(&repo, &repo, name)?
        } else {
            Session::new_temporary()
        }
    } else {
        Session::new_temporary()
    };

    let local_context = extract_content(&args.directory, &args.directories, &args.exclude);
    let input = extract_input_or_quit(&args);
    request_response_from_ai(
        &repo,
        &repo,
        &repo,
        &input,
        &mut session,
        args.system_prompt,
        &local_context,
    )
    .await
}

fn db_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let default_dir = home_dir.join(".config/termai");
    create_dir_all(&default_dir).expect("Failed to create default directory");
    let db_path = default_dir.join("app.db");
    db_path
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

fn print_session<SR: SessionRepository, MR: MessageRepository>(
    session_repository: &SR,
    message_repository: &MR,
    args: &Args,
) -> Result<()> {
    if let Some(session_name) = &args.print_session {
        match sessions_service::session(session_repository, message_repository, session_name) {
            Ok(session) => {
                let output_messages = session
                    .messages
                    .iter()
                    .filter(|message| message.role != Role::System)
                    .map(|message| message.to_output_message())
                    .collect::<Vec<Message>>();
                
                outputter::print(output_messages);
                Ok(())
            }
            Err(_) => {
                println!("Session '{}' not found", session_name);
                Ok(())
            }
        }
    } else {
        println!("No session name provided");
        Ok(())
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
                    format!("{}\n```\n{}```", file_path, file_content)
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
                println!("{:#?}", err);
                timer.stop();
                return Err(err);
            }
        }
        Provider::Openapi => {
            if let Err(err) = openai::service::chat::chat(&provider_api_key.value, session).await {
                println!("{:#?}", err);
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
        .map(|message| message.to_output_message())
        .collect::<Vec<Message>>();

    outputter::print(output_messages);
    Ok(())
}

fn extract_input_or_quit(args: &Args) -> String {
    let mut input = String::new();
    if let Some(ref data_arg) = args.data {
        input.push_str(data_arg);
    }
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        if !input.is_empty() {
            input.push('\n');
            input.push('\n');
        }
        input.push_str(buffer.trim());
    }
    if input.is_empty() {
        eprintln!("No input provided. Use positional arguments or pipe data.");
        std::process::exit(1);
    }
    input
}
