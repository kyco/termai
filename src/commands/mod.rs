/// Command structure and handlers for TermAI CLI
///
/// This module provides a clean separation of command logic from argument parsing,
/// making it easier to test, maintain, and extend the CLI functionality.
pub mod ask;
pub mod branch;
pub mod chat;
pub mod codex_auth;
pub mod commit;
pub mod completion;
pub mod config;
pub mod conflicts;
pub mod help;
pub mod hooks;
pub mod preset;
pub mod rebase;
pub mod review;
pub mod session;
pub mod setup;
pub mod stash;
pub mod tag;

#[cfg(test)]
mod tests;

use crate::args::{Args, ArgumentValidator, Commands};
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use clap::CommandFactory;
use colored::*;

/// Main command dispatcher that routes commands to their appropriate handlers
/// Provides enhanced error handling with actionable guidance messages
pub async fn dispatch_command(args: &Args, repo: &SqliteRepository) -> Result<bool> {
    // Validate arguments before processing
    if let Err(validation_error) = ArgumentValidator::validate(args) {
        ArgumentValidator::display_validation_error(&validation_error);
        std::process::exit(1);
    }

    match &args.command {
        Some(Commands::Setup(setup_args)) => {
            setup::handle_setup_command(repo, setup_args)
                .await
                .context("❌ Setup wizard failed")
                .map_err(enhance_setup_error)?;
            Ok(true)
        }
        Some(Commands::Config {
            args: config_args,
            action,
        }) => {
            config::handle_config_command(repo, action, config_args)
                .await
                .context("❌ Configuration command failed")
                .map_err(|e| enhance_config_error(e, action))?;
            Ok(true)
        }
        Some(Commands::Redact {
            args: redact_args,
            action,
        }) => {
            config::handle_redact_command(repo, action, redact_args)
                .context("❌ Redaction command failed")
                .map_err(|e| enhance_redact_error(e, action))?;
            Ok(true)
        }
        Some(Commands::Sessions {
            args: session_args,
            action,
        }) => {
            session::handle_sessions_command(repo, action, session_args)
                .context("❌ Session command failed")
                .map_err(|e| enhance_session_error(e, action))?;
            Ok(true)
        }
        Some(Commands::Chat(chat_args)) => {
            chat::handle_chat_command(chat_args, repo)
                .await
                .context("❌ Chat session failed")
                .map_err(enhance_chat_error)?;
            Ok(true)
        }
        Some(Commands::Ask(ask_args)) => {
            ask::handle_ask_command(ask_args, repo)
                .await
                .context("❌ Ask command failed")
                .map_err(enhance_ask_error)?;
            Ok(true)
        }
        Some(Commands::Commit(commit_args)) => {
            commit::handle_commit_command(commit_args, repo)
                .await
                .context("❌ Commit command failed")
                .map_err(enhance_commit_error)?;
            Ok(true)
        }
        Some(Commands::Review(review_args)) => {
            review::handle_review_command(review_args, repo)
                .await
                .context("❌ Review command failed")
                .map_err(enhance_review_error)?;
            Ok(true)
        }
        Some(Commands::BranchSummary(branch_args)) => {
            if branch_args.release_notes {
                branch::generate_release_notes(
                    branch_args.from_tag.as_deref().unwrap_or("HEAD~10"),
                    branch_args.to_tag.as_deref(),
                    repo,
                )
                .await
                .context("❌ Release notes generation failed")
                .map_err(enhance_branch_error)?;
            } else if branch_args.pr_description {
                branch::generate_pr_description(
                    branch_args.branch.as_deref(),
                    branch_args.base_branch.as_deref(),
                    repo,
                )
                .await
                .context("❌ PR description generation failed")
                .map_err(enhance_branch_error)?;
            } else if branch_args.suggest_name {
                branch::suggest_branch_name(branch_args.context.as_deref(), repo)
                    .await
                    .context("❌ Branch naming suggestions failed")
                    .map_err(enhance_branch_error)?;
            } else {
                branch::handle_branch_summary_command(branch_args.branch.as_deref(), repo)
                    .await
                    .context("❌ Branch analysis failed")
                    .map_err(enhance_branch_error)?;
            }
            Ok(true)
        }
        Some(Commands::Hooks(hooks_args)) => {
            hooks::handle_hooks_command(&hooks_args.action, hooks_args.hook_type.as_deref(), repo)
                .await
                .context("❌ Hooks command failed")
                .map_err(enhance_hooks_error)?;
            Ok(true)
        }
        Some(Commands::Stash(stash_args)) => {
            stash::handle_stash_command(stash_args, repo)
                .await
                .context("❌ Stash command failed")
                .map_err(enhance_stash_error)?;
            Ok(true)
        }
        Some(Commands::Tag(tag_args)) => {
            tag::handle_tag_command(tag_args, repo)
                .await
                .context("❌ Tag command failed")
                .map_err(enhance_tag_error)?;
            Ok(true)
        }
        Some(Commands::Rebase(rebase_args)) => {
            rebase::handle_rebase_command(rebase_args, repo)
                .await
                .context("❌ Rebase command failed")
                .map_err(enhance_rebase_error)?;
            Ok(true)
        }
        Some(Commands::Conflicts(conflicts_args)) => {
            conflicts::handle_conflicts_command(conflicts_args, repo)
                .await
                .context("❌ Conflicts command failed")
                .map_err(enhance_conflicts_error)?;
            Ok(true)
        }
        Some(Commands::Preset(preset_args)) => {
            preset::handle_preset_command(preset_args, repo)
                .await
                .context("❌ Preset command failed")
                .map_err(enhance_preset_error)?;
            Ok(true)
        }
        Some(Commands::Completion {
            args: completion_args,
            action,
        }) => {
            let mut cmd = Args::command();
            completion::handle_completion_command(action, completion_args, &mut cmd)
                .context("❌ Completion generation failed")
                .map_err(enhance_completion_error)?;
            Ok(true)
        }
        Some(Commands::Complete { args }) => {
            // Hidden completion command for shell integration
            crate::completion::DynamicCompleter::print_completions(repo, args)?;
            Ok(true)
        }
        Some(Commands::Help) => {
            // Hidden help command that shows discovery suggestions
            crate::discovery::display_discovery_help();
            Ok(true)
        }
        Some(Commands::Man {
            output,
            install_help,
        }) => {
            // Hidden man page generation command
            if *install_help {
                crate::manpage::ManPageGenerator::display_installation_instructions();
            } else if let Some(path) = output {
                crate::manpage::ManPageGenerator::generate_to_file(path)
                    .context("❌ Failed to generate man page file")?;
            } else {
                crate::manpage::ManPageGenerator::generate_to_stdout()
                    .context("❌ Failed to generate man page")?;
            }
            Ok(true)
        }
        None => {
            // No subcommand provided - check for legacy patterns
            Ok(false)
        }
    }
}

