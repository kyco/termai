mod args;
mod config;
mod repository;

use anyhow::Result;
use clap::Parser;
use config::{model::keys::ConfigKeys, service::config_service};
use repository::db::SqliteRepository;

fn main() -> Result<()> {
    let args = args::Args::parse();
    let repo = SqliteRepository::new("app.db")?;

    if let Some(chat_gpt_api_key) = args.chat_gpt_api_key {
        config_service::write_config(
            &repo,
            &ConfigKeys::chat_gpt_api_key.to_key(),
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

    Ok(())
}
