mod args;
mod config;
mod openai;
mod repository;

use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use openai::service::chat::chat;
use repository::db::SqliteRepository;
use std::io::IsTerminal;
use std::io::{self, Read};

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let repo = SqliteRepository::new("app.db")?;

    if let Some(chat_gpt_api_key) = args.chat_gpt_api_key {
        config_service::write_config(
            &repo,
            &ConfigKeys::ChatGptApiKey.to_key(),
            &chat_gpt_api_key,
        )?;
    }

    if args.print_config {
        match config_service::fetch_config(&repo) {
            Ok(configs) => configs.iter().for_each(|config| {
                println!("{:} -> {:}", config.key, config.value);
            }),
            Err(_) => println!("failed to fetch config"),
        }
    }

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

    let openaikey = config_service::fetch_by_key(&repo, &ConfigKeys::ChatGptApiKey.to_key())?;
    let chat_response = chat(&openaikey.value, &input).await?;

    Ok(())
}
