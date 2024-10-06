mod args;
mod config;
mod openai;
mod output;
mod repository;
mod path;

use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use openai::service::chat::chat;
use output::outputter;
use repository::db::SqliteRepository;
use std::io::IsTerminal;
use std::io::{self, Read};
use crate::args::Args;
use crate::config::repository::ConfigRepository;
use crate::path::extract::extract_content;
use crate::path::model::Files;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let repo = SqliteRepository::new("app.db")?;

    if let Some(ref chat_gpt_api_key) = args.chat_gpt_api_key {
        config_service::write_config(
            &repo,
            &ConfigKeys::ChatGptApiKey.to_key(),
            &chat_gpt_api_key,
        )?;
        return Ok(());
    }

    if args.print_config {
        return print_config(&repo);
    }

    let local_context = extract_content(&args.directory, &args.exclude);
    let input = extract_input_or_quit(&args);
    request_response_from_ai(&repo, &input, &local_context).await
}

fn print_config<R: ConfigRepository>(repo: &R) -> Result<()> {
    match config_service::fetch_config(repo) {
        Ok(configs) => {
            configs.iter().for_each(|config| println!("{:} -> {:}", config.key, config.value));
            Ok(())
        }
        Err(_) => {
            println!("failed to fetch config");
            Ok(())
        }
    }
}

async fn request_response_from_ai<R: ConfigRepository>(repo: &R, input: &String, local_context: &Option<Vec<Files>>) -> Result<()> {
    let open_ai_api_key = config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())?;
    let input_with_local_context = match local_context {
        Some(files) => {
            let local_context: Vec<String> = files.iter().map(|file| {
                let file_path = file.path.clone();
                let file_content = file.content.clone();
                format!("{}\n```{}```", file_path, file_content)
            }).collect();
            format!("{}\n{}", input, local_context.join("\n"))
        }
        None => input.clone(),
    };

    let chat_response = match chat(&open_ai_api_key.value, &input_with_local_context).await {
        Ok(response) => response,
        Err(err) => {
            println!("{:#?}", err);
            return Err(err);
        }
    };

    let output_messages = chat_response
        .iter()
        .map(|message| message.to_output_message())
        .collect();

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
