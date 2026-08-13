use crate::args::TagFormat;
/// Git tag and release management
use crate::git::repository::GitRepository;
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input};
use regex::Regex;
use std::collections::BTreeSet;

/// Handle the tag management subcommand
pub async fn handle_tag_command(
    args: &crate::args::TagArgs,
    _repo: &SqliteRepository,
) -> Result<()> {
    println!(
        "{}",
        "🏷️  TermAI Git Tag & Release Management"
            .bright_blue()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════".white().dimmed()
    );

    // Discover and analyze the Git repository
    let git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;

    match args.action.as_str() {
        "list" => {
            list_tags(&git_repo).await?;
        }
        "create" => {
            create_tag(&git_repo, args).await?;
        }
        "delete" => {
            delete_tag(&git_repo, args).await?;
        }
        "show" => {
            show_tag(&git_repo, args).await?;
        }
        "release-notes" => {
            generate_release_notes(&git_repo, args).await?;
        }
        "suggest" => {
            suggest_next_tag(&git_repo).await?;
        }
        _ => {
            anyhow::bail!("Unknown tag action: {}. Use 'list', 'create', 'delete', 'show', 'release-notes', or 'suggest'", args.action);
        }
    }

    Ok(())
}

/// List all tags read from the actual repository
async fn list_tags(git_repo: &GitRepository) -> Result<()> {
    println!("\n{}", "📋 Git Tags".bright_green().bold());

    let tags = collect_tags(git_repo)?;

    if tags.is_empty() {
        println!("   {}", "No tags found".dimmed());
        println!(
            "\n{}",
            "💡 Create your first tag with: termai tag create".cyan()
        );
        return Ok(());
    }

    let head_id = git_repo
        .inner()
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.id());

    for tag in &tags {
        let type_indicator = match tag.tag_type {
            TagType::Annotated => "📝",
            TagType::Lightweight => "📌",
        };

        let head_info = if Some(tag.target) == head_id {
            "HEAD".bright_green().to_string()
        } else {
            String::new()
        };

        println!(
            "\n   {} {} {} {}",
            type_indicator.cyan(),
            tag.name.bright_yellow().bold(),
            short_id(&tag.target.to_string()).dimmed(),
            head_info
        );
        println!("      {} {}", tag.date.bright_blue(), tag.message.white());
    }

    println!("\n{}", "💡 Suggested Actions:".bright_green().bold());
    println!(
        "   • {} - Suggest the next version",
        "termai tag suggest".cyan()
    );
    println!(
        "   • {} - Generate release notes",
        "termai tag release-notes".cyan()
    );
    println!(
        "   • {} - Show detailed tag info",
        "termai tag show <tag>".cyan()
    );

    Ok(())
}

