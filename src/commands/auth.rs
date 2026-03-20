//! Authentication command handlers for the top-level `auth` surface.

use crate::args::{AuthAction, Provider};
use crate::config::model::keys::ConfigKeys;
use crate::config::service::config_service;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Input};

/// Handle `termai auth ...` commands.
pub async fn handle_auth_command(repo: &SqliteRepository, action: &AuthAction) -> Result<()> {
    match action {
        AuthAction::Login(args) => handle_login(repo, args.provider).await,
        AuthAction::Logout(args) => handle_logout(repo, args.provider),
        AuthAction::Status(args) => handle_status(repo, args.provider),
    }
}

async fn handle_login(repo: &SqliteRepository, provider: Provider) -> Result<()> {
    match provider {
        Provider::OpenaiCodex => crate::commands::codex_auth::handle_login_codex(repo).await,
        Provider::Openai => handle_api_key_login(repo, "OpenAI", &ConfigKeys::ChatGptApiKey.to_key()),
        Provider::Claude => handle_api_key_login(repo, "Claude", &ConfigKeys::ClaudeApiKey.to_key()),
    }
}

fn handle_logout(repo: &SqliteRepository, provider: Provider) -> Result<()> {
    match provider {
        Provider::OpenaiCodex => crate::commands::codex_auth::handle_logout_codex(repo),
        Provider::Openai => clear_api_key(repo, "OpenAI", &ConfigKeys::ChatGptApiKey.to_key()),
        Provider::Claude => clear_api_key(repo, "Claude", &ConfigKeys::ClaudeApiKey.to_key()),
    }
}

fn handle_status(repo: &SqliteRepository, provider: Provider) -> Result<()> {
    match provider {
        Provider::OpenaiCodex => crate::commands::codex_auth::handle_codex_status(repo),
        Provider::Openai => {
            let key = ConfigKeys::ChatGptApiKey.to_key();
            print_key_status(repo, "OpenAI", key.as_str())
        }
        Provider::Claude => {
            let key = ConfigKeys::ClaudeApiKey.to_key();
            print_key_status(repo, "Claude", key.as_str())
        }
    }
}

fn handle_api_key_login(repo: &SqliteRepository, provider: &str, config_key: &str) -> Result<()> {
    let theme = ColorfulTheme::default();
    let api_key: String = Input::with_theme(&theme)
        .with_prompt(format!("Enter your {} API key", provider))
        .interact_text()?;

    if api_key.trim().is_empty() {
        return Err(anyhow::anyhow!("API key cannot be empty"));
    }

    config_service::write_config(repo, config_key, api_key.trim())?;
    println!(
        "{}",
        format!("✅ {} API key stored successfully", provider)
            .green()
            .bold()
    );
    Ok(())
}

fn clear_api_key(repo: &SqliteRepository, provider: &str, config_key: &str) -> Result<()> {
    config_service::write_config(repo, config_key, "")?;
    println!(
        "{}",
        format!("✅ {} API key cleared", provider).green().bold()
    );
    Ok(())
}

fn print_key_status(repo: &SqliteRepository, provider: &str, config_key: &str) -> Result<()> {
    println!("{}", format!("📊 {} Status", provider).bright_cyan().bold());
    println!("{}", "══════════════════════".white().dimmed());
    println!();

    let configured = config_service::has_config(repo, config_key);
    if configured {
        println!("{}", "Status: Configured ✅".bright_green());
        println!(
            "{}",
            "The provider has a stored credential or environment fallback available.".white()
        );
    } else {
        println!("{}", "Status: Not configured".red());
        println!(
            "{}",
            "Run the matching auth login command or set the matching API key environment variable.".white()
        );
    }
    println!();

    Ok(())
}
