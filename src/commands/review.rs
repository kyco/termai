/// Git code review command handler
use crate::args::{ReviewArgs, ReviewFormat};
use crate::git::{diff::DiffAnalyzer, repository::GitRepository};
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use serde_json::json;
use std::fs;

/// Handle the review subcommand
pub async fn handle_review_command(args: &ReviewArgs, _repo: &SqliteRepository) -> Result<()> {
    println!(
        "{}",
        "🔍 Starting code review analysis...".bright_blue().bold()
    );

    // Discover and analyze the Git repository
    let git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;

    // Check repository state
    if git_repo.is_merging() || git_repo.is_rebasing() {
        println!(
            "{}",
            "⚠️  Repository is in an active merge/rebase state.".yellow()
        );
        println!(
            "{}",
            "   Review results may include merge artifacts.".dimmed()
        );
    }

    // Get repository status
    let status = git_repo
        .status()
        .context("Failed to get repository status")?;

    // Check for staged changes
    if !status.has_staged_changes() {
        println!("{}", "❌ No staged changes found to review.".red().bold());
        println!();
        println!("{}", "💡 Suggestions:".bright_yellow().bold());
        println!(
            "   • Stage your changes first: {}",
            "git add <files>".cyan()
        );
        println!(
            "   • Review all uncommitted changes with: {}",
            "git add . && termai review".cyan()
        );
        return Ok(());
    }

    // Display current status
    println!("\n{}", "📊 Repository Status:".bright_green().bold());
    status.display_summary();

    // Analyze staged changes
    let diff_analyzer = DiffAnalyzer::new(git_repo.inner());
    let diff_summary = diff_analyzer
        .analyze_staged()
        .context("Failed to analyze staged changes")?;

    println!("\n{}", "📋 Change Analysis:".bright_blue().bold());
    diff_summary.display_summary();

    // Filter files if specific patterns provided
    let filtered_files = if !args.files.is_empty() {
        filter_files_by_patterns(&diff_summary.files, &args.files)
    } else {
        diff_summary.files.clone()
    };

    if filtered_files.is_empty() {
        println!("{}", "❌ No files match the specified patterns.".red());
        return Ok(());
    }

    // Perform code review based on depth
    let review_result = perform_code_review(&filtered_files, args).await?;

    // Display results based on format
    match args.format {
        ReviewFormat::Text => display_text_review(&review_result),
        ReviewFormat::Json => display_json_review(&review_result)?,
        ReviewFormat::Markdown => display_markdown_review(&review_result),
    }

    // Save to file if requested
    if let Some(ref output_path) = args.output {
        save_review_to_file(&review_result, output_path, &args.format)?;
        println!(
            "\n{} Review results saved to: {}",
            "💾".bright_green(),
            output_path.bright_cyan()
        );
    }

    // Display summary
    display_review_summary(&review_result);

    Ok(())
}

/// Filter files by user-provided patterns
fn filter_files_by_patterns(
    files: &[crate::git::diff::FileChange],
    patterns: &[String],
) -> Vec<crate::git::diff::FileChange> {
    files
        .iter()
        .filter(|file| {
            let file_path = file
                .new_path
                .as_ref()
                .or(file.old_path.as_ref())
                .map(|p| p.to_string_lossy())
                .unwrap_or_default();

            patterns.iter().any(|pattern| {
                // Simple pattern matching (could be enhanced with glob patterns)
                file_path.contains(pattern)
                    || pattern.contains("*") && simple_glob_match(pattern, &file_path)
            })
        })
        .cloned()
        .collect()
}

/// Simple glob pattern matching
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return text.starts_with(prefix) && text.ends_with(suffix);
        }
    }
    text.contains(pattern)
}

