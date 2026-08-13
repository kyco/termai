/// Git conflict resolution assistance
use crate::git::repository::GitRepository;
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Select};
use std::collections::HashMap;
use std::path::Path;

/// Handle conflict resolution commands
pub async fn handle_conflicts_command(
    args: &crate::args::ConflictsArgs,
    _repo: &SqliteRepository,
) -> Result<()> {
    println!(
        "{}",
        "⚔️ TermAI Conflict Resolution Assistant"
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
        "detect" => {
            detect_conflicts(&git_repo).await?;
        }
        "analyze" => {
            analyze_conflicts(&git_repo).await?;
        }
        "suggest" => {
            suggest_resolution_strategies(&git_repo).await?;
        }
        "resolve" => {
            interactive_conflict_resolution(&git_repo).await?;
        }
        "status" => {
            show_conflict_status(&git_repo).await?;
        }
        "guide" => {
            show_resolution_guide().await?;
        }
        _ => {
            anyhow::bail!("Unknown conflicts action: {}. Use 'detect', 'analyze', 'suggest', 'resolve', 'status', or 'guide'", args.action);
        }
    }

    Ok(())
}

/// Detect and list all real conflicts in the repository
async fn detect_conflicts(git_repo: &GitRepository) -> Result<()> {
    println!("\n{}", "🔍 Detecting Merge Conflicts".bright_green().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());

    let conflicts = detect_conflicted_files(git_repo)?;

    if conflicts.is_empty() {
        println!("\n   {} No merge conflicts detected", "✅".green());
        println!("   {} Repository is in a clean state", "🎉".green());
        return Ok(());
    }

    let total_markers: usize = conflicts.values().map(|c| c.markers.len().max(1)).sum();
    println!(
        "\n{}",
        format!(
            "⚔️  {} conflicts detected in {} file(s)",
            total_markers,
            conflicts.len()
        )
        .bright_red()
        .bold()
    );

    for (file, conflict_info) in &conflicts {
        println!("\n   {} {}", "📁".red(), file.bright_white());

        if conflict_info.markers.is_empty() {
            println!(
                "      {} conflicted in index (no markers found in working tree)",
                "⚔️".yellow()
            );
            continue;
        }

        println!(
            "      {} {} conflict marker(s)",
            "⚔️".yellow(),
            conflict_info.markers.len()
        );

        for marker in &conflict_info.markers {
            println!(
                "        • Line {}: {} vs {}",
                marker.line_number.to_string().cyan(),
                marker.our_label.bright_green(),
                marker.their_label.bright_red()
            );
        }
    }

    // Show quick resolution options
    show_quick_resolution_options().await?;

    Ok(())
}

/// Analyze real conflicts with rule-based insights
async fn analyze_conflicts(git_repo: &GitRepository) -> Result<()> {
    println!("\n{}", "🔎 Conflict Analysis".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());

    let conflicts = detect_conflicted_files(git_repo)?;

    if conflicts.is_empty() {
        println!("\n   {} No conflicts to analyze", "ℹ️".cyan());
        return Ok(());
    }

    for (file, conflict_info) in &conflicts {
        println!(
            "\n{}",
            format!("📊 Analysis: {}", file).bright_cyan().bold()
        );
        println!("{}", "─────────────────────────".white().dimmed());

        let analysis = analyze_file_conflicts(file, conflict_info);

        println!(
            "   {} {}",
            "Conflict type:".bright_white(),
            analysis.conflict_type.bright_yellow()
        );
        println!(
            "   {} {}",
            "Complexity:".bright_white(),
            format_complexity(&analysis.complexity)
        );

        if !analysis.recommendations.is_empty() {
            println!("\n   {}", "Recommendations:".bright_cyan().bold());
            for (i, rec) in analysis.recommendations.iter().enumerate() {
                println!("      {}. {}", (i + 1).to_string().bright_yellow(), rec);
            }
        }
    }

    Ok(())
}

