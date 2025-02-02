mod args;
mod common;
mod config;
mod openai;
mod output;
mod path;
mod redactions;
mod repository;
mod session;

use crate::args::Args;
use crate::config::repository::ConfigRepository;
use crate::config::service::{open_ai_config, redacted_config};
use crate::openai::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
use crate::session::model::message::{contains_system_prompt, messages_with_system_prompt};
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::session::service::sessions_service::session_add_messages;
use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use openai::service::chat::chat;
use output::message::Message;
use output::outputter;
use repository::db::SqliteRepository;
use session::service::sessions_service::fetch_current_session;
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

    if args.is_redaction() {
        redacted_config::redaction(&repo, &args)?;
        return Ok(());
    }

    if args.is_sessions_all() {
        sessions_service::fetch_all_sessions(&repo, &repo)?;
        return Ok(());
    }

    if args.is_session_add() {
        let name = session_name(&args);
        sessions_service::session_add(&repo, &name)?;
    }

    if args.print_config {
        return print_config(&repo);
    }

    let mut current_session = fetch_current_session(&repo, &repo)?;
    let local_context = extract_content(&args.directory, &args.exclude);
    let input = extract_input_or_quit(&args);
    request_response_from_ai(
        &repo,
        &repo,
        &repo,
        &input,
        &mut current_session,
        args.system_prompt,
        &local_context,
    )
    .await
}

fn session_name(args: &Args) -> String {
    let name = if let Some(session_name) = &args.sessions_new {
        session_name
    } else {
        &common::unique_id::generate_uuid_v4().to_string()
    };
    name.to_string()
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
    let open_ai_api_key = config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?;

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

    if let Err(err) = chat(&open_ai_api_key.value, session).await {
        println!("{:#?}", err);
        return Err(err);
    }

    session.unredact();
    session_add_messages(session_repository, message_repository, session)
        .expect("could not write new messages to repo");

    let output_messages = session
        .messages
        .iter()
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