/// Check and handle legacy command patterns with deprecation warnings
pub fn handle_legacy_patterns(args: &Args, repo: &SqliteRepository) -> Result<bool> {
    // Check for legacy patterns and show deprecation warnings
    let mut handled = false;

    if args.is_chat_gpt_api_key() {
        eprintln!("⚠️  Warning: --chat-gpt-api-key is deprecated. Use 'termai config set-openai <key>' instead.");
        crate::config::service::open_ai_config::write_open_ai_key(repo, args)?;
        handled = true;
    }

    if args.is_claude_api_key() {
        eprintln!("⚠️  Warning: --claude-api-key is deprecated. Use 'termai config set-claude <key>' instead.");
        crate::config::service::claude_config::write_claude_key(repo, args)?;
        handled = true;
    }

    if args.is_redaction() {
        eprintln!("⚠️  Warning: Legacy redaction flags are deprecated. Use 'termai redact' subcommands instead.");
        crate::config::service::redacted_config::redaction(repo, args)?;
        handled = true;
    }

    if args.is_sessions_all() {
        eprintln!("⚠️  Warning: --sessions-all is deprecated. Use 'termai session list' instead.");
        crate::session::service::sessions_service::fetch_all_sessions(repo, repo)?;
        handled = true;
    }

    if args.is_provider() {
        eprintln!("⚠️  Warning: --provider is deprecated. Use 'termai config set-provider <provider>' instead.");
        crate::config::service::provider_config::write_provider_key(repo, args)?;
        handled = true;
    }

    if args.is_config_show() {
        eprintln!("⚠️  Warning: --print-config is deprecated. Use 'termai config show' instead.");
        print_config(repo)?;
        handled = true;
    }

    Ok(handled)
}

fn print_config(repo: &SqliteRepository) -> Result<()> {
    let configs = crate::config::service::config_service::fetch_config(repo)?;
    println!("📋 Current Configuration");
    println!("═══════════════════════");
    for config in configs {
        println!("{} = {}", config.key, config.value);
    }
    Ok(())
}