/// Suggest resolution strategies for the real conflicts
async fn suggest_resolution_strategies(git_repo: &GitRepository) -> Result<()> {
    println!(
        "\n{}",
        "💡 Resolution Strategy Suggestions".bright_green().bold()
    );
    println!("{}", "═════════════════════════════════".white().dimmed());

    let conflicts = detect_conflicted_files(git_repo)?;

    if conflicts.is_empty() {
        println!(
            "\n   {} No conflicts need resolution strategies",
            "ℹ️".cyan()
        );
        return Ok(());
    }

    // File-specific strategies based on real conflict data
    for (file, conflict_info) in &conflicts {
        let strategy = generate_file_strategy(file, conflict_info);

        println!(
            "\n{}",
            format!("📋 Strategy: {}", file).bright_cyan().bold()
        );
        println!(
            "   {} {}",
            "Method:".bright_white(),
            strategy.method.bright_green()
        );
        println!(
            "   {} {}",
            "Tools:".bright_white(),
            strategy.recommended_tools.join(", ").cyan()
        );

        if !strategy.steps.is_empty() {
            println!("\n   {}", "Steps:".bright_yellow().bold());
            for (i, step) in strategy.steps.iter().enumerate() {
                println!("      {}. {}", (i + 1).to_string().bright_yellow(), step);
            }
        }

        if !strategy.gotchas.is_empty() {
            println!("\n   {}", "Watch out for:".bright_red().bold());
            for gotcha in &strategy.gotchas {
                println!("      • {}", gotcha.yellow());
            }
        }
    }

    Ok(())
}

/// Interactive conflict resolution wizard operating on real conflicts
async fn interactive_conflict_resolution(git_repo: &GitRepository) -> Result<()> {
    println!(
        "\n{}",
        "🧙 Interactive Resolution Wizard".bright_green().bold()
    );
    println!("{}", "═════════════════════════════════".white().dimmed());

    let conflicts = detect_conflicted_files(git_repo)?;

    if conflicts.is_empty() {
        println!("\n   {} No conflicts to resolve", "ℹ️".cyan());
        return Ok(());
    }

    println!(
        "\n{}",
        format!("Found {} conflicted files", conflicts.len()).bright_yellow()
    );

    for (file, conflict_info) in &conflicts {
        println!(
            "\n{}",
            format!("🔧 Resolving: {}", file).bright_cyan().bold()
        );

        // Show conflict preview from real content
        show_conflict_preview(conflict_info);

        // Get user choice for resolution method
        let resolution_methods = vec![
            "Accept ours (current branch)",
            "Accept theirs (incoming changes)",
            "Manual merge with editor",
            "Skip this file for now",
        ];

        let selection = Select::new()
            .with_prompt("How would you like to resolve this conflict?")
            .items(&resolution_methods)
            .default(2)
            .interact()?;

        match selection {
            0 => resolve_with_side(git_repo, file, ConflictSide::Ours)?,
            1 => resolve_with_side(git_repo, file, ConflictSide::Theirs)?,
            2 => {
                println!(
                    "   {} Edit {} in your editor, resolve the markers, then save",
                    "📝".blue(),
                    file
                );
            }
            3 => {
                println!("   {} Skipped {}", "⏭️".yellow(), file);
                continue;
            }
            _ => unreachable!(),
        }

        // Confirm resolution
        if Confirm::new()
            .with_prompt("Mark this file as resolved?")
            .default(true)
            .interact()?
        {
            stage_resolved_file(git_repo, file)?;
            println!("   {} {} marked as resolved", "✅".green(), file);
        }
    }

    // Final steps
    show_final_resolution_steps().await?;

    Ok(())
}

/// Show current conflict status based on the real repository state
async fn show_conflict_status(git_repo: &GitRepository) -> Result<()> {
    println!("\n{}", "📊 Conflict Status".bright_green().bold());
    println!("{}", "═══════════════════".white().dimmed());

    let conflicts = detect_conflicted_files(git_repo)?;

    if git_repo.is_merging() {
        println!("\n{}", "🔄 Merge in progress".bright_cyan().bold());
        println!(
            "   {} {}",
            "Current branch:".bright_white(),
            git_repo
                .current_branch()
                .unwrap_or_else(|_| "unknown".to_string())
                .bright_blue()
        );
    } else if git_repo.is_rebasing() {
        println!("\n{}", "🔄 Rebase in progress".bright_cyan().bold());
    } else {
        println!("\n   {} No merge operation in progress", "ℹ️".cyan());
    }

    if conflicts.is_empty() {
        println!("\n   {} No conflicted files", "✅".green());
        return Ok(());
    }

    println!("\n{}", "⚠️  Conflicted Files:".bright_red().bold());
    for file in conflicts.keys() {
        println!("   • {}", file.red());
    }

    println!("\n{}", "💡 Next Steps:".bright_yellow().bold());
    println!(
        "   • {} - Detect and analyze conflicts",
        "termai conflicts detect".cyan()
    );
    println!(
        "   • {} - Get resolution suggestions",
        "termai conflicts suggest".cyan()
    );
    println!(
        "   • {} - Interactive resolution wizard",
        "termai conflicts resolve".cyan()
    );

    Ok(())
}

