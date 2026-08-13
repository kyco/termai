/// Git stash management
use crate::git::repository::{GitRepository, StashEntry};
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::Confirm;

/// Handle the stash management subcommand
pub async fn handle_stash_command(
    args: &crate::args::StashArgs,
    _repo: &SqliteRepository,
) -> Result<()> {
    println!("{}", "📦 TermAI Git Stash Management".bright_blue().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());

    // Discover and analyze the Git repository
    let mut git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;

    match args.action.as_str() {
        "list" => {
            list_stashes(&mut git_repo).await?;
        }
        "push" | "save" => {
            push_stash(&mut git_repo, args).await?;
        }
        "pop" => {
            pop_stash(&mut git_repo, args).await?;
        }
        "apply" => {
            apply_stash(&mut git_repo, args).await?;
        }
        "drop" => {
            drop_stash(&mut git_repo, args).await?;
        }
        "show" => {
            show_stash(&mut git_repo, args).await?;
        }
        "clear" => {
            clear_stashes(&mut git_repo, args).await?;
        }
        _ => {
            anyhow::bail!("Unknown stash action: {}. Use 'list', 'push', 'pop', 'apply', 'drop', 'show', or 'clear'", args.action);
        }
    }

    Ok(())
}

/// List all stashes
async fn list_stashes(git_repo: &mut GitRepository) -> Result<()> {
    println!("\n{}", "📋 Git Stashes".bright_green().bold());

    let stashes = git_repo.stash_list()?;

    if stashes.is_empty() {
        println!("   {}", "No stashes found".dimmed());
        println!("\n{}", "💡 Create a stash with: termai stash push".cyan());
        return Ok(());
    }

    for stash in &stashes {
        println!(
            "\n   {}: {}",
            format!("stash@{{{}}}", stash.index).bright_yellow(),
            stash.message.bright_white(),
        );

        if let Ok(files) = git_repo.stash_changed_files(stash.id) {
            println!(
                "      {} {} file(s) changed",
                "📁".cyan(),
                files.len().to_string().cyan()
            );
        }
    }

    println!("\n{}", "💡 Quick Actions:".bright_green().bold());
    println!(
        "   • {} - Apply most recent stash",
        "termai stash pop".cyan()
    );
    println!(
        "   • {} - Show detailed changes",
        "termai stash show 0".cyan()
    );
    println!(
        "   • {} - Apply without removing from stash list",
        "termai stash apply 0".cyan()
    );

    Ok(())
}

/// Create a new stash
async fn push_stash(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    println!("\n{}", "💾 Creating Git Stash".bright_green().bold());

    // Check if there are changes to stash
    let status = git_repo
        .status()
        .context("Failed to get repository status")?;

    if status.is_clean {
        println!(
            "{}",
            "ℹ️  Working directory is clean - nothing to stash".cyan()
        );
        return Ok(());
    }

    // Show what will be stashed
    println!("\n{}", "📊 Changes to be stashed:".bright_cyan().bold());
    if status.has_staged_changes() {
        println!(
            "   • {} staged files",
            status.staged_files.len().to_string().green()
        );
    }
    if status.has_unstaged_changes() {
        println!(
            "   • {} unstaged files",
            status.unstaged_files.len().to_string().yellow()
        );
    }
    if status.has_untracked_files() && args.include_untracked {
        println!(
            "   • {} untracked files",
            status.untracked_files.len().to_string().bright_black()
        );
    }

    // Generate or get stash message
    let stash_message = if let Some(message) = &args.message {
        message.clone()
    } else {
        generate_stash_message(git_repo, &status)?
    };

    println!("\n{}", "💭 Stash message:".bright_cyan().bold());
    println!("   {}", stash_message.bright_white());

    // Confirm stash creation (skipped with --yes)
    if args.interactive
        && !args.yes
        && !Confirm::new()
            .with_prompt("Create stash with this message?")
            .default(true)
            .interact()?
    {
        println!("{}", "Stash creation cancelled".yellow());
        return Ok(());
    }

    // Create the stash for real
    println!("\n{}", "🔄 Creating stash...".cyan());
    git_repo.stash_push(&stash_message, args.include_untracked)?;

    let stash_options = if args.include_untracked {
        "with untracked files"
    } else {
        "tracked files only"
    };

    println!(
        "   {} Stash created: {} ({})",
        "✅".green(),
        stash_message.bright_white(),
        stash_options.dimmed()
    );

    println!("\n{}", "💡 Next steps:".bright_yellow().bold());
    println!("   • Use {} to see all stashes", "termai stash list".cyan());
    println!(
        "   • Use {} to restore these changes",
        "termai stash pop".cyan()
    );
    println!(
        "   • Use {} to apply without removing from stash",
        "termai stash apply".cyan()
    );

    Ok(())
}