/// Perform code review analysis
async fn perform_code_review(
    files: &[crate::git::diff::FileChange],
    args: &ReviewArgs,
) -> Result<ReviewResult> {
    println!(
        "\n{}",
        format!(
            "🔍 Performing {} review of {} files...",
            format!("{:?}", args.depth).to_lowercase(),
            files.len()
        )
        .bright_blue()
    );

    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    let mut positive_feedback = Vec::new();

    // Analyze each file
    for file in files {
        let file_path = file
            .new_path
            .as_ref()
            .or(file.old_path.as_ref())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("   • Analyzing {}", file_path.bright_white());

        // File-specific analysis based on language and change type
        analyze_file_changes(file, &mut issues, &mut suggestions, args).await;
    }

    // General review feedback
    analyze_overall_changes(files, &mut suggestions, &mut positive_feedback, args).await;

    // Add security analysis if requested
    if args.security {
        perform_security_analysis(files, &mut issues, &mut suggestions).await;
    }

    // Add performance analysis if requested
    if args.performance {
        perform_performance_analysis(files, &mut issues, &mut suggestions).await;
    }

    Ok(ReviewResult {
        issues,
        suggestions,
        positive_feedback,
    })
}

/// Analyze individual file changes
async fn analyze_file_changes(
    file: &crate::git::diff::FileChange,
    issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
    args: &ReviewArgs,
) {
    let _file_path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Analyze based on file type and change type
    if let Some(ref language) = file.language {
        match language.as_str() {
            "Rust" => analyze_rust_file(file, issues, suggestions, args).await,
            "JavaScript" | "TypeScript" => {
                analyze_js_ts_file(file, issues, suggestions, args).await
            }
            "Python" => analyze_python_file(file, issues, suggestions, args).await,
            _ => analyze_generic_file(file, issues, suggestions, args).await,
        }
    }

    // Check for common patterns regardless of language
    analyze_common_patterns(file, issues, suggestions).await;
}

/// Analyze Rust-specific patterns
async fn analyze_rust_file(
    file: &crate::git::diff::FileChange,
    issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
    _args: &ReviewArgs,
) {
    let file_path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Rust-specific checks
    if file.change_type == crate::git::diff::ChangeType::Addition {
        suggestions.push(format!(
            "New Rust file '{}': Consider adding documentation and tests",
            file_path
        ));
    }

    if file_path.ends_with(".rs") && !file_path.contains("test") && !file_path.contains("example") {
        suggestions.push("Consider adding #[derive(Debug)] for better error messages".to_string());
    }

    // Placeholder for more detailed analysis
    if file.additions > 50 {
        issues.push(ReviewIssue {
            file_path: file_path.clone(),
            line_number: None,
            severity: IssueSeverity::Info,
            message: "Large addition - consider breaking into smaller commits".to_string(),
            suggestion: Some(
                "Split large changes into focused commits for better review".to_string(),
            ),
        });
    }
}

/// Analyze JavaScript/TypeScript-specific patterns
async fn analyze_js_ts_file(
    file: &crate::git::diff::FileChange,
    _issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
    _args: &ReviewArgs,
) {
    let file_path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if file_path.ends_with(".js") {
        suggestions.push("Consider migrating to TypeScript for better type safety".to_string());
    }

    if file.change_type == crate::git::diff::ChangeType::Addition {
        suggestions.push(format!(
            "New JS/TS file '{}': Ensure proper error handling and testing",
            file_path
        ));
    }
}

/// Analyze Python-specific patterns
async fn analyze_python_file(
    file: &crate::git::diff::FileChange,
    _issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
    _args: &ReviewArgs,
) {
    let file_path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if file.change_type == crate::git::diff::ChangeType::Addition {
        suggestions.push(format!(
            "New Python file '{}': Consider adding type hints and docstrings",
            file_path
        ));
    }
}

/// Analyze generic file patterns
async fn analyze_generic_file(
    file: &crate::git::diff::FileChange,
    _issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
    _args: &ReviewArgs,
) {
    if file.is_binary {
        suggestions.push(
            "Binary file detected - ensure it's necessary and not accidentally committed"
                .to_string(),
        );
    }
}