/// Create a new tag using git2
async fn create_tag(git_repo: &GitRepository, args: &crate::args::TagArgs) -> Result<()> {
    println!("\n{}", "🏷️  Creating Git Tag".bright_green().bold());

    // Get tag name
    let tag_name = if let Some(name) = &args.tag_name {
        name.clone()
    } else {
        let suggested_name = suggest_tag_name(git_repo)?;

        println!("\n{}", "💡 Suggested Tag Name:".bright_cyan().bold());
        println!("   {}", suggested_name.bright_yellow().bold());

        let input = Input::<String>::new();
        input
            .with_prompt("Enter tag name")
            .default(suggested_name)
            .interact_text()?
    };

    // Validate tag name
    if !is_valid_tag_name(&tag_name) {
        anyhow::bail!(
            "Invalid tag name: {}. Use semantic versioning (e.g., v1.2.3)",
            tag_name
        );
    }

    // Get tag message
    let tag_message = if let Some(message) = &args.message {
        message.clone()
    } else if !args.lightweight {
        let suggested_message = format!("Release {}", tag_name);

        println!("\n{}", "💭 Suggested Release Message:".bright_cyan().bold());
        println!("   {}", suggested_message.bright_white());

        let input = Input::<String>::new();
        input
            .with_prompt("Enter tag message (or press Enter to use suggested)")
            .default(suggested_message)
            .interact_text()?
    } else {
        String::new()
    };

    // Check if tag already exists
    if tag_exists(git_repo, &tag_name) && !args.force {
        println!("\n{}", "⚠️  Warning:".bright_yellow().bold());
        println!("   Tag '{}' already exists", tag_name.bright_yellow());

        if !args.yes
            && !Confirm::new()
                .with_prompt("Overwrite existing tag?")
                .default(false)
                .interact()?
        {
            println!("{}", "Tag creation cancelled".yellow());
            return Ok(());
        }
    }

    // Show what will be tagged
    let repo = git_repo.inner();
    let head_commit = repo
        .head()
        .context("Repository has no HEAD to tag")?
        .peel_to_commit()
        .context("Failed to resolve HEAD commit")?;

    println!("\n{}", "📊 Tag Summary:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Tag name:".bright_white(),
        tag_name.bright_yellow()
    );
    println!(
        "   {} {}",
        "Type:".bright_white(),
        if args.lightweight {
            "Lightweight"
        } else {
            "Annotated"
        }
        .cyan()
    );
    println!(
        "   {} {} ({})",
        "Target:".bright_white(),
        short_id(&head_commit.id().to_string()).bright_blue(),
        head_commit.summary().unwrap_or("").white()
    );

    if !tag_message.is_empty() {
        println!("   {} {}", "Message:".bright_white(), tag_message.white());
    }

    // Confirm creation (skipped with --yes for non-interactive use)
    if !args.yes
        && !Confirm::new()
            .with_prompt("Create this tag?")
            .default(true)
            .interact()?
    {
        println!("{}", "Tag creation cancelled".yellow());
        return Ok(());
    }

    // Create the tag for real
    println!("\n{}", "🔄 Creating tag...".cyan());
    let target = head_commit.as_object();
    if args.lightweight {
        repo.tag_lightweight(&tag_name, target, true)
            .with_context(|| format!("Failed to create lightweight tag '{}'", tag_name))?;
    } else {
        let signature = repo
            .signature()
            .context("Failed to determine tagger identity (set user.name/user.email)")?;
        repo.tag(&tag_name, target, &signature, &tag_message, true)
            .with_context(|| format!("Failed to create annotated tag '{}'", tag_name))?;
    }

    let tag_type = if args.lightweight {
        "lightweight"
    } else {
        "annotated"
    };
    println!(
        "   {} Tag '{}' created successfully ({})",
        "✅".green(),
        tag_name.bright_yellow(),
        tag_type.dimmed()
    );

    // Show next steps
    println!("\n{}", "💡 Next Steps:".bright_yellow().bold());
    println!("   • {} to push tags to remote", "git push --tags".cyan());
    println!(
        "   • {} to generate release notes",
        "termai tag release-notes".cyan()
    );
    println!("   • {} to see all tags", "termai tag list".cyan());

    Ok(())
}

/// Delete a tag with safety checks
async fn delete_tag(git_repo: &GitRepository, args: &crate::args::TagArgs) -> Result<()> {
    let tag_name = args
        .tag_name
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Tag name is required for delete operation"))?;

    println!("\n{}", "🗑️  Deleting Git Tag".bright_red().bold());

    if !tag_exists(git_repo, tag_name) {
        anyhow::bail!("Tag '{}' does not exist", tag_name);
    }

    // Safety warnings
    println!("\n{}", "⚠️  Warning:".bright_yellow().bold());
    println!("   This will delete tag '{}'", tag_name.bright_yellow());
    println!("   This action cannot be undone");
    println!("   If the tag is pushed to remote, you'll need to delete it there too");

    if !args.yes
        && !Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete tag '{}'?",
                tag_name
            ))
            .default(false)
            .interact()?
    {
        println!("{}", "Tag deletion cancelled".yellow());
        return Ok(());
    }

    // Delete the tag for real
    git_repo
        .inner()
        .tag_delete(tag_name)
        .with_context(|| format!("Failed to delete tag '{}'", tag_name))?;

    println!(
        "\n   {} Tag '{}' deleted successfully",
        "✅".green(),
        tag_name.bright_yellow()
    );

    println!("\n{}", "💡 Remember:".bright_yellow().bold());
    println!(
        "   • Use {} to delete from remote",
        format!("git push --delete origin {}", tag_name).cyan()
    );

    Ok(())
}