/// Enhanced error handlers that provide actionable guidance messages
fn enhance_setup_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Setup Troubleshooting:".bright_yellow().bold(),
        "   The setup wizard encountered an issue. Try these steps:".white(),
        "Run 'termai config show' to check current configuration".cyan(),
        "Ensure you have a stable internet connection for API validation".cyan(),
        "Try running 'termai config reset' if configuration is corrupted".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_config_error(
    error: anyhow::Error,
    _action: &crate::args::ConfigAction,
) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Configuration Troubleshooting:".bright_yellow().bold(),
        "   Configuration management failed. Try these steps:".white(),
        "Check if the configuration file is writable (~/.config/termai/)".cyan(),
        "Run 'termai config show' to see current settings".cyan(),
        "Use 'termai setup' to reconfigure from scratch".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_redact_error(
    error: anyhow::Error,
    _action: &crate::args::RedactAction,
) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Redaction Troubleshooting:".bright_yellow().bold(),
        "   Redaction pattern management failed. Try these steps:".white(),
        "Check if redaction patterns contain valid syntax".cyan(),
        "Run 'termai redact list' to see current patterns".cyan(),
        "Use 'termai config show' to verify redaction configuration".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_session_error(
    error: anyhow::Error,
    _action: &crate::args::SessionAction,
) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Session Troubleshooting:".bright_yellow().bold(),
        "   Session management failed. Try these steps:".white(),
        "Run 'termai session list' to see available sessions".cyan(),
        "Check if the session name exists and is accessible".cyan(),
        "Verify database permissions (~/.config/termai/app.db)".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_chat_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Chat Session Troubleshooting:".bright_yellow().bold(),
        "   Chat session failed to start. Try these steps:".white(),
        "Run 'termai config show' to verify API keys are configured".cyan(),
        "Check your internet connection for API access".cyan(),
        "Use 'termai setup' to reconfigure API credentials".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_ask_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Ask Command Troubleshooting:".bright_yellow().bold(),
        "   Ask command failed. Try these steps:".white(),
        "Use 'termai chat' for now (Ask command is under development)".cyan(),
        "Check if API keys are properly configured with 'termai config show'".cyan(),
        "Ensure your question is properly quoted if it contains special characters".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_completion_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Completion Troubleshooting:".bright_yellow().bold(),
        "   Shell completion generation failed. Try these steps:".white(),
        "Run 'termai completion --help' to see available shells".cyan(),
        "Try a different shell format (bash, zsh, fish, powershell)".cyan(),
        "Ensure you have write permissions to save completion scripts".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_commit_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Commit Troubleshooting:".bright_yellow().bold(),
        "   Git commit message generation failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check that you have staged changes with 'git status'".cyan(),
        "Use --force to generate messages without staged changes".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_review_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Review Troubleshooting:".bright_yellow().bold(),
        "   Code review analysis failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check that you have staged changes with 'git status'".cyan(),
        "Use --files to focus on specific file patterns".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_branch_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Branch Analysis Troubleshooting:".bright_yellow().bold(),
        "   Branch analysis failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check that the specified branch exists with 'git branch'".cyan(),
        "Use --release-notes with --from-tag for release note generation".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_hooks_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Hooks Management Troubleshooting:"
            .bright_yellow()
            .bold(),
        "   Git hooks management failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check that .git/hooks directory is writable".cyan(),
        "Use 'termai hooks status' to check current hook status".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_stash_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Stash Management Troubleshooting:"
            .bright_yellow()
            .bold(),
        "   Git stash management failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check that you have changes to stash with 'git status'".cyan(),
        "Use 'termai stash list' to see existing stashes".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_tag_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Tag Management Troubleshooting:".bright_yellow().bold(),
        "   Git tag management failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Use semantic versioning format (e.g., v1.2.3)".cyan(),
        "Use 'termai tag list' to see existing tags".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_rebase_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Rebase Troubleshooting:".bright_yellow().bold(),
        "   Interactive rebase failed. Try these steps:".white(),
        "Ensure you're in a Git repository directory".cyan(),
        "Check rebase status with 'termai rebase status'".cyan(),
        "Use 'termai rebase abort' to cancel if needed".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_conflicts_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Conflict Resolution Troubleshooting:".bright_yellow().bold(),
        "   Conflict resolution failed. Try these steps:".white(),
        "Check for active conflicts with 'termai conflicts detect'".cyan(),
        "Use 'termai conflicts guide' for comprehensive help".cyan(),
        "Try manual resolution with 'git mergetool'".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}

fn enhance_preset_error(error: anyhow::Error) -> anyhow::Error {
    let guidance = format!(
        "\n{}\n{}\n• {}\n• {}\n• {}",
        "💡 Preset Management Troubleshooting:".bright_yellow().bold(),
        "   Preset operation failed. Try these steps:".white(),
        "Use 'termai preset list' to see available presets".cyan(),
        "Check preset name spelling and case sensitivity".cyan(),
        "Use 'termai preset --help' for command usage information".cyan()
    );
    anyhow::anyhow!("{}\n{}", error, guidance)
}