/// Analyze common patterns across all languages
async fn analyze_common_patterns(
    file: &crate::git::diff::FileChange,
    issues: &mut Vec<ReviewIssue>,
    _suggestions: &mut Vec<String>,
) {
    let file_path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check for large files
    if file.additions > 100 {
        issues.push(ReviewIssue {
            file_path: file_path.clone(),
            line_number: None,
            severity: IssueSeverity::Warning,
            message: format!("Large file change: {} additions", file.additions),
            suggestion: Some(
                "Consider breaking large changes into smaller, focused commits".to_string(),
            ),
        });
    }

    // Check for deleted files
    if file.change_type == crate::git::diff::ChangeType::Deletion {
        issues.push(ReviewIssue {
            file_path: file_path.clone(),
            line_number: None,
            severity: IssueSeverity::Info,
            message: "File deletion detected".to_string(),
            suggestion: Some(
                "Ensure file deletion is intentional and no dependencies are broken".to_string(),
            ),
        });
    }
}

/// Analyze overall changes across all files
async fn analyze_overall_changes(
    files: &[crate::git::diff::FileChange],
    suggestions: &mut Vec<String>,
    positive_feedback: &mut Vec<String>,
    _args: &ReviewArgs,
) {
    let total_additions = files.iter().map(|f| f.additions).sum::<usize>();
    let total_deletions = files.iter().map(|f| f.deletions).sum::<usize>();

    if total_additions > 0 && total_deletions > 0 {
        positive_feedback.push(
            "Good mix of additions and deletions - suggests thoughtful refactoring".to_string(),
        );
    }

    if files.len() > 10 {
        suggestions.push(
            "Large number of files changed - consider splitting into multiple commits".to_string(),
        );
    }

    // Check for consistent patterns
    let languages: std::collections::HashSet<_> =
        files.iter().filter_map(|f| f.language.as_ref()).collect();

    if languages.len() == 1 {
        positive_feedback
            .push("Changes focused on single language - good for maintainability".to_string());
    }
}

/// Perform security-focused analysis
async fn perform_security_analysis(
    files: &[crate::git::diff::FileChange],
    issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
) {
    println!("   🔒 Performing security analysis...");

    for file in files {
        let file_path = file
            .new_path
            .as_ref()
            .or(file.old_path.as_ref())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Check for potential security issues based on file names/patterns
        if file_path.to_lowercase().contains("password")
            || file_path.to_lowercase().contains("secret")
            || file_path.to_lowercase().contains("key")
        {
            issues.push(ReviewIssue {
                file_path: file_path.clone(),
                line_number: None,
                severity: IssueSeverity::Warning,
                message: "File name suggests sensitive information".to_string(),
                suggestion: Some("Ensure no secrets are hardcoded in this file".to_string()),
            });
        }
    }

    suggestions
        .push("Consider running security linters and dependency vulnerability scans".to_string());
}

/// Perform performance-focused analysis
async fn perform_performance_analysis(
    files: &[crate::git::diff::FileChange],
    issues: &mut Vec<ReviewIssue>,
    suggestions: &mut Vec<String>,
) {
    println!("   ⚡ Performing performance analysis...");

    for file in files {
        if file.additions > 200 {
            let file_path = file
                .new_path
                .as_ref()
                .or(file.old_path.as_ref())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            issues.push(ReviewIssue {
                file_path,
                line_number: None,
                severity: IssueSeverity::Info,
                message: "Large code addition - consider performance implications".to_string(),
                suggestion: Some("Profile performance impact of large code changes".to_string()),
            });
        }
    }

    suggestions.push("Consider benchmarking performance-critical changes".to_string());
}

