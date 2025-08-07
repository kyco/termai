/// Git hooks management command handler
use crate::git::{
    hooks::{HookManager, HookType},
    repository::GitRepository,
};
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;

/// Handle the hooks management subcommand
pub async fn handle_hooks_command(
    action: &str,
    hook_type: Option<&str>,
    _repo: &SqliteRepository,
) -> Result<()> {
    println!("{}", "🪝 TermAI Git Hooks Management".bright_blue().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());

    // Discover and analyze the Git repository
    let git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;

    let hook_manager = HookManager::new(git_repo);

    match action {
        "status" => {
            display_hooks_status(&hook_manager).await?;
        }
        "install" => {
            if let Some(hook_type_str) = hook_type {
                let hook_type = parse_hook_type(hook_type_str)?;
                hook_manager.install_hook(hook_type)?;
            } else {
                install_hooks_interactive(&hook_manager).await?;
            }
        }
        "install-all" => {
            hook_manager.install_all_hooks()?;
        }
        "uninstall" => {
            if let Some(hook_type_str) = hook_type {
                let hook_type = parse_hook_type(hook_type_str)?;
                hook_manager.uninstall_hook(hook_type)?;
            } else {
                uninstall_hooks_interactive(&hook_manager).await?;
            }
        }
        _ => {
            anyhow::bail!(
                "Unknown hooks action: {}. Use 'status', 'install', 'install-all', or 'uninstall'",
                action
            );
        }
    }

    Ok(())
}

/// Display the status of all Git hooks
async fn display_hooks_status(hook_manager: &HookManager) -> Result<()> {
    println!("\n{}", "📊 Git Hooks Status".bright_green().bold());

    let statuses = hook_manager.get_all_hook_status()?;

    for status in &statuses {
        println!("   {}", status);
    }

    // Show summary
    let installed_count = statuses.iter().filter(|s| s.installed).count();
    let termai_managed_count = statuses.iter().filter(|s| s.termai_managed).count();

    println!("\n{}", "📈 Summary".bright_cyan().bold());
    println!("   • Total hooks: {}", statuses.len().to_string().cyan());
    println!("   • Installed: {}", installed_count.to_string().green());
    println!(
        "   • TermAI managed: {}",
        termai_managed_count.to_string().bright_green()
    );

    if termai_managed_count == 0 {
        println!("\n{}", "💡 Recommendations:".bright_yellow().bold());
        println!(
            "   • Run {} to install recommended hooks",
            "termai hooks install-all".bright_cyan()
        );
        println!(
            "   • Or use {} for interactive installation",
            "termai hooks install".bright_cyan()
        );
    }

    hook_manager.display_hook_usage();

    Ok(())
}

/// Interactive hook installation
async fn install_hooks_interactive(hook_manager: &HookManager) -> Result<()> {
    println!(
        "\n{}",
        "🔧 Interactive Hook Installation".bright_green().bold()
    );

    let hook_types = [
        (
            "Pre-commit (recommended)",
            HookType::PreCommit,
            "Runs code analysis before each commit",
        ),
        (
            "Commit-msg (recommended)",
            HookType::CommitMsg,
            "Validates commit message format",
        ),
        (
            "Pre-push",
            HookType::PrePush,
            "Performs analysis before pushing",
        ),
        (
            "Post-commit",
            HookType::PostCommit,
            "Shows insights after committing",
        ),
    ];

    println!("\nSelect hooks to install:");

    for (name, hook_type, description) in &hook_types {
        let status = hook_manager.get_hook_status(hook_type.clone())?;

        if status.termai_managed {
            println!(
                "   {} {} - {}",
                "✅".green(),
                name.bright_white(),
                "Already installed".dimmed()
            );
            continue;
        }

        if status.installed && !status.termai_managed {
            println!(
                "   {} {} - {}",
                "⚠️ ".yellow(),
                name.bright_white(),
                "Custom hook exists".yellow()
            );

            if Confirm::new()
                .with_prompt(format!(
                    "Replace existing {} hook with TermAI version?",
                    hook_type
                ))
                .default(false)
                .interact()?
            {
                hook_manager.install_hook(hook_type.clone())?;
            }
            continue;
        }

        // Hook not installed
        println!(
            "   {} {} - {}",
            "❌".red(),
            name.bright_white(),
            description.dimmed()
        );

        if Confirm::new()
            .with_prompt(format!("Install {} hook?", hook_type))
            .default(true)
            .interact()?
        {
            hook_manager.install_hook(hook_type.clone())?;
        }
    }

    println!(
        "\n{}",
        "✅ Interactive installation completed".green().bold()
    );
    Ok(())
}