/// Show detailed information about a tag, read from the repository
async fn show_tag(git_repo: &GitRepository, args: &crate::args::TagArgs) -> Result<()> {
    let tag_name = args
        .tag_name
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Tag name is required for show operation"))?;

    println!(
        "\n{}",
        format!("🔍 Tag Details: {}", tag_name)
            .bright_green()
            .bold()
    );
    println!("{}", "═══════════════════════════════".white().dimmed());

    let repo = git_repo.inner();
    let obj = repo
        .revparse_single(&format!("refs/tags/{}", tag_name))
        .with_context(|| format!("Tag '{}' not found", tag_name))?;

    println!("\n{}", "📋 Tag Information:".bright_cyan().bold());
    println!("   {} {}", "Name:".bright_white(), tag_name);

    if let Some(tag) = obj.as_tag() {
        let commit = tag.target()?.peel_to_commit()?;
        println!("   {} Annotated", "Type:".bright_white());
        println!("   {} {}", "Commit:".bright_white(), commit.id());
        if let Some(tagger) = tag.tagger() {
            println!(
                "   {} {}",
                "Date:".bright_white(),
                format_git_time(tagger.when())
            );
            println!(
                "   {} {} <{}>",
                "Tagger:".bright_white(),
                tagger.name().unwrap_or("unknown"),
                tagger.email().unwrap_or("unknown")
            );
        }

        let message = tag.message().unwrap_or("").trim();
        if !message.is_empty() {
            println!("\n{}", "💭 Tag Message:".bright_cyan().bold());
            for line in message.lines() {
                println!("   {}", line);
            }
        }

        println!("\n{}", "📝 Tagged Commit:".bright_cyan().bold());
        println!("   {}", commit.summary().unwrap_or(""));
    } else {
        let commit = obj.peel_to_commit()?;
        println!("   {} Lightweight", "Type:".bright_white());
        println!("   {} {}", "Commit:".bright_white(), commit.id());
        println!(
            "   {} {}",
            "Date:".bright_white(),
            format_git_time(commit.time())
        );

        println!("\n{}", "📝 Tagged Commit:".bright_cyan().bold());
        println!("   {}", commit.summary().unwrap_or(""));
    }

    Ok(())
}

/// Generate release notes from the actual commit history between two revs
async fn generate_release_notes(
    git_repo: &GitRepository,
    args: &crate::args::TagArgs,
) -> Result<()> {
    let latest_tag = collect_tags(git_repo)?.first().map(|t| t.name.clone());

    let from_rev = args.from_tag.clone().or(latest_tag).unwrap_or_default();
    let to_rev = args.to_tag.as_deref().unwrap_or("HEAD").to_string();

    println!("\n{}", "📝 Generating Release Notes".bright_green().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());

    let range_label = if from_rev.is_empty() {
        format!("(start) → {}", to_rev)
    } else {
        format!("{} → {}", from_rev, to_rev)
    };
    println!(
        "\n{}",
        format!("📊 Analyzing changes: {}", range_label).bright_blue()
    );

    // Analyze commits and categorize changes for real
    let release_data = analyze_release_changes(git_repo, &from_rev, &to_rev)?;

    if release_data.stats.commits == 0 {
        println!("\n   {}", "No commits found in the given range".yellow());
        return Ok(());
    }

    // Generate release notes based on format
    match args.format {
        TagFormat::Markdown => generate_markdown_release_notes(&release_data).await?,
        TagFormat::Text => generate_text_release_notes(&release_data).await?,
        TagFormat::Json => generate_json_release_notes(&release_data).await?,
    }

    Ok(())
}

