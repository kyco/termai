//! Codex OAuth authentication command handlers
//!
//! Handles login-codex, logout-codex, and codex-status commands.

use crate::auth::oauth_client::OAuthClient;
use crate::auth::token_manager::{AuthStatus, TokenManager};
use crate::config::repository::ConfigRepository;
use anyhow::Result;
use colored::*;

/// Handle the login-codex command
///
/// Starts the OAuth flow to authenticate with OpenAI Codex.
pub async fn handle_login_codex<R: ConfigRepository>(repo: &R) -> Result<()> {
    println!("{}", "🔐 OpenAI Codex Authentication".bright_cyan().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());
    println!();

    let token_manager = TokenManager::new(repo);

    // Check if already authenticated
    if token_manager.is_authenticated() {
        println!(
            "{}",
            "You are already authenticated with Codex.".yellow()
        );
        println!();

        let status = token_manager.auth_status();
        println!("Current status: {}", format!("{}", status).bright_white());
        println!();

        println!(
            "{}",
            "To re-authenticate, first run 'termai config logout-codex'.".white()
        );
        return Ok(());
    }

    println!("This will open your browser to authenticate with your OpenAI account.");
    println!("You need a ChatGPT Plus or Pro subscription to use Codex.");
    println!();

    println!("{}", "Opening browser for authentication...".bright_yellow());
    println!();

    // Start OAuth flow
    let oauth_client = OAuthClient::new();
    let tokens = oauth_client.authorize().await?;

    // Save tokens
    token_manager.save_tokens(&tokens)?;

    println!();
    println!("{}", "✅ Authentication successful!".bright_green().bold());
    println!();

    // Show expiry info
    println!(
        "Token expires: {}",
        tokens.expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_white()
    );

    if tokens.refresh_token.is_some() {
        println!(
            "{}",
            "Refresh token saved - your session will be automatically renewed.".green()
        );
    }

    println!();
    println!("{}", "You can now use Codex as your AI provider:".white());
    println!(
        "   {}",
        "termai config set-provider openai-codex".cyan()
    );
    println!("   {}", "termai chat".cyan());
    println!();

    Ok(())
}

/// Handle the logout-codex command
///
/// Clears stored OAuth tokens.
pub fn handle_logout_codex<R: ConfigRepository>(repo: &R) -> Result<()> {
    println!("{}", "🔓 OpenAI Codex Logout".bright_cyan().bold());
    println!("{}", "══════════════════════".white().dimmed());
    println!();

    let token_manager = TokenManager::new(repo);

    if !token_manager.is_authenticated() {
        println!("{}", "You are not currently authenticated with Codex.".yellow());
        return Ok(());
    }

    token_manager.clear_tokens()?;

    println!("{}", "✅ Successfully logged out from Codex.".bright_green());
    println!();
    println!(
        "{}",
        "Your OAuth tokens have been removed.".white()
    );
    println!(
        "Run '{}' to authenticate again.",
        "termai config login-codex".cyan()
    );

    Ok(())
}

/// Handle the codex-status command
///
/// Shows current Codex authentication status.
pub fn handle_codex_status<R: ConfigRepository>(repo: &R) -> Result<()> {
    println!("{}", "📊 OpenAI Codex Status".bright_cyan().bold());
    println!("{}", "══════════════════════".white().dimmed());
    println!();

    let token_manager = TokenManager::new(repo);
    let status = token_manager.auth_status();

    match &status {
        AuthStatus::NotAuthenticated => {
            println!("{}", "Status: Not authenticated".red());
            println!();
            println!(
                "Run '{}' to authenticate with your ChatGPT Plus/Pro subscription.",
                "termai config login-codex".cyan()
            );
        }
        AuthStatus::Authenticated { expires_at } => {
            println!("{}", "Status: Authenticated ✅".bright_green());
            println!();
            println!(
                "Token expires: {}",
                expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string().bright_white()
            );

            // Calculate time remaining
            let now = chrono::Utc::now();
            if *expires_at > now {
                let duration = *expires_at - now;
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;

                if hours > 0 {
                    println!(
                        "Time remaining: {} hours, {} minutes",
                        hours.to_string().bright_green(),
                        minutes.to_string().bright_green()
                    );
                } else {
                    println!(
                        "Time remaining: {} minutes",
                        minutes.to_string().bright_yellow()
                    );
                }
            }

            println!();
            println!("{}", "Your Codex authentication is active.".green());
        }
        AuthStatus::Expired { can_refresh } => {
            println!("{}", "Status: Token expired ⚠️".yellow());
            println!();

            if *can_refresh {
                println!(
                    "{}",
                    "Your token has expired but can be refreshed automatically.".white()
                );
                println!("The next API call will attempt to refresh your token.");
            } else {
                println!(
                    "{}",
                    "Your token has expired and needs to be renewed.".red()
                );
                println!();
                println!(
                    "Run '{}' to re-authenticate.",
                    "termai config login-codex".cyan()
                );
            }
        }
    }

    println!();

    Ok(())
}