/// Show comprehensive resolution guide
async fn show_resolution_guide() -> Result<()> {
    println!("\n{}", "📚 Conflict Resolution Guide".bright_green().bold());
    println!("{}", "═════════════════════════════".white().dimmed());

    println!(
        "\n{}",
        "🔍 Understanding Conflict Markers:".bright_cyan().bold()
    );
    println!(
        "   {} Marks the start of your changes",
        "<<<<<<< HEAD".green()
    );
    println!(
        "   {} Separates your changes from theirs",
        "=======".yellow()
    );
    println!(
        "   {} Marks the end of their changes",
        ">>>>>>> branch-name".red()
    );

    println!("\n{}", "🛠️  Resolution Strategies:".bright_cyan().bold());

    println!(
        "\n   {}",
        "Accept Ours (Keep Current)".bright_green().bold()
    );
    println!("      • When your changes are correct");
    println!("      • Use: git checkout --ours <file>");

    println!(
        "\n   {}",
        "Accept Theirs (Take Incoming)".bright_red().bold()
    );
    println!("      • When incoming changes are better");
    println!("      • Use: git checkout --theirs <file>");

    println!("\n   {}", "Manual Merge".bright_blue().bold());
    println!("      • When both changes are needed");
    println!("      • Edit file to combine changes");
    println!("      • Remove conflict markers");

    println!("\n{}", "🔧 Recommended Tools:".bright_cyan().bold());
    println!("   • {} - Built-in merge tool", "git mergetool".cyan());
    println!("   • {} - VS Code with GitLens", "code --merge".cyan());
    println!("   • {} - Vim with fugitive", "vim -d".cyan());
    println!("   • {} - Beyond Compare, P4Merge", "External tools".cyan());

    println!("\n{}", "⚡ Quick Commands:".bright_cyan().bold());
    println!(
        "   • {} - See all conflicts",
        "termai conflicts detect".green()
    );
    println!("   • {} - Get analysis", "termai conflicts analyze".green());
    println!(
        "   • {} - Interactive resolution",
        "termai conflicts resolve".green()
    );
    println!(
        "   • {} - Check progress",
        "termai conflicts status".green()
    );

    println!("\n{}", "⚠️  Common Pitfalls:".bright_yellow().bold());
    println!("   • Don't forget to remove conflict markers");
    println!("   • Test your changes after resolving");
    println!("   • Stage resolved files with git add");
    println!("   • Commit the merge when all conflicts are resolved");

    Ok(())
}

// Helper functions

/// Detect the real conflicted files from the git index and working tree
fn detect_conflicted_files(git_repo: &GitRepository) -> Result<HashMap<String, ConflictInfo>> {
    let repo = git_repo.inner();
    let mut conflicts = HashMap::new();

    let index = repo.index().context("Failed to read repository index")?;
    if !index.has_conflicts() {
        return Ok(conflicts);
    }

    for conflict in index.conflicts()? {
        let conflict = conflict?;

        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string());

        let Some(path) = path else { continue };

        if conflicts.contains_key(&path) {
            continue;
        }

        // Parse the real conflict markers from the working tree file
        let markers = git_repo
            .root_path()
            .join(&path)
            .to_str()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|content| parse_conflict_markers(&content))
            .unwrap_or_default();

        conflicts.insert(path, ConflictInfo { markers });
    }

    Ok(conflicts)
}

/// Parse actual conflict markers from file content
fn parse_conflict_markers(content: &str) -> Vec<ConflictMarker> {
    let mut markers = Vec::new();
    let mut lines = content.lines().enumerate().peekable();

    while let Some((idx, line)) = lines.next() {
        if !line.starts_with("<<<<<<<") {
            continue;
        }

        let our_label = line.trim_start_matches('<').trim().to_string();
        let start_line = idx + 1; // 1-based
        let mut our_content = Vec::new();
        let mut their_content = Vec::new();
        let mut their_label = String::new();
        let mut in_theirs = false;
        let mut closed = false;

        for (_, inner) in lines.by_ref() {
            if inner.starts_with("=======") && !in_theirs {
                in_theirs = true;
            } else if inner.starts_with(">>>>>>>") {
                their_label = inner.trim_start_matches('>').trim().to_string();
                closed = true;
                break;
            } else if in_theirs {
                their_content.push(inner.to_string());
            } else {
                our_content.push(inner.to_string());
            }
        }

        if closed {
            markers.push(ConflictMarker {
                line_number: start_line,
                our_label: if our_label.is_empty() {
                    "ours".to_string()
                } else {
                    our_label
                },
                their_label: if their_label.is_empty() {
                    "theirs".to_string()
                } else {
                    their_label
                },
                our_content: our_content.join("\n"),
                their_content: their_content.join("\n"),
            });
        }
    }

    markers
}