/// Suggest the next appropriate tag name based on the real commit history
async fn suggest_next_tag(git_repo: &GitRepository) -> Result<()> {
    println!("\n{}", "🎯 Tag Suggestion".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());

    println!("\n{}", "🔍 Analyzing recent changes...".cyan());

    let analysis = analyze_changes_since_last_tag(git_repo)?;

    println!("\n{}", "📊 Change Analysis:".bright_cyan().bold());
    println!(
        "   {} {}",
        "Current version:".bright_white(),
        analysis
            .current_version
            .as_deref()
            .unwrap_or("(no tags yet)")
            .bright_yellow()
    );
    println!(
        "   {} {}",
        "Commits since last tag:".bright_white(),
        analysis.commits_since_last.to_string().cyan()
    );
    println!(
        "   {} {}",
        "Breaking changes:".bright_white(),
        if analysis.breaking_changes {
            "Yes".red()
        } else {
            "No".green()
        }
    );
    println!(
        "   {} {}",
        "New features:".bright_white(),
        if !analysis.features.is_empty() {
            "Yes".green()
        } else {
            "No".dimmed()
        }
    );
    println!(
        "   {} {}",
        "Bug fixes:".bright_white(),
        if !analysis.fixes.is_empty() {
            "Yes".yellow()
        } else {
            "No".dimmed()
        }
    );

    if analysis.commits_since_last == 0 {
        println!(
            "\n   {} No new commits since the last tag - nothing to release",
            "ℹ️".cyan()
        );
        return Ok(());
    }

    let suggested_version = suggest_version_bump(&analysis);

    println!("\n{}", "🎯 Recommendation:".bright_green().bold());
    println!(
        "   {} {}",
        "Suggested tag:".bright_white(),
        suggested_version.bright_yellow().bold()
    );

    let rationale = if analysis.breaking_changes {
        "Major version bump due to breaking changes"
    } else if !analysis.features.is_empty() {
        "Minor version bump due to new features"
    } else if !analysis.fixes.is_empty() {
        "Patch version bump for bug fixes"
    } else {
        "Patch version bump for miscellaneous changes"
    };

    println!("   {} {}", "Rationale:".bright_white(), rationale.cyan());

    // Show what's included, from real commit subjects
    if !analysis.features.is_empty() {
        println!("\n{}", "✨ New Features:".bright_green().bold());
        for feature in &analysis.features {
            println!("   • {}", feature);
        }
    }

    if !analysis.fixes.is_empty() {
        println!("\n{}", "🐛 Bug Fixes:".bright_yellow().bold());
        for fix in &analysis.fixes {
            println!("   • {}", fix);
        }
    }

    println!("\n{}", "💡 Next Steps:".bright_cyan().bold());
    println!(
        "   • {} - Create the suggested tag",
        format!("termai tag create {}", suggested_version).cyan()
    );
    println!(
        "   • {} - Generate release notes",
        "termai tag release-notes".cyan()
    );
    println!("   • {} - Push tag to remote", "git push --tags".cyan());

    Ok(())
}

// Helper functions

/// Collect real tags from the repository, newest first (by target commit time)
fn collect_tags(git_repo: &GitRepository) -> Result<Vec<TagInfo>> {
    let repo = git_repo.inner();
    let mut tags = Vec::new();

    let names = repo.tag_names(None)?;
    for name in names.iter().flatten() {
        let obj = match repo.revparse_single(&format!("refs/tags/{}", name)) {
            Ok(obj) => obj,
            Err(_) => continue,
        };

        if let Some(tag) = obj.as_tag() {
            let commit = tag.target()?.peel_to_commit()?;
            let time = tag
                .tagger()
                .map(|t| t.when())
                .unwrap_or_else(|| commit.time());
            tags.push(TagInfo {
                name: name.to_string(),
                target: commit.id(),
                date: format_git_time(time),
                sort_key: time.seconds(),
                message: tag.message().unwrap_or("").trim().to_string(),
                tag_type: TagType::Annotated,
            });
        } else if let Ok(commit) = obj.peel_to_commit() {
            tags.push(TagInfo {
                name: name.to_string(),
                target: commit.id(),
                date: format_git_time(commit.time()),
                sort_key: commit.time().seconds(),
                message: commit.summary().unwrap_or("").to_string(),
                tag_type: TagType::Lightweight,
            });
        }
    }

    tags.sort_by_key(|b| std::cmp::Reverse(b.sort_key));
    Ok(tags)
}

/// Suggest a tag name based on the real repository history
fn suggest_tag_name(git_repo: &GitRepository) -> Result<String> {
    let analysis = analyze_changes_since_last_tag(git_repo)?;
    Ok(suggest_version_bump(&analysis))
}