/// Find a stash entry by index, with a helpful error when missing
fn find_stash(stashes: &[StashEntry], index: usize) -> Result<&StashEntry> {
    stashes.iter().find(|s| s.index == index).ok_or_else(|| {
        anyhow::anyhow!(
            "stash@{{{}}} not found ({} stash(es) exist)",
            index,
            stashes.len()
        )
    })
}

/// Pop (apply and remove) the most recent or specified stash
async fn pop_stash(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    println!("\n{}", "📤 Popping Git Stash".bright_green().bold());

    let index = args.stash_index.unwrap_or(0);
    let stashes = git_repo.stash_list()?;
    let stash = find_stash(&stashes, index)?;
    let stash_ref = format!("stash@{{{}}}", index);

    println!("\n{}", "📋 Restoring changes from:".bright_cyan().bold());
    println!(
        "   {} {}",
        stash_ref.bright_yellow(),
        stash.message.bright_white()
    );

    if args.interactive
        && !args.yes
        && !Confirm::new()
            .with_prompt(format!("Apply and remove {}?", stash_ref))
            .default(true)
            .interact()?
    {
        println!("{}", "Stash pop cancelled".yellow());
        return Ok(());
    }

    println!("\n{}", "🔄 Applying stash changes...".cyan());
    git_repo.stash_pop(index)?;

    println!("   {} Changes applied successfully", "✅".green());
    println!("   {} {} removed from stash list", "🗑️ ".red(), stash_ref);

    Ok(())
}

/// Apply stash without removing from stash list
async fn apply_stash(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    println!("\n{}", "📥 Applying Git Stash".bright_green().bold());

    let index = args.stash_index.unwrap_or(0);
    let stashes = git_repo.stash_list()?;
    let stash = find_stash(&stashes, index)?;
    let stash_ref = format!("stash@{{{}}}", index);

    println!(
        "   {} Applying changes from: {} ({})",
        "🔄".cyan(),
        stash_ref.bright_yellow(),
        stash.message.white()
    );

    git_repo.stash_apply(index)?;

    println!("   {} Changes applied successfully", "✅".green());
    println!("   {} {} remains in stash list", "📋".blue(), stash_ref);

    println!("\n{}", "💡 Note:".bright_yellow().bold());
    println!("   • The stash is still available for future use");
    println!(
        "   • Use {} to remove it from the stash list",
        "termai stash drop".cyan()
    );

    Ok(())
}

/// Drop (delete) a stash without applying
async fn drop_stash(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    println!("\n{}", "🗑️  Dropping Git Stash".bright_red().bold());

    let index = args.stash_index.unwrap_or(0);
    let stashes = git_repo.stash_list()?;
    let stash = find_stash(&stashes, index)?;
    let stash_ref = format!("stash@{{{}}}", index);

    println!("\n{}", "⚠️  Warning:".bright_yellow().bold());
    println!(
        "   This will permanently delete {} ({})",
        stash_ref.bright_yellow(),
        stash.message.white()
    );
    println!("   This action cannot be undone");

    // --yes skips the confirmation prompt (short-circuits before Confirm)
    if args.yes
        || Confirm::new()
            .with_prompt(format!("Are you sure you want to drop {}?", stash_ref))
            .default(false)
            .interact()?
    {
        git_repo.stash_drop(index)?;
        println!(
            "\n   {} {} has been dropped",
            "✅".green(),
            stash_ref.bright_yellow()
        );
    } else {
        println!("{}", "Stash drop cancelled".yellow());
    }

    Ok(())
}