/// Display review results in text format
fn display_text_review(review: &ReviewResult) {
    println!("\n{}", "🔍 Code Review Results".bright_blue().bold());
    println!("{}", "═════════════════════════".white().dimmed());

    if !review.issues.is_empty() {
        println!("\n{}", "⚠️  Issues Found:".bright_red().bold());
        for issue in &review.issues {
            let severity_color = match issue.severity {
                IssueSeverity::Critical => "🔴".red(),
                IssueSeverity::Error => "🟠".yellow(),
                IssueSeverity::Warning => "🟡".yellow(),
                IssueSeverity::Info => "🔵".blue(),
            };

            println!(
                "   {} {} in {}",
                severity_color,
                issue.message.bright_white(),
                issue.file_path.cyan()
            );

            if let Some(line) = issue.line_number {
                println!("      Line: {}", line.to_string().bright_yellow());
            }

            if let Some(ref suggestion) = issue.suggestion {
                println!("      💡 {}", suggestion.bright_green());
            }
            println!();
        }
    }

    if !review.suggestions.is_empty() {
        println!("{}", "💡 Suggestions:".bright_yellow().bold());
        for suggestion in &review.suggestions {
            println!("   • {}", suggestion.white());
        }
        println!();
    }

    if !review.positive_feedback.is_empty() {
        println!("{}", "✅ Positive Findings:".bright_green().bold());
        for feedback in &review.positive_feedback {
            println!("   • {}", feedback.green());
        }
    }
}

/// Display review results in JSON format
fn display_json_review(review: &ReviewResult) -> Result<()> {
    let json_output = json!({
        "issues": review.issues.iter().map(|issue| json!({
            "file_path": issue.file_path,
            "line_number": issue.line_number,
            "severity": format!("{:?}", issue.severity),
            "message": issue.message,
            "suggestion": issue.suggestion
        })).collect::<Vec<_>>(),
        "suggestions": review.suggestions,
        "positive_feedback": review.positive_feedback
    });

    println!("{}", serde_json::to_string_pretty(&json_output)?);
    Ok(())
}

/// Display review results in Markdown format
fn display_markdown_review(review: &ReviewResult) {
    println!("# Code Review Results\n");

    if !review.issues.is_empty() {
        println!("## Issues Found\n");
        for issue in &review.issues {
            let severity_emoji = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::Error => "🟠",
                IssueSeverity::Warning => "🟡",
                IssueSeverity::Info => "🔵",
            };

            println!(
                "### {} {} in `{}`\n",
                severity_emoji, issue.message, issue.file_path
            );

            if let Some(line) = issue.line_number {
                println!("**Line:** {}\n", line);
            }

            if let Some(ref suggestion) = issue.suggestion {
                println!("💡 **Suggestion:** {}\n", suggestion);
            }
        }
    }

    if !review.suggestions.is_empty() {
        println!("## Suggestions\n");
        for suggestion in &review.suggestions {
            println!("- {}", suggestion);
        }
        println!();
    }

    if !review.positive_feedback.is_empty() {
        println!("## Positive Findings\n");
        for feedback in &review.positive_feedback {
            println!("- ✅ {}", feedback);
        }
    }
}