/// Analyze the real commits since the most recent tag
fn analyze_changes_since_last_tag(git_repo: &GitRepository) -> Result<ChangeAnalysis> {
    let repo = git_repo.inner();
    let tags = collect_tags(git_repo)?;
    let latest_tag = tags.first();

    let mut analysis = ChangeAnalysis {
        current_version: latest_tag.map(|t| t.name.clone()),
        commits_since_last: 0,
        breaking_changes: false,
        features: Vec::new(),
        fixes: Vec::new(),
    };

    let head = match repo.head().ok().and_then(|h| h.peel_to_commit().ok()) {
        Some(commit) => commit,
        None => return Ok(analysis),
    };

    let mut walker = repo.revwalk()?;
    walker.push(head.id())?;
    if let Some(tag) = latest_tag {
        let _ = walker.hide(tag.target);
    }

    for oid in walker.take(500) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let subject = commit.summary().unwrap_or("").to_string();
        let message = commit.message().unwrap_or("");

        analysis.commits_since_last += 1;

        let prefix = subject.split(':').next().unwrap_or("");
        if prefix.contains('!') || message.contains("BREAKING CHANGE") {
            analysis.breaking_changes = true;
        }
        if prefix.starts_with("feat") {
            analysis.features.push(subject.clone());
        } else if prefix.starts_with("fix") {
            analysis.fixes.push(subject.clone());
        }
    }

    Ok(analysis)
}

/// Compute the suggested next semantic version from a real change analysis
fn suggest_version_bump(analysis: &ChangeAnalysis) -> String {
    let (major, minor, patch) = analysis
        .current_version
        .as_deref()
        .and_then(parse_semver)
        .unwrap_or((0, 0, 0));

    if analysis.breaking_changes {
        format!("v{}.0.0", major + 1)
    } else if !analysis.features.is_empty() {
        format!("v{}.{}.0", major, minor + 1)
    } else {
        format!("v{}.{}.{}", major, minor, patch + 1)
    }
}

/// Parse a version string like "v1.2.3" or "1.2.3"
fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    let version = version.trim_start_matches('v');
    let core = version.split('-').next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn is_valid_tag_name(name: &str) -> bool {
    // Basic semantic version validation
    let version_regex = Regex::new(r"^v?\d+\.\d+\.\d+(-[a-zA-Z0-9.-]+)?$").unwrap();
    version_regex.is_match(name)
}

/// Check whether a tag actually exists in the repository
fn tag_exists(git_repo: &GitRepository, name: &str) -> bool {
    git_repo
        .inner()
        .find_reference(&format!("refs/tags/{}", name))
        .is_ok()
}

fn short_id(id: &str) -> String {
    id.chars().take(7).collect()
}