/// Show detailed information about a stash
async fn show_stash(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    let index = args.stash_index.unwrap_or(0);
    let stashes = git_repo.stash_list()?;
    let stash = find_stash(&stashes, index)?.clone();
    let stash_ref = format!("stash@{{{}}}", index);

    println!(
        "\n{}",
        format!("🔍 Stash Details: {}", stash_ref)
            .bright_green()
            .bold()
    );
    println!("{}", "═══════════════════════════════".white().dimmed());

    println!("\n{}", "📋 Stash Information:".bright_cyan().bold());
    println!("   {} {}", "Message:".bright_white(), stash.message);

    let files = git_repo.stash_changed_files(stash.id)?;
    println!(
        "   {} {} file(s) changed",
        "Files:".bright_white(),
        files.len()
    );

    if !files.is_empty() {
        println!("\n{}", "📁 Files Changed:".bright_cyan().bold());
        for (file, status) in &files {
            let status_str = status.to_string();
            let status_color = match status {
                'A' => status_str.green(),
                'M' => status_str.yellow(),
                'D' => status_str.red(),
                _ => status_str.normal(),
            };
            println!("   {} {}", status_color, file.bright_white());
        }
    }

    println!("\n{}", "💡 Actions:".bright_green().bold());
    println!(
        "   • {} - Apply and continue working",
        "termai stash pop".cyan()
    );
    println!(
        "   • {} - Apply without removing stash",
        "termai stash apply".cyan()
    );
    println!("   • {} - Show the full patch", "git stash show -p".cyan());

    Ok(())
}

/// Clear all stashes with confirmation (or non-interactively with --yes)
async fn clear_stashes(git_repo: &mut GitRepository, args: &crate::args::StashArgs) -> Result<()> {
    println!("\n{}", "🧹 Clearing All Stashes".bright_red().bold());

    let stash_count = git_repo.stash_list()?.len();

    if stash_count == 0 {
        println!("\n   {}", "No stashes to clear".dimmed());
        return Ok(());
    }

    println!("\n{}", "⚠️  Warning:".bright_yellow().bold());
    println!("   This will delete ALL stashes and their changes");
    println!("   This action cannot be undone");
    println!(
        "   {} stashes will be deleted",
        stash_count.to_string().bright_red()
    );

    // --yes skips the confirmation prompt (short-circuits before Confirm)
    if args.yes
        || Confirm::new()
            .with_prompt("Are you absolutely sure you want to clear all stashes?")
            .default(false)
            .interact()?
    {
        // Dropping index 0 repeatedly removes every stash
        for _ in 0..stash_count {
            git_repo.stash_drop(0)?;
        }
        println!("\n   {} All stashes have been cleared", "✅".green());
        println!("   {} {} stashes deleted", "🗑️ ".red(), stash_count);
    } else {
        println!("{}", "Stash clear cancelled".yellow());
    }

    Ok(())
}

/// Generate a stash message based on the actual working tree state
fn generate_stash_message(
    git_repo: &GitRepository,
    status: &crate::git::repository::RepoStatus,
) -> Result<String> {
    let current_branch = git_repo
        .current_branch()
        .unwrap_or_else(|_| "unknown".to_string());

    let message = if !status.staged_files.is_empty() && !status.unstaged_files.is_empty() {
        "WIP: Mixed staged and unstaged changes"
    } else if !status.staged_files.is_empty() {
        "WIP: Staged changes ready for commit"
    } else if !status.unstaged_files.is_empty() {
        "WIP: Unstaged changes in progress"
    } else {
        "WIP: Work in progress"
    };

    Ok(format!("{} on {}", message, current_branch))
}