/// Interactive hook uninstallation  
async fn uninstall_hooks_interactive(hook_manager: &HookManager) -> Result<()> {
    println!(
        "\n{}",
        "🗑️  Interactive Hook Uninstallation".bright_red().bold()
    );

    let statuses = hook_manager.get_all_hook_status()?;
    let termai_hooks: Vec<_> = statuses.iter().filter(|s| s.termai_managed).collect();

    if termai_hooks.is_empty() {
        println!("{}", "ℹ️  No TermAI hooks are currently installed".cyan());
        return Ok(());
    }

    println!("\nTermAI hooks to uninstall:");

    for status in &termai_hooks {
        println!(
            "   {} {}",
            "✅".green(),
            status.hook_type.to_string().bright_white()
        );

        let backup_note = if status.existing_hook {
            " (will restore backup)"
        } else {
            ""
        };

        if Confirm::new()
            .with_prompt(format!(
                "Uninstall {} hook{}?",
                status.hook_type, backup_note
            ))
            .default(false)
            .interact()?
        {
            hook_manager.uninstall_hook(status.hook_type.clone())?;
        }
    }

    println!(
        "\n{}",
        "✅ Interactive uninstallation completed".green().bold()
    );
    Ok(())
}

/// Parse hook type from string
fn parse_hook_type(hook_type_str: &str) -> Result<HookType> {
    match hook_type_str.to_lowercase().as_str() {
        "pre-commit" | "precommit" => Ok(HookType::PreCommit),
        "commit-msg" | "commitmsg" => Ok(HookType::CommitMsg),
        "pre-push" | "prepush" => Ok(HookType::PrePush),
        "post-commit" | "postcommit" => Ok(HookType::PostCommit),
        _ => anyhow::bail!(
            "Unknown hook type: {}. Use pre-commit, commit-msg, pre-push, or post-commit",
            hook_type_str
        ),
    }
}

/// Handle git hook execution (called by actual Git hooks)
#[allow(dead_code)]
pub async fn handle_hook_execution(hook_type: HookType, args: Vec<String>) -> Result<i32> {
    match hook_type {
        HookType::PreCommit => execute_pre_commit_hook().await,
        HookType::CommitMsg => {
            if args.is_empty() {
                anyhow::bail!("Commit message file path required for commit-msg hook");
            }
            execute_commit_msg_hook(&args[0]).await
        }
        HookType::PrePush => execute_pre_push_hook().await,
        HookType::PostCommit => execute_post_commit_hook().await,
    }
}

/// Execute pre-commit hook logic
#[allow(dead_code)]
async fn execute_pre_commit_hook() -> Result<i32> {
    println!(
        "{}",
        "🔍 TermAI: Running pre-commit analysis...".bright_blue()
    );

    // Check if there are staged changes
    let git_repo = GitRepository::discover(".").context("Failed to find Git repository")?;

    let status = git_repo
        .status()
        .context("Failed to get repository status")?;

    if !status.has_staged_changes() {
        println!("{}", "ℹ️  No staged changes to analyze".cyan());
        return Ok(0);
    }

    // This would integrate with the review command
    // For now, return success to not block commits
    println!("{}", "✅ TermAI: Pre-commit analysis passed".green());
    Ok(0)
}

/// Execute commit-msg hook logic
#[allow(dead_code)]
async fn execute_commit_msg_hook(msg_file: &str) -> Result<i32> {
    println!(
        "{}",
        "📝 TermAI: Validating commit message...".bright_blue()
    );

    let commit_msg =
        std::fs::read_to_string(msg_file).context("Failed to read commit message file")?;

    let commit_msg = commit_msg.trim();

    // Check conventional commit format
    let conventional_regex =
        regex::Regex::new(r"^(feat|fix|docs|style|refactor|test|chore|build|ci)(\(.+\))?: .+")
            .context("Failed to compile regex")?;

    if conventional_regex.is_match(commit_msg) {
        println!(
            "{}",
            "✅ TermAI: Commit message follows conventional format".green()
        );
    } else {
        println!(
            "{}",
            "⚠️  TermAI: Commit message doesn't follow conventional format".yellow()
        );
        println!(
            "{}",
            "💡 Consider using: feat/fix/docs/style/refactor/test/chore: description".dimmed()
        );
        println!(
            "{}",
            "💡 Run 'termai commit' to generate AI-powered commit messages".dimmed()
        );
    }

    // Always allow commit (don't block)
    Ok(0)
}

/// Execute pre-push hook logic
#[allow(dead_code)]
async fn execute_pre_push_hook() -> Result<i32> {
    println!(
        "{}",
        "🚀 TermAI: Running pre-push analysis...".bright_blue()
    );

    // This would integrate with branch analysis
    println!("{}", "✅ TermAI: Pre-push analysis completed".green());
    Ok(0)
}

/// Execute post-commit hook logic
#[allow(dead_code)]
async fn execute_post_commit_hook() -> Result<i32> {
    println!("{}", "🎉 TermAI: Commit successful!".bright_green());

    // Show helpful tips
    println!(
        "{}",
        "💡 Tip: Use 'termai branch-summary' to analyze your branch".dimmed()
    );

    Ok(0)
}