fn format_git_time(time: git2::Time) -> String {
    chrono::DateTime::from_timestamp(time.seconds(), 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Analyze the real commit history between two revisions
fn analyze_release_changes(git_repo: &GitRepository, from: &str, to: &str) -> Result<ReleaseData> {
    let repo = git_repo.inner();

    let to_commit = repo
        .revparse_single(to)
        .with_context(|| format!("Cannot resolve revision '{}'", to))?
        .peel_to_commit()?;

    let from_commit = if from.is_empty() {
        None
    } else {
        Some(
            repo.revparse_single(from)
                .with_context(|| format!("Cannot resolve revision '{}'", from))?
                .peel_to_commit()?,
        )
    };

    let mut walker = repo.revwalk()?;
    walker.push(to_commit.id())?;
    if let Some(from_commit) = &from_commit {
        walker.hide(from_commit.id())?;
    }

    let mut features = Vec::new();
    let mut fixes = Vec::new();
    let mut breaking_changes = Vec::new();
    let mut contributors = BTreeSet::new();
    let mut commit_count = 0usize;

    for oid in walker.take(500) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let subject = commit.summary().unwrap_or("").to_string();
        let message = commit.message().unwrap_or("");

        commit_count += 1;
        if let Some(author) = commit.author().name() {
            contributors.insert(author.to_string());
        }

        let prefix = subject.split(':').next().unwrap_or("");
        if prefix.contains('!') || message.contains("BREAKING CHANGE") {
            breaking_changes.push(subject.clone());
        }
        if prefix.starts_with("feat") {
            features.push(subject.clone());
        } else if prefix.starts_with("fix") {
            fixes.push(subject.clone());
        }
    }

    // Real diff statistics between the two trees
    let from_tree = from_commit.as_ref().map(|c| c.tree()).transpose()?;
    let diff = repo.diff_tree_to_tree(from_tree.as_ref(), Some(&to_commit.tree()?), None)?;
    let stats = diff.stats()?;

    Ok(ReleaseData {
        version: if to == "HEAD" {
            "unreleased".to_string()
        } else {
            to.to_string()
        },
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        features,
        fixes,
        breaking_changes,
        contributors: contributors.into_iter().collect(),
        stats: ReleaseStats {
            commits: commit_count,
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        },
    })
}

async fn generate_markdown_release_notes(data: &ReleaseData) -> Result<()> {
    println!("\n{}", "📝 Release Notes (Markdown):".bright_green().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());

    println!("\n# Release {}", data.version);
    println!("*Generated on {}*", data.date);

    if !data.breaking_changes.is_empty() {
        println!("\n## ⚠️  Breaking Changes");
        for change in &data.breaking_changes {
            println!("- {}", change);
        }
    }

    if !data.features.is_empty() {
        println!("\n## ✨ New Features");
        for feature in &data.features {
            println!("- {}", feature);
        }
    }

    if !data.fixes.is_empty() {
        println!("\n## 🐛 Bug Fixes");
        for fix in &data.fixes {
            println!("- {}", fix);
        }
    }

    if !data.contributors.is_empty() {
        println!("\n## 👥 Contributors");
        for contributor in &data.contributors {
            println!("- {}", contributor);
        }
    }

    println!("\n## 📊 Statistics");
    println!("- {} commits", data.stats.commits);
    println!("- {} files changed", data.stats.files_changed);
    println!("- {} insertions(+)", data.stats.insertions);
    println!("- {} deletions(-)", data.stats.deletions);

    Ok(())
}

async fn generate_text_release_notes(data: &ReleaseData) -> Result<()> {
    println!("\n{}", "📝 Release Notes (Text):".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());

    println!("\nRelease {} - {}", data.version, data.date);
    println!("{}", "=".repeat(50));

    if !data.features.is_empty() {
        println!("\nNEW FEATURES:");
        for feature in &data.features {
            println!("  * {}", feature);
        }
    }

    if !data.fixes.is_empty() {
        println!("\nBUG FIXES:");
        for fix in &data.fixes {
            println!("  * {}", fix);
        }
    }

    println!("\nSTATISTICS:");
    println!("  Commits: {}", data.stats.commits);
    println!("  Files changed: {}", data.stats.files_changed);
    println!("  Lines added: {}", data.stats.insertions);
    println!("  Lines removed: {}", data.stats.deletions);

    Ok(())
}

async fn generate_json_release_notes(data: &ReleaseData) -> Result<()> {
    println!("\n{}", "📝 Release Notes (JSON):".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());

    println!("\n{{");
    println!("  \"version\": \"{}\",", data.version);
    println!("  \"date\": \"{}\",", data.date);
    println!("  \"features\": [");
    for (i, feature) in data.features.iter().enumerate() {
        let comma = if i < data.features.len() - 1 { "," } else { "" };
        println!("    \"{}\"{}", feature.replace('"', "\\\""), comma);
    }
    println!("  ],");
    println!("  \"fixes\": [");
    for (i, fix) in data.fixes.iter().enumerate() {
        let comma = if i < data.fixes.len() - 1 { "," } else { "" };
        println!("    \"{}\"{}", fix.replace('"', "\\\""), comma);
    }
    println!("  ]");
    println!("}}");

    Ok(())
}

// Data structures
#[derive(Debug, Clone)]
struct TagInfo {
    name: String,
    target: git2::Oid,
    date: String,
    sort_key: i64,
    message: String,
    tag_type: TagType,
}

#[derive(Debug, Clone)]
enum TagType {
    Annotated,
    Lightweight,
}

#[derive(Debug)]
struct ChangeAnalysis {
    current_version: Option<String>,
    commits_since_last: usize,
    breaking_changes: bool,
    features: Vec<String>,
    fixes: Vec<String>,
}

#[derive(Debug)]
struct ReleaseData {
    version: String,
    date: String,
    features: Vec<String>,
    fixes: Vec<String>,
    breaking_changes: Vec<String>,
    contributors: Vec<String>,
    stats: ReleaseStats,
}

#[derive(Debug)]
struct ReleaseStats {
    commits: usize,
    files_changed: usize,
    insertions: usize,
    deletions: usize,
}
