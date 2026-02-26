mod args;
mod auth;
mod branch;
mod chat;
mod commands;
mod common;
mod completion;
mod config;
mod context;
mod discovery;
mod git;
mod llm;
mod manpage;
mod output;
mod path;
mod preset;
mod redactions;
mod repository;
mod session;
mod setup;
mod ui;

use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use clap::{error::ErrorKind, Parser};
use colored::*;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize with better error handling
    let args = match args::Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            if err.kind() == ErrorKind::DisplayHelp || err.kind() == ErrorKind::DisplayVersion {
                err.exit();
            } else {
                eprintln!("{}", "❌ Invalid command line arguments".red().bold());
                eprintln!("{}", err);

                // Show intelligent suggestions based on the error
                let error_string = err.to_string();
                let suggestions = discovery::CommandDiscovery::suggest_for_error(
                    &args::Args::default(),
                    &error_string,
                );

                if !suggestions.is_empty() {
                    eprintln!("\n{}", "💡 Suggestions:".bright_yellow().bold());
                    for suggestion in suggestions.iter().take(3) {
                        eprintln!("   • {}", suggestion.cyan());
                    }
                } else {
                    eprintln!("\n{}", "💡 Quick help:".bright_yellow().bold());
                    eprintln!("   termai --help           # Show detailed help");
                    eprintln!("   termai ask \"question\"   # Ask a quick question");
                    eprintln!("   termai chat             # Start chat session");
                }

                eprintln!(
                    "\n{}",
                    "🔍 For more guidance, run: termai discovery".bright_blue()
                );
                std::process::exit(1);
            }
        }
    };

    let db_path = db_path().map_err(enhance_database_path_error)?;

    let repo = SqliteRepository::new(db_path.to_str().unwrap())
        .context("❌ Failed to initialize database")
        .map_err(enhance_database_init_error)?;

    // Try to dispatch to subcommands first
    if commands::dispatch_command(&args, &repo).await? {
        return Ok(());
    }

    // Handle legacy patterns with deprecation warnings
    if commands::handle_legacy_patterns(&args, &repo)
        .context("❌ Failed to handle legacy command patterns")
        .map_err(enhance_legacy_error)?
    {
        return Ok(());
    }

    // No subcommand provided and no legacy patterns matched
    // Show help and suggest using subcommands
    println!(
        "{}",
        "🤖 TermAI - A powerful, privacy-focused AI assistant for your terminal"
            .bright_cyan()
            .bold()
    );
    println!();
    println!(
        "{}",
        "💡 Use subcommands for better organization:"
            .bright_yellow()
            .bold()
    );
    println!(
        "   {}      # Ask a one-shot question",
        "termai ask \"your question\"".cyan()
    );
    println!(
        "   {}                     # Start interactive session",
        "termai chat".cyan()
    );
    println!(
        "   {}                    # Run configuration wizard",
        "termai setup".cyan()
    );
    println!(
        "   {}             # Manage sessions",
        "termai session list".cyan()
    );
    println!(
        "   {}              # View configuration",
        "termai config show".cyan()
    );
    println!();
    println!(
        "Use {} for detailed command information.",
        "termai --help".bright_cyan()
    );
    println!();
    println!(
        "{} {}",
        "💭 Quick example:".bright_yellow().bold(),
        "termai ask \"explain this code\" src/main.rs".cyan()
    );

    Ok(())
}

fn db_path() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let default_dir = home_dir.join(".config/termai");
    create_dir_all(&default_dir).context("Failed to create TermAI configuration directory")?;
    Ok(default_dir.join("app.db"))
}

/// Enhanced error handlers for main initialization
fn enhance_database_path_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Database Path Troubleshooting:".bright_yellow().bold(),
        "   Could not determine database location. Try these steps:".white(),
        "Ensure you have permission to write to ~/.config/".cyan(),
        "Try creating the directory manually: mkdir -p ~/.config/termai".cyan(),
        "Check if your HOME environment variable is set correctly".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_database_init_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Database Initialization Troubleshooting:"
            .bright_yellow()
            .bold(),
        "   Could not initialize SQLite database. Try these steps:".white(),
        "Check if ~/.config/termai/app.db is writable".cyan(),
        "Try removing the database file to recreate it: rm ~/.config/termai/app.db".cyan(),
        "Ensure you have sufficient disk space available".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_legacy_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}",
        "💡 Legacy Command Troubleshooting:".bright_yellow().bold(),
        "   Legacy command handling failed. Try these steps:".white(),
        "Use the new subcommand structure instead (termai --help)".cyan(),
        "Check that your arguments are properly quoted".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}
