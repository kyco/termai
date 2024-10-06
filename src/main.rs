mod args;
mod config;
mod openai;
mod output;
mod repository;

use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use openai::service::chat::chat;
use output::outputter;
use repository::db::SqliteRepository;
use std::io::IsTerminal;
use std::io::{self, Read};
use crate::args::Args;

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

    let input = extract_input_or_quit(&args);
    request_response_from_ai(&repo, &input).await
}

fn print_config(repo: &SqliteRepository) -> Result<()> {
    match config_service::fetch_config(&repo) {
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

async fn request_response_from_ai(repo: &SqliteRepository, input: &String) -> Result<()> {
    let open_ai_api_key = config_service::fetch_by_key(&repo, &ConfigKeys::ChatGptApiKey.to_key())?;
    let chat_response = match chat(&open_ai_api_key.value, &input).await {
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
    let input = if !args.data.is_empty() {
        args.data.join(" ")
    } else if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read stdin");
        buffer.trim().to_string()
    } else {
        eprintln!("No input provided. Use positional arguments or pipe data.");
        std::process::exit(1);
    };
    input
}