/// Rule-based analysis of a real conflict
fn analyze_file_conflicts(file: &str, conflict_info: &ConflictInfo) -> ConflictAnalysis {
    let conflict_type = if file.ends_with(".rs") {
        "Code conflict in Rust file".to_string()
    } else if file.ends_with(".yaml") || file.ends_with(".yml") {
        "Configuration conflict".to_string()
    } else {
        "General file conflict".to_string()
    };

    let complexity = if conflict_info.markers.len() > 3 {
        ConflictComplexity::High
    } else if conflict_info.markers.len() > 1 {
        ConflictComplexity::Medium
    } else {
        ConflictComplexity::Low
    };

    let recommendations = match complexity {
        ConflictComplexity::Low => vec![
            "Simple conflict - manual resolution recommended".to_string(),
            "Review both changes and combine if possible".to_string(),
        ],
        ConflictComplexity::Medium => vec![
            "Multiple conflicts detected - resolve systematically".to_string(),
            "Consider using a visual merge tool".to_string(),
            "Test changes after each resolution".to_string(),
        ],
        ConflictComplexity::High => vec![
            "Complex conflicts - take extra care".to_string(),
            "Consider pair programming for resolution".to_string(),
            "Use comprehensive testing after resolution".to_string(),
            "Document resolution decisions".to_string(),
        ],
    };

    ConflictAnalysis {
        conflict_type,
        complexity,
        recommendations,
    }
}

async fn show_quick_resolution_options() -> Result<()> {
    println!(
        "\n{}",
        "🚀 Quick Resolution Options:".bright_yellow().bold()
    );
    println!(
        "   • {} - Get conflict analysis",
        "termai conflicts analyze".cyan()
    );
    println!(
        "   • {} - Get resolution strategies",
        "termai conflicts suggest".cyan()
    );
    println!(
        "   • {} - Interactive resolution wizard",
        "termai conflicts resolve".cyan()
    );
    println!("   • {} - Open merge tool", "git mergetool".cyan());

    Ok(())
}

fn generate_file_strategy(file: &str, conflict_info: &ConflictInfo) -> FileStrategy {
    let method = if conflict_info.markers.len() <= 1 {
        "Direct resolution".to_string()
    } else {
        "Multi-step resolution".to_string()
    };

    let recommended_tools = if file.ends_with(".rs") {
        vec![
            "rust-analyzer".to_string(),
            "VS Code".to_string(),
            "vim".to_string(),
        ]
    } else if file.contains("config") {
        vec!["YAML validator".to_string(), "text editor".to_string()]
    } else {
        vec!["git mergetool".to_string(), "text editor".to_string()]
    };

    let steps = vec![
        "Open file in preferred editor".to_string(),
        "Locate conflict markers".to_string(),
        "Analyze both versions of the code".to_string(),
        "Choose appropriate resolution strategy".to_string(),
        "Remove conflict markers".to_string(),
        "Test the changes".to_string(),
    ];

    let gotchas = if file.ends_with(".rs") {
        vec![
            "Check syntax after resolution".to_string(),
            "Run cargo check".to_string(),
        ]
    } else {
        vec!["Validate file syntax after resolution".to_string()]
    };

    FileStrategy {
        method,
        recommended_tools,
        steps,
        gotchas,
    }
}

fn show_conflict_preview(conflict_info: &ConflictInfo) {
    println!("\n{}", "🔍 Conflict Preview:".bright_cyan().bold());

    if conflict_info.markers.is_empty() {
        println!(
            "   {} Conflicted in index (no markers found in working tree)",
            "⚔️".yellow()
        );
        return;
    }

    for (i, marker) in conflict_info.markers.iter().enumerate() {
        println!(
            "\n   {} Conflict {} (Line {})",
            "⚔️".yellow(),
            (i + 1).to_string().bright_yellow(),
            marker.line_number.to_string().cyan()
        );

        println!(
            "   {} {}",
            "Ours:".green().bold(),
            preview_content(&marker.our_content).bright_white()
        );
        println!(
            "   {} {}",
            "Theirs:".red().bold(),
            preview_content(&marker.their_content).bright_white()
        );
    }
}

fn preview_content(content: &str) -> String {
    let first_line = content.lines().next().unwrap_or("").to_string();
    if content.lines().count() > 1 {
        format!("{} …", first_line)
    } else {
        first_line
    }
}

#[derive(Debug, Clone, Copy)]
enum ConflictSide {
    Ours,
    Theirs,
}

