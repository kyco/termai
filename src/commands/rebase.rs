/// Git interactive rebase assistance
use crate::git::repository::GitRepository;
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;

/// Handle the rebase subcommand
pub async fn handle_rebase_command(
    args: &crate::args::RebaseArgs,
    _repo: &SqliteRepository,
) -> Result<()> {
    println!(
        "{}",
        "🔄 TermAI Interactive Rebase Assistant"
            .bright_blue()
            .bold()
    );
    println!(
        "{}",
        "════════════════════════════════════════".white().dimmed()
    );

    // Discover and analyze the Git repository
    let git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;

    // Check current rebase state
    let rebase_state = check_rebase_state(&git_repo)?;

    match args.action.as_str() {
        "start" | "interactive" => {
            start_interactive_rebase(&git_repo, args).await?;
        }
        "continue" => {
            continue_rebase(&rebase_state)?;
        }
        "abort" => {
            abort_rebase(&rebase_state)?;
        }
        "skip" => {
            skip_rebase_commit(&rebase_state)?;
        }
        "status" => {
            show_rebase_status(&rebase_state)?;
        }
        "plan" => {
            generate_rebase_plan(&git_repo, args).await?;
        }
        "analyze" => {
            analyze_commits_for_rebase(&git_repo, args).await?;
        }
        _ => {
            anyhow::bail!("Unknown rebase action: {}. Use 'start', 'continue', 'abort', 'skip', 'status', 'plan', or 'analyze'", args.action);
        }
    }

    Ok(())
}

/// Check the real rebase state via git2 and the .git/rebase-* metadata
fn check_rebase_state(git_repo: &GitRepository) -> Result<RebaseState> {
    let repo = git_repo.inner();
    let is_in_progress = git_repo.is_rebasing();

    let current_branch = git_repo
        .current_branch()
        .unwrap_or_else(|_| "unknown".to_string());

    let git_dir = repo.path();
    let mut current_step = 0usize;
    let mut total_steps = 0usize;
    let mut current_commit = None;

    if is_in_progress {
        // Interactive/merge rebases record progress in .git/rebase-merge
        let rebase_merge = git_dir.join("rebase-merge");
        let rebase_apply = git_dir.join("rebase-apply");
        let dir = if rebase_merge.exists() {
            Some((rebase_merge.clone(), "msgnum", "end"))
        } else if rebase_apply.exists() {
            Some((rebase_apply.clone(), "next", "last"))
        } else {
            None
        };

        if let Some((dir, step_file, total_file)) = dir {
            current_step = read_number_file(&dir.join(step_file)).unwrap_or(0);
            total_steps = read_number_file(&dir.join(total_file)).unwrap_or(0);
            current_commit = std::fs::read_to_string(dir.join("stopped-sha"))
                .ok()
                .map(|s| s.trim().to_string());
        }
    }

    // Real conflicts from the index
    let conflicts = git_repo
        .status()
        .map(|s| {
            s.conflicted_files
                .iter()
                .map(|f| f.path.display().to_string())
                .collect()
        })
        .unwrap_or_default();

    Ok(RebaseState {
        is_in_progress,
        current_commit,
        current_step,
        total_steps,
        branch: current_branch,
        conflicts,
    })
}

fn read_number_file(path: &std::path::Path) -> Option<usize> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// Start an interactive rebase session
async fn start_interactive_rebase(
    git_repo: &GitRepository,
    args: &crate::args::RebaseArgs,
) -> Result<()> {
    println!(
        "\n{}",
        "🚀 Starting Interactive Rebase".bright_green().bold()
    );

    // Get target for rebase
    let target = args
        .target
        .clone()
        .or_else(|| determine_rebase_target(git_repo));

    // Get real commits to rebase
    let commits = get_commits_for_rebase(git_repo, target.as_deref(), args.count)?;

    if commits.is_empty() {
        println!("\n{}", "ℹ️  No commits found for rebasing".yellow());
        return Ok(());
    }

    println!("\n{}", "📊 Rebase Analysis:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Target:".bright_white(),
        target
            .as_deref()
            .unwrap_or("(no upstream detected - recent commits)")
            .bright_yellow()
    );
    println!(
        "   {} {}",
        "Commits to rebase:".bright_white(),
        commits.len().to_string().cyan()
    );
    println!(
        "   {} {}",
        "Current branch:".bright_white(),
        git_repo
            .current_branch()
            .unwrap_or("unknown".to_string())
            .bright_blue()
    );

    // Analyze commits for potential issues
    let analysis = analyze_commits(&commits);
    show_rebase_analysis(&analysis);

    // Show the plan derived from real commits
    show_plan(&commits, &analysis);

    // Executing the rebase itself is not implemented - be honest about it
    let rebase_cmd = match &target {
        Some(target) => format!("git rebase -i {}", target),
        None => format!("git rebase -i HEAD~{}", commits.len()),
    };
    anyhow::bail!(
        "Interactive rebase execution is not supported yet.\n   Run '{}' directly to perform this rebase.",
        rebase_cmd
    );
}