/// Save review results to file
fn save_review_to_file(
    review: &ReviewResult,
    output_path: &str,
    format: &ReviewFormat,
) -> Result<()> {
    let content = match format {
        ReviewFormat::Json => {
            let json_output = json!({
                "issues": review.issues.iter().map(|issue| json!({
                    "file_path": issue.file_path,
                    "line_number": issue.line_number,
                    "severity": format!("{:?}", issue.severity),
                    "message": issue.message,
                    "suggestion": issue.suggestion
                })).collect::<Vec<_>>(),
                "suggestions": review.suggestions,
                "positive_feedback": review.positive_feedback
            });
            serde_json::to_string_pretty(&json_output)?
        }
        ReviewFormat::Markdown => {
            let mut content = String::new();
            content.push_str("# Code Review Results\n\n");

            if !review.issues.is_empty() {
                content.push_str("## Issues Found\n\n");
                for issue in &review.issues {
                    let severity_emoji = match issue.severity {
                        IssueSeverity::Critical => "🔴",
                        IssueSeverity::Error => "🟠",
                        IssueSeverity::Warning => "🟡",
                        IssueSeverity::Info => "🔵",
                    };

                    content.push_str(&format!(
                        "### {} {} in `{}`\n\n",
                        severity_emoji, issue.message, issue.file_path
                    ));

                    if let Some(line) = issue.line_number {
                        content.push_str(&format!("**Line:** {}\n\n", line));
                    }

                    if let Some(ref suggestion) = issue.suggestion {
                        content.push_str(&format!("💡 **Suggestion:** {}\n\n", suggestion));
                    }
                }
            }

            if !review.suggestions.is_empty() {
                content.push_str("## Suggestions\n\n");
                for suggestion in &review.suggestions {
                    content.push_str(&format!("- {}\n", suggestion));
                }
                content.push('\n');
            }

            if !review.positive_feedback.is_empty() {
                content.push_str("## Positive Findings\n\n");
                for feedback in &review.positive_feedback {
                    content.push_str(&format!("- ✅ {}\n", feedback));
                }
            }

            content
        }
        ReviewFormat::Text => {
            let mut content = String::new();
            content.push_str("Code Review Results\n");
            content.push_str("==================\n\n");

            if !review.issues.is_empty() {
                content.push_str("Issues Found:\n");
                for issue in &review.issues {
                    let severity_text = format!("{:?}", issue.severity);
                    content.push_str(&format!(
                        "  [{}] {} in {}\n",
                        severity_text, issue.message, issue.file_path
                    ));

                    if let Some(line) = issue.line_number {
                        content.push_str(&format!("    Line: {}\n", line));
                    }

                    if let Some(ref suggestion) = issue.suggestion {
                        content.push_str(&format!("    Suggestion: {}\n", suggestion));
                    }
                    content.push('\n');
                }
            }

            if !review.suggestions.is_empty() {
                content.push_str("Suggestions:\n");
                for suggestion in &review.suggestions {
                    content.push_str(&format!("  - {}\n", suggestion));
                }
                content.push('\n');
            }

            if !review.positive_feedback.is_empty() {
                content.push_str("Positive Findings:\n");
                for feedback in &review.positive_feedback {
                    content.push_str(&format!("  - {}\n", feedback));
                }
            }

            content
        }
    };

    fs::write(output_path, content)
        .with_context(|| format!("Failed to write review results to {}", output_path))?;

    Ok(())
}

/// Display review summary
fn display_review_summary(review: &ReviewResult) {
    let total_issues = review.issues.len();
    let critical_issues = review
        .issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Critical)
        .count();
    let error_issues = review
        .issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Error)
        .count();
    let warning_issues = review
        .issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Warning)
        .count();

    println!("\n{}", "📊 Review Summary:".bright_blue().bold());
    println!("{}", "══════════════════".white().dimmed());

    if total_issues == 0 {
        println!("   {}", "✅ No issues found!".green().bold());
    } else {
        if critical_issues > 0 {
            println!("   {} {} critical issues", "🔴".red(), critical_issues);
        }
        if error_issues > 0 {
            println!("   {} {} errors", "🟠".yellow(), error_issues);
        }
        if warning_issues > 0 {
            println!("   {} {} warnings", "🟡".yellow(), warning_issues);
        }
    }

    println!(
        "   {} {} suggestions",
        "💡".bright_yellow(),
        review.suggestions.len()
    );
    println!(
        "   {} {} positive findings",
        "✅".green(),
        review.positive_feedback.len()
    );
}

// Supporting types and implementations
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub issues: Vec<ReviewIssue>,
    pub suggestions: Vec<String>,
    pub positive_feedback: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReviewIssue {
    pub file_path: String,
    pub line_number: Option<u32>,
    pub severity: IssueSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