/// Really resolve a conflict by writing the chosen side's blob to the working tree
fn resolve_with_side(git_repo: &GitRepository, file: &str, side: ConflictSide) -> Result<()> {
    let repo = git_repo.inner();
    let index = repo.index()?;

    let mut chosen_blob = None;
    let mut found = false;

    for conflict in index.conflicts()? {
        let conflict = conflict?;
        let path = conflict
            .our
            .as_ref()
            .or(conflict.their.as_ref())
            .or(conflict.ancestor.as_ref())
            .map(|entry| String::from_utf8_lossy(&entry.path).to_string());

        if path.as_deref() != Some(file) {
            continue;
        }

        found = true;
        let entry = match side {
            ConflictSide::Ours => conflict.our,
            ConflictSide::Theirs => conflict.their,
        };
        chosen_blob = entry.map(|e| e.id);
        break;
    }

    if !found {
        anyhow::bail!("No conflict found for file '{}'", file);
    }

    let target_path = git_repo.root_path().join(file);
    match chosen_blob {
        Some(id) => {
            let blob = repo.find_blob(id)?;
            std::fs::write(&target_path, blob.content())
                .with_context(|| format!("Failed to write {}", file))?;
            let label = match side {
                ConflictSide::Ours => "our",
                ConflictSide::Theirs => "their",
            };
            println!("   {} Wrote {} version of {}", "✅".green(), label, file);
        }
        None => {
            // The chosen side deleted the file
            if target_path.exists() {
                std::fs::remove_file(&target_path)
                    .with_context(|| format!("Failed to remove {}", file))?;
            }
            println!(
                "   {} Chosen side deleted {} - removed from working tree",
                "🗑️".yellow(),
                file
            );
        }
    }

    Ok(())
}

/// Really stage a resolved file (clears its conflict entries)
fn stage_resolved_file(git_repo: &GitRepository, file: &str) -> Result<()> {
    let repo = git_repo.inner();
    let mut index = repo.index()?;

    let target_path = git_repo.root_path().join(file);
    if target_path.exists() {
        index
            .add_path(Path::new(file))
            .with_context(|| format!("Failed to stage {}", file))?;
    } else {
        index
            .remove_path(Path::new(file))
            .with_context(|| format!("Failed to stage removal of {}", file))?;
    }
    index.write().context("Failed to write index")?;

    println!("   {} Staged resolved file: {}", "📋".green(), file);
    Ok(())
}

async fn show_final_resolution_steps() -> Result<()> {
    println!("\n{}", "💡 Final Steps:".bright_yellow().bold());
    println!(
        "   1. {} - Verify all conflicts are resolved",
        "git status".cyan()
    );
    println!("   2. {} - Test your changes", "Run tests".cyan());
    println!("   3. {} - Complete the merge", "git commit".cyan());

    Ok(())
}

fn format_complexity(complexity: &ConflictComplexity) -> colored::ColoredString {
    match complexity {
        ConflictComplexity::Low => "Low".green(),
        ConflictComplexity::Medium => "Medium".yellow(),
        ConflictComplexity::High => "High".red(),
    }
}

// Data structures

#[derive(Debug)]
struct ConflictInfo {
    markers: Vec<ConflictMarker>,
}

#[derive(Debug)]
struct ConflictMarker {
    line_number: usize,
    our_label: String,
    their_label: String,
    our_content: String,
    their_content: String,
}

#[derive(Debug)]
struct ConflictAnalysis {
    conflict_type: String,
    complexity: ConflictComplexity,
    recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
enum ConflictComplexity {
    Low,
    Medium,
    High,
}

#[derive(Debug)]
struct FileStrategy {
    method: String,
    recommended_tools: Vec<String>,
    steps: Vec<String>,
    gotchas: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflict_markers_basic() {
        let content =
            "line1\n<<<<<<< HEAD\nour change\n=======\ntheir change\n>>>>>>> feature\nline2\n";
        let markers = parse_conflict_markers(content);
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].line_number, 2);
        assert_eq!(markers[0].our_label, "HEAD");
        assert_eq!(markers[0].their_label, "feature");
        assert_eq!(markers[0].our_content, "our change");
        assert_eq!(markers[0].their_content, "their change");
    }

    #[test]
    fn test_parse_conflict_markers_none() {
        let content = "no conflicts here\njust code\n";
        assert!(parse_conflict_markers(content).is_empty());
    }

    #[test]
    fn test_parse_conflict_markers_multiple() {
        let content =
            "<<<<<<< HEAD\na\n=======\nb\n>>>>>>> x\nmid\n<<<<<<< HEAD\nc\n=======\nd\n>>>>>>> x\n";
        let markers = parse_conflict_markers(content);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[1].our_content, "c");
        assert_eq!(markers[1].their_content, "d");
    }
}