/// Continue an interrupted rebase
fn continue_rebase(state: &RebaseState) -> Result<()> {
    println!(
        "\n{}",
        "▶️  Continuing Interactive Rebase".bright_green().bold()
    );

    if !state.is_in_progress {
        println!("\n{}", "ℹ️  No rebase in progress".yellow());
        println!("   Use 'git rebase <target>' to begin a new rebase");
        return Ok(());
    }

    if !state.conflicts.is_empty() {
        println!("\n{}", "⚠️  Unresolved Conflicts:".bright_red().bold());
        for conflict in &state.conflicts {
            println!("   • {}", conflict.red());
        }
        println!("\n{}", "💡 Resolution Steps:".bright_yellow().bold());
        println!("   1. Resolve conflicts in the listed files");
        println!("   2. Stage resolved files with 'git add'");
        println!("   3. Run 'git rebase --continue'");
    }

    anyhow::bail!(
        "Continuing a rebase is not supported yet. Run 'git rebase --continue' directly."
    );
}

/// Abort current rebase
fn abort_rebase(state: &RebaseState) -> Result<()> {
    println!("\n{}", "⏹️  Aborting Rebase".bright_red().bold());

    if !state.is_in_progress {
        println!(
            "\n{}",
            "ℹ️  No rebase in progress - nothing to abort".yellow()
        );
        return Ok(());
    }

    anyhow::bail!("Aborting a rebase is not supported yet. Run 'git rebase --abort' directly.");
}

/// Skip current commit in rebase
fn skip_rebase_commit(state: &RebaseState) -> Result<()> {
    println!("\n{}", "⏭️  Skipping Current Commit".bright_yellow().bold());

    if !state.is_in_progress {
        println!(
            "\n{}",
            "ℹ️  No rebase in progress - nothing to skip".yellow()
        );
        return Ok(());
    }

    anyhow::bail!(
        "Skipping a rebase commit is not supported yet. Run 'git rebase --skip' directly."
    );
}

/// Show current rebase status based on the real repository state
fn show_rebase_status(state: &RebaseState) -> Result<()> {
    println!("\n{}", "📊 Rebase Status".bright_green().bold());
    println!("{}", "═══════════════════".white().dimmed());

    if !state.is_in_progress {
        println!("\n   {} No rebase in progress", "ℹ️".cyan());
        println!(
            "   {} Current branch: {}",
            "📍".cyan(),
            state.branch.bright_blue()
        );
        return Ok(());
    }

    println!("\n{}", "🔄 Active Rebase:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Branch:".bright_white(),
        state.branch.bright_blue()
    );
    if state.total_steps > 0 {
        println!(
            "   {} {}/{}",
            "Progress:".bright_white(),
            state.current_step.to_string().green(),
            state.total_steps.to_string().cyan()
        );
    }

    if let Some(current) = &state.current_commit {
        println!(
            "   {} {}",
            "Current commit:".bright_white(),
            current.bright_yellow()
        );
    }

    if !state.conflicts.is_empty() {
        println!("\n{}", "⚠️  Conflicts:".bright_red().bold());
        for conflict in &state.conflicts {
            println!("   • {}", conflict.red());
        }

        println!("\n{}", "💡 Resolution Steps:".bright_yellow().bold());
        println!("   1. Resolve conflicts in the listed files");
        println!("   2. Stage resolved files with 'git add'");
        println!("   3. Continue with 'git rebase --continue'");
    }

    Ok(())
}

/// Generate and display a rebase plan from real commits
async fn generate_rebase_plan(
    git_repo: &GitRepository,
    args: &crate::args::RebaseArgs,
) -> Result<()> {
    println!("\n{}", "📋 Rebase Plan Generation".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());

    let target = args
        .target
        .clone()
        .or_else(|| determine_rebase_target(git_repo));

    let commits = get_commits_for_rebase(git_repo, target.as_deref(), args.count)?;

    if commits.is_empty() {
        println!("\n{}", "ℹ️  No commits found for rebasing".yellow());
        return Ok(());
    }

    println!("\n{}", "🎯 Rebase Target Analysis:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Target:".bright_white(),
        target
            .as_deref()
            .unwrap_or("(no upstream detected - recent commits)")
            .bright_yellow()
    );
    println!(
        "   {} {}",
        "Commits to rebase:".bright_white(),
        commits.len().to_string().cyan()
    );

    // Rule-based analysis of the real commits
    let analysis = analyze_commits(&commits);

    println!("\n{}", "📝 Rebase Recommendations:".bright_cyan().bold());

    let mut has_recommendation = false;
    if analysis.has_fixup_commits {
        println!(
            "   • {} Enable --autosquash to automatically handle fixup commits",
            "✨".green()
        );
        has_recommendation = true;
    }

    if analysis.has_large_commits {
        println!(
            "   • {} Consider splitting large commits for better history",
            "📝".yellow()
        );
        has_recommendation = true;
    }

    if analysis.has_merge_commits {
        println!(
            "   • {} Merge commits detected - consider --rebase-merges",
            "🔄".cyan()
        );
        has_recommendation = true;
    }

    if !has_recommendation {
        println!("   • {} No issues detected in these commits", "✅".green());
    }

    show_plan(&commits, &analysis);

    println!("\n{}", "💡 Next Steps:".bright_yellow().bold());
    let rebase_cmd = match &target {
        Some(target) => format!("git rebase -i {}", target),
        None => format!("git rebase -i HEAD~{}", commits.len()),
    };
    println!("   • {} - Execute the rebase plan", rebase_cmd.cyan());

    Ok(())
}

/// Print the suggested plan for a set of real commits
fn show_plan(commits: &[CommitInfo], analysis: &CommitAnalysis) {
    println!("\n{}", "📋 Suggested Rebase Plan:".bright_green().bold());
    for (i, commit) in commits.iter().enumerate() {
        let action = suggest_rebase_action(commit, analysis);
        let action_color = match action.as_str() {
            "pick" => action.green(),
            "squash" => action.yellow(),
            "fixup" => action.blue(),
            "edit" => action.cyan(),
            "drop" => action.red(),
            _ => action.white(),
        };

        println!(
            "   {}. {} {} {}",
            (i + 1).to_string().dimmed(),
            action_color.bold(),
            commit.id.bright_yellow(),
            commit.message.white()
        );
    }
}

/// Analyze real commits for rebase planning
async fn analyze_commits_for_rebase(
    git_repo: &GitRepository,
    args: &crate::args::RebaseArgs,
) -> Result<()> {
    println!(
        "\n{}",
        "🔬 Commit Analysis for Rebase".bright_green().bold()
    );
    println!("{}", "═══════════════════════════════".white().dimmed());

    let target = args
        .target
        .clone()
        .or_else(|| determine_rebase_target(git_repo));

    let commits = get_commits_for_rebase(git_repo, target.as_deref(), args.count)?;

    if commits.is_empty() {
        println!("\n{}", "ℹ️  No commits found for analysis".yellow());
        return Ok(());
    }

    println!("\n{}", "📊 Commit Statistics:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Total commits:".bright_white(),
        commits.len().to_string().cyan()
    );

    // Analyze commit patterns
    let mut commit_types = HashMap::new();
    let mut large_commits = 0;
    let mut fixup_commits = 0;
    let mut merge_commits = 0;

    for commit in &commits {
        let commit_type = extract_commit_type(&commit.message);
        *commit_types.entry(commit_type).or_insert(0) += 1;

        if commit.files_changed > 10 {
            large_commits += 1;
        }

        if commit.message.starts_with("fixup!") || commit.message.starts_with("squash!") {
            fixup_commits += 1;
        }

        if commit.is_merge {
            merge_commits += 1;
        }
    }

    println!(
        "   {} {}",
        "Large commits (>10 files):".bright_white(),
        large_commits.to_string().yellow()
    );
    println!(
        "   {} {}",
        "Fixup/squash commits:".bright_white(),
        fixup_commits.to_string().green()
    );
    println!(
        "   {} {}",
        "Merge commits:".bright_white(),
        merge_commits.to_string().blue()
    );

    println!("\n{}", "📈 Commit Type Distribution:".bright_cyan().bold());
    for (commit_type, count) in &commit_types {
        let percentage = (*count as f64 / commits.len() as f64 * 100.0) as u32;
        println!(
            "   {} {}% ({})",
            commit_type.cyan(),
            percentage.to_string().bright_white(),
            count.to_string().dimmed()
        );
    }

    // Detailed commit list from real history
    println!("\n{}", "📝 Commit Details:".bright_cyan().bold());
    for (i, commit) in commits.iter().enumerate() {
        let commit_type_emoji = match extract_commit_type(&commit.message).as_str() {
            "feat" => "✨",
            "fix" => "🐛",
            "docs" => "📚",
            "style" => "💄",
            "refactor" => "♻️",
            "test" => "🧪",
            "chore" => "🔧",
            _ => "📝",
        };

        println!(
            "\n   {}. {} {} {}",
            (i + 1).to_string().dimmed(),
            commit_type_emoji,
            commit.id.bright_yellow(),
            commit.date.dimmed()
        );
        println!("      {}", commit.message.white());
        println!(
            "      {} {} files, {} insertions, {} deletions",
            "📁".dimmed(),
            commit.files_changed.to_string().cyan(),
            commit.insertions.to_string().green(),
            commit.deletions.to_string().red()
        );
    }

    Ok(())
}

// Helper functions

/// Detect a sensible rebase target from real refs (upstream main/master)
fn determine_rebase_target(git_repo: &GitRepository) -> Option<String> {
    let repo = git_repo.inner();
    let current_branch = git_repo.current_branch().ok();

    let candidates = ["origin/main", "origin/master", "main", "master"];
    for candidate in candidates {
        if Some(candidate) == current_branch.as_deref() {
            continue;
        }
        if repo.revparse_single(candidate).is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}

/// List the real commits that would be rebased
fn get_commits_for_rebase(
    git_repo: &GitRepository,
    target: Option<&str>,
    count: Option<usize>,
) -> Result<Vec<CommitInfo>> {
    let repo = git_repo.inner();

    let head = match repo.head().ok().and_then(|h| h.peel_to_commit().ok()) {
        Some(commit) => commit,
        None => return Ok(Vec::new()),
    };

    let mut walker = repo.revwalk()?;
    walker.push(head.id())?;

    if let Some(target) = target {
        if let Ok(target_commit) = repo
            .revparse_single(target)
            .and_then(|obj| obj.peel_to_commit())
        {
            if target_commit.id() != head.id() {
                let _ = walker.hide(target_commit.id());
            }
        }
    }

    let limit = count.unwrap_or(10);
    let mut commits = Vec::new();

    for oid in walker.take(limit) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Real per-commit diff statistics
        let tree = commit.tree()?;
        let parent_tree = match commit.parent(0) {
            Ok(parent) => Some(parent.tree()?),
            Err(_) => None,
        };
        let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
        let stats = diff.stats()?;

        commits.push(CommitInfo {
            id: commit.id().to_string().chars().take(7).collect(),
            message: commit.summary().unwrap_or("").to_string(),
            date: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
            is_merge: commit.parent_count() > 1,
        });
    }

    Ok(commits)
}

fn analyze_commits(commits: &[CommitInfo]) -> CommitAnalysis {
    let has_fixup_commits = commits
        .iter()
        .any(|c| c.message.starts_with("fixup!") || c.message.starts_with("squash!"));
    let has_large_commits = commits.iter().any(|c| c.files_changed > 10);
    let has_merge_commits = commits.iter().any(|c| c.is_merge);

    CommitAnalysis {
        has_fixup_commits,
        has_large_commits,
        has_merge_commits,
    }
}

fn show_rebase_analysis(analysis: &CommitAnalysis) {
    println!("\n{}", "📝 Analysis:".bright_cyan().bold());

    if analysis.has_fixup_commits {
        println!(
            "   • {} Fixup commits detected - recommend --autosquash",
            "✨".green()
        );
    }

    if analysis.has_large_commits {
        println!(
            "   • {} Large commits found - consider splitting for better history",
            "📝".yellow()
        );
    }

    if analysis.has_merge_commits {
        println!(
            "   • {} Merge commits present - may need --rebase-merges",
            "🔄".cyan()
        );
    }

    if !analysis.has_fixup_commits && !analysis.has_large_commits && !analysis.has_merge_commits {
        println!("   • {} No issues detected in these commits", "✅".green());
    }
}

fn suggest_rebase_action(commit: &CommitInfo, analysis: &CommitAnalysis) -> String {
    if commit.message.starts_with("fixup!") {
        "fixup".to_string()
    } else if commit.message.starts_with("squash!") {
        "squash".to_string()
    } else if commit.files_changed > 15 && analysis.has_large_commits {
        "edit".to_string()
    } else if commit.message.contains("WIP") || commit.message.contains("tmp") {
        "squash".to_string()
    } else {
        "pick".to_string()
    }
}

fn extract_commit_type(message: &str) -> String {
    if let Some(colon_pos) = message.find(':') {
        let prefix = &message[..colon_pos];
        if let Some(paren_pos) = prefix.find('(') {
            prefix[..paren_pos].to_string()
        } else {
            prefix.to_string()
        }
    } else {
        "other".to_string()
    }
}

// Data structures

#[derive(Debug)]
struct RebaseState {
    is_in_progress: bool,
    current_commit: Option<String>,
    current_step: usize,
    total_steps: usize,
    branch: String,
    conflicts: Vec<String>,
}

#[derive(Debug)]
struct CommitInfo {
    id: String,
    message: String,
    date: String,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
    is_merge: bool,
}

#[derive(Debug)]
struct CommitAnalysis {
    has_fixup_commits: bool,
    has_large_commits: bool,
    has_merge_commits: bool,
}
