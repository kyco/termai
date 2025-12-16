/// Git commit message generation command handler
use crate::args::CommitArgs;
use crate::config::model::keys::ConfigKeys;
use crate::config::service::config_service;
use crate::git::{diff::DiffAnalyzer, repository::GitRepository};
use crate::llm::claude::adapter::claude_adapter;
use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_message::ChatMessage;
use crate::llm::common::model::role::Role;
use crate::llm::openai::adapter::responses_adapter::ResponsesAdapter;
use crate::llm::openai::model::responses_api::{ResponsesRequest, InputMessage, ResponseOutput, ContentItem};
use crate::llm::openai::model::model::Model;
use crate::repository::db::SqliteRepository;
use anyhow::{bail, Context, Result};
use colored::*;
use dialoguer::{Confirm, Input, Select};

/// Handle the commit subcommand
pub async fn handle_commit_command(args: &CommitArgs, repo: &SqliteRepository) -> Result<()> {
    println!("{}", "🔍 Analyzing Git repository...".bright_blue().bold());
    eprintln!("DEBUG: Starting commit command handler");

    // Discover and analyze the Git repository
    eprintln!("DEBUG: Discovering Git repository");
    let git_repo = GitRepository::discover(".").context(
        "❌ No Git repository found. Please run this command from within a Git repository.",
    )?;
    eprintln!("DEBUG: Git repository discovered successfully");

    // Check repository state
    if git_repo.is_merging() {
        bail!("❌ Repository is in a merge state. Complete the merge before generating commit messages.");
    }

    if git_repo.is_rebasing() {
        bail!("❌ Repository is in a rebase state. Complete the rebase before generating commit messages.");
    }

    // Get repository status
    eprintln!("DEBUG: Getting repository status");
    let status = git_repo
        .status()
        .context("Failed to get repository status")?;
    eprintln!("DEBUG: Repository status obtained");

    // Check for staged changes unless forced
    if !args.force && !status.has_staged_changes() {
        if args.add_all {
            println!(
                "{}",
                "📝 No staged changes found. The --add-all flag would stage all changes.".yellow()
            );
            println!(
                "{}",
                "   Note: This is a placeholder - actual staging not implemented yet.".dimmed()
            );
        } else {
            println!("{}", "❌ No staged changes found.".red().bold());
            println!();
            println!("{}", "💡 Suggestions:".bright_yellow().bold());
            println!(
                "   • Stage your changes first: {}",
                "git add <files>".cyan()
            );
            println!("   • Use {} to include all changes", "--add-all".cyan());
            println!("   • Use {} to generate a message anyway", "--force".cyan());
            return Ok(());
        }
    }

    // Display current status
    if !status.is_clean {
        println!("\n{}", "📊 Repository Status:".bright_green().bold());
        status.display_summary();
    }

    // Analyze staged changes
    eprintln!("DEBUG: Starting diff analysis");
    let diff_analyzer = DiffAnalyzer::new(git_repo.inner());
    let diff_summary = diff_analyzer
        .analyze_staged()
        .context("Failed to analyze staged changes")?;
    eprintln!("DEBUG: Diff analysis completed");

    println!("\n{}", "📋 Change Analysis:".bright_blue().bold());
    diff_summary.display_summary();

    // Generate commit message based on changes
    eprintln!("DEBUG: Starting commit message generation");
    let commit_message = generate_commit_message(&diff_summary, args, repo).await?;
    eprintln!("DEBUG: Commit message generation completed");

    println!("\n{}", "💬 Generated Commit Message:".bright_green().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());
    println!("{}", commit_message.subject.bright_white().bold());
    if let Some(ref body) = commit_message.body {
        println!();
        println!("{}", body.white());
    }
    println!("{}", "═══════════════════════════════".white().dimmed());

    // Handle auto-commit or interactive approval
    eprintln!("DEBUG: Starting interactive or auto-commit section");
    if args.auto {
        println!(
            "\n{}",
            "🚀 Auto-committing with generated message...".bright_green()
        );
        execute_commit(&git_repo, &commit_message).await?;
        println!("{}", "✅ Commit created successfully!".green().bold());
    } else {
        // Interactive workflow
        eprintln!("DEBUG: Creating interactive dialog");
        let actions = vec![
            "Accept and commit",
            "Edit message",
            "Regenerate message",
            "Cancel",
        ];

        eprintln!("DEBUG: About to show Select dialog");
        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .items(&actions)
            .default(0)
            .interact()?;
        eprintln!("DEBUG: Select dialog completed with selection: {}", selection);

        match selection {
            0 => {
                // Accept and commit
                execute_commit(&git_repo, &commit_message).await?;
                println!("{}", "✅ Commit created successfully!".green().bold());
            }
            1 => {
                // Edit message
                let edited_subject: String = Input::new()
                    .with_prompt("Edit commit subject")
                    .with_initial_text(&commit_message.subject)
                    .interact()?;

                let edited_body = if let Some(ref body) = commit_message.body {
                    let body_input: String = Input::new()
                        .with_prompt("Edit commit body (optional)")
                        .with_initial_text(body)
                        .allow_empty(true)
                        .interact()?;
                    if body_input.trim().is_empty() {
                        None
                    } else {
                        Some(body_input)
                    }
                } else {
                    let body_input: String = Input::new()
                        .with_prompt("Add commit body (optional)")
                        .allow_empty(true)
                        .interact()?;
                    if body_input.trim().is_empty() {
                        None
                    } else {
                        Some(body_input)
                    }
                };

                let edited_message = crate::git::commit::CommitMessage {
                    subject: edited_subject,
                    body: edited_body,
                    message_type: commit_message.message_type.clone(),
                    scope: commit_message.scope.clone(),
                };

                if Confirm::new()
                    .with_prompt("Commit with edited message?")
                    .default(true)
                    .interact()?
                {
                    execute_commit(&git_repo, &edited_message).await?;
                    println!("{}", "✅ Commit created successfully!".green().bold());
                }
            }
            2 => {
                // Regenerate message
                println!("{}", "🔄 Regenerating commit message...".bright_blue());
                let new_commit_message = generate_commit_message(&diff_summary, args, repo).await?;

                println!(
                    "\n{}",
                    "💬 Regenerated Commit Message:".bright_green().bold()
                );
                println!("{}", "═══════════════════════════════".white().dimmed());
                println!("{}", new_commit_message.subject.bright_white().bold());
                if let Some(ref body) = new_commit_message.body {
                    println!();
                    println!("{}", body.white());
                }
                println!("{}", "═══════════════════════════════".white().dimmed());

                if Confirm::new()
                    .with_prompt("Accept regenerated message and commit?")
                    .default(true)
                    .interact()?
                {
                    execute_commit(&git_repo, &new_commit_message).await?;
                    println!("{}", "✅ Commit created successfully!".green().bold());
                }
            }
            3 => {
                // Cancel
                println!("{}", "❌ Commit cancelled.".yellow());
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// Generate AI-powered commit message from diff analysis
async fn generate_ai_commit_message(
    diff_summary: &crate::git::diff::DiffSummary,
    args: &CommitArgs,
    repo: &SqliteRepository,
) -> Result<crate::git::commit::CommitMessage> {
    // Get provider configuration
    let provider = config_service::fetch_with_env_fallback(repo, &ConfigKeys::ProviderKey.to_key())
        .unwrap_or_else(|_| crate::config::entity::config_entity::ConfigEntity {
            id: None,
            key: ConfigKeys::ProviderKey.to_key(),
            value: "claude".to_string(),
        });

    // Create detailed diff analysis for AI
    let diff_context = create_diff_context_for_ai(diff_summary);

    // Create AI prompt for commit message generation
    let prompt = create_commit_message_prompt(diff_summary, args, &diff_context);

    // Call appropriate AI service
    eprintln!("DEBUG: Using provider: {}", provider.value);
    let ai_response = match provider.value.as_str() {
        "claude" => {
            eprintln!("DEBUG: Getting Claude API key");
            let api_key =
                config_service::fetch_with_env_fallback(repo, &ConfigKeys::ClaudeApiKey.to_key())
                    .context("Claude API key not configured")?;
            eprintln!("DEBUG: Calling Claude API");
            generate_with_claude(&prompt, &api_key.value).await?
        }
        "openai" => {
            eprintln!("DEBUG: Getting OpenAI API key");
            let api_key =
                config_service::fetch_with_env_fallback(repo, &ConfigKeys::ChatGptApiKey.to_key())
                    .context("OpenAI API key not configured")?;
            eprintln!("DEBUG: Calling OpenAI API (falling back to Chat Completions API)");
            // For now, fall back to a simple implementation that doesn't hang
            generate_with_openai_fallback(&prompt, &api_key.value).await?
        }
        _ => bail!("Unsupported provider: {}", provider.value),
    };
    eprintln!("DEBUG: AI response received");

    // Parse AI response into structured commit message
    parse_ai_commit_response(&ai_response, args)
}

/// Create diff context for AI analysis
fn create_diff_context_for_ai(diff_summary: &crate::git::diff::DiffSummary) -> String {
    let mut context = Vec::new();

    context.push(format!("Files changed: {}", diff_summary.files_changed));
    context.push(format!("Total additions: {}", diff_summary.total_additions));
    context.push(format!("Total deletions: {}", diff_summary.total_deletions));

    if !diff_summary.language_breakdown.is_empty() {
        context.push("\nLanguages modified:".to_string());
        for (lang, (additions, deletions)) in &diff_summary.language_breakdown {
            context.push(format!("  {}: +{} -{}", lang, additions, deletions));
        }
    }

    context.push("\nFile changes:".to_string());
    for file in &diff_summary.files {
        let path = file
            .new_path
            .as_ref()
            .or(file.old_path.as_ref())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let change_symbol = match file.change_type {
            crate::git::diff::ChangeType::Addition | crate::git::diff::ChangeType::Added => "A",
            crate::git::diff::ChangeType::Deletion | crate::git::diff::ChangeType::Deleted => "D",
            crate::git::diff::ChangeType::Modification | crate::git::diff::ChangeType::Modified => {
                "M"
            }
            crate::git::diff::ChangeType::Rename | crate::git::diff::ChangeType::Renamed => "R",
            crate::git::diff::ChangeType::Copy | crate::git::diff::ChangeType::Copied => "C",
        };

        context.push(format!(
            "  {} {} (+{} -{}) {}",
            change_symbol,
            path,
            file.additions,
            file.deletions,
            file.language.as_ref().unwrap_or(&"unknown".to_string())
        ));
    }

    context.join("\n")
}

/// Create AI prompt for commit message generation
fn create_commit_message_prompt(
    _diff_summary: &crate::git::diff::DiffSummary,
    args: &CommitArgs,
    diff_context: &str,
) -> String {
    let mut prompt_parts = vec![
        "You are an expert Git commit message generator. Analyze the provided diff and generate a high-quality commit message following conventional commits format.".to_string(),
        "".to_string(),
        "Guidelines:".to_string(),
        "- Use conventional commits format: type(scope): description".to_string(),
        "- Types: feat, fix, docs, style, refactor, test, chore, build, ci".to_string(),
        "- Keep subject line under 72 characters".to_string(),
        "- Provide a clear, concise description of what was done".to_string(),
        "- Add body with more details if changes are complex".to_string(),
        "- Focus on the 'what' and 'why', not the 'how'".to_string(),
        "".to_string(),
    ];

    // Add user preferences
    if let Some(ref msg_type) = args.message_type {
        prompt_parts.push(format!("Preferred commit type: {}", msg_type));
    }
    if let Some(ref scope) = args.scope {
        prompt_parts.push(format!("Preferred scope: {}", scope));
    }
    if let Some(ref template) = args.template {
        prompt_parts.push(format!("Template to incorporate: {}", template));
    }

    prompt_parts.push("".to_string());
    prompt_parts.push("Diff Analysis:".to_string());
    prompt_parts.push(diff_context.to_string());

    prompt_parts.push("".to_string());
    prompt_parts.push("Generate a commit message with:".to_string());
    prompt_parts.push("SUBJECT: [conventional commit format subject line]".to_string());
    prompt_parts.push("BODY: [optional detailed explanation if needed]".to_string());

    prompt_parts.join("\n")
}

/// Generate commit message using Claude
async fn generate_with_claude(prompt: &str, api_key: &str) -> Result<String> {
    let request = ChatCompletionRequest {
        model: "claude-opus-4-1-20250805".to_string(),
        max_tokens: 1000,
        messages: vec![
            ChatMessage {
                role: Role::User.to_string(),
                content: prompt.to_string(),
            }
        ],
        system: Some("You are an expert Git commit message generator. Generate clear, conventional commit messages based on diff analysis.".to_string()),
        thinking: None,
    };

    let (_, response) = claude_adapter::chat(&request, api_key).await?;

    if let Some(content) = response.content.first() {
        match content {
            crate::llm::claude::model::content_block::ContentBlock::Text { text } => {
                Ok(text.clone())
            }
            _ => bail!("Unexpected content block type from Claude"),
        }
    } else {
        bail!("No response content from Claude")
    }
}

/// Generate commit message using OpenAI
#[allow(dead_code)]
async fn generate_with_openai(prompt: &str, api_key: &str) -> Result<String> {
    eprintln!("DEBUG: Creating OpenAI request");
    let messages = vec![
        InputMessage {
            role: "system".to_string(),
            content: "You are an expert Git commit message generator. Generate clear, conventional commit messages based on diff analysis.".to_string(),
        },
        InputMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }
    ];

    let request = ResponsesRequest::from_messages(Model::Gpt5.to_string(), messages);

    eprintln!("DEBUG: Sending OpenAI API request");
    let response = ResponsesAdapter::chat(&request, api_key).await?;
    eprintln!("DEBUG: OpenAI API response received");

    // Check if request was successful
    if response.status != "completed" {
        if let Some(error) = response.error {
            bail!("OpenAI API error: {}", error.message);
        } else {
            bail!("Request failed with status: {}", response.status);
        }
    }

    // Extract text from response
    for output in response.output {
        match output {
            ResponseOutput::Message { content, .. } => {
                let message_text: String = content
                    .into_iter()
                    .filter_map(|item| match item {
                        ContentItem::OutputText { text, .. } => Some(text),
                    })
                    .collect::<Vec<String>>()
                    .join("\n");
                
                if !message_text.is_empty() {
                    return Ok(message_text);
                }
            }
            _ => continue,
        }
    }

    bail!("No response content from OpenAI")
}

/// Fallback OpenAI implementation using standard Chat Completions API
async fn generate_with_openai_fallback(prompt: &str, api_key: &str) -> Result<String> {
    eprintln!("DEBUG: Using OpenAI Chat Completions fallback");
    use reqwest::Client;
    use serde_json::json;
    
    let client = Client::new();
    
    let request_body = json!({
        "model": "gpt-5.2",
        "messages": [
            {"role": "system", "content": "You are an expert Git commit message generator. Generate clear, conventional commit messages based on diff analysis."},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 1000,
        "reasoning_effort": "xhigh"
    });

    eprintln!("DEBUG: Sending fallback OpenAI request");
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("OpenAI Chat Completions API failed with status {}: {}", status, error_text);
    }

    let response_json: serde_json::Value = response.json().await?;
    eprintln!("DEBUG: Fallback OpenAI response received: {:?}", response_json);

    // Extract the message content
    if let Some(choices) = response_json["choices"].as_array() {
        if let Some(choice) = choices.first() {
            if let Some(message) = choice["message"].as_object() {
                if let Some(content) = message["content"].as_str() {
                    eprintln!("DEBUG: Extracted content: {}", content);
                    return Ok(content.to_string());
                }
            }
        }
    }

    bail!("No response content from OpenAI Chat Completions API")
}

/// Parse AI response into structured commit message
fn parse_ai_commit_response(
    response: &str,
    args: &CommitArgs,
) -> Result<crate::git::commit::CommitMessage> {
    // Clean up the response - remove markdown code blocks
    let cleaned_response = response
        .trim()
        .strip_prefix("```")
        .unwrap_or(response)
        .strip_suffix("```")
        .unwrap_or(response)
        .trim();
    
    let lines: Vec<&str> = cleaned_response.lines().collect();
    eprintln!("DEBUG: Parsing response lines: {:?}", lines);

    // Extract subject and body from AI response
    let mut subject = None;
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for line in lines {
        if line.trim().starts_with("SUBJECT:") {
            subject = Some(line.trim_start_matches("SUBJECT:").trim().to_string());
        } else if line.trim().starts_with("BODY:") {
            let body_content = line.trim_start_matches("BODY:").trim();
            if !body_content.is_empty()
                && body_content != "[optional detailed explanation if needed]"
            {
                body_lines.push(body_content.to_string());
            }
            in_body = true;
        } else if in_body && !line.trim().is_empty() {
            body_lines.push(line.trim().to_string());
        } else if !in_body && !line.trim().is_empty() && subject.is_none() {
            // If no explicit SUBJECT: marker, treat first non-empty line as subject
            subject = Some(line.trim().to_string());
        } else if !line.trim().is_empty() && subject.is_some() && !in_body {
            // Start collecting body after subject
            in_body = true;
            body_lines.push(line.trim().to_string());
        } else if in_body && !line.trim().is_empty() {
            body_lines.push(line.trim().to_string());
        }
    }

    // Extract type and scope from subject
    let subject = subject.unwrap_or_else(|| "chore: update files".to_string());
    let (message_type, scope) = parse_conventional_commit(&subject);

    // Limit subject to 72 characters
    let subject = if subject.len() > 72 {
        format!("{}...", &subject[..69])
    } else {
        subject
    };

    let body = if body_lines.is_empty() {
        None
    } else {
        Some(body_lines.join("\n"))
    };

    Ok(crate::git::commit::CommitMessage {
        subject,
        body,
        message_type: args.message_type.clone().unwrap_or(message_type),
        scope: args.scope.clone().or(scope),
    })
}

/// Parse conventional commit format to extract type and scope
fn parse_conventional_commit(subject: &str) -> (String, Option<String>) {
    // Pattern: type(scope): description or type: description
    if let Some(colon_pos) = subject.find(':') {
        let prefix = &subject[..colon_pos];
        if let Some(paren_start) = prefix.find('(') {
            if let Some(paren_end) = prefix.find(')') {
                let msg_type = prefix[..paren_start].trim().to_string();
                let scope = prefix[paren_start + 1..paren_end].trim().to_string();
                return (msg_type, if scope.is_empty() { None } else { Some(scope) });
            }
        } else {
            let msg_type = prefix.trim().to_string();
            return (msg_type, None);
        }
    }

    // Fallback
    ("feat".to_string(), None)
}

/// Generate a commit message from diff summary and arguments
async fn generate_commit_message(
    diff_summary: &crate::git::diff::DiffSummary,
    args: &CommitArgs,
    repo: &SqliteRepository,
) -> Result<crate::git::commit::CommitMessage> {
    // Try AI-powered generation first, fall back to heuristics
    match generate_ai_commit_message(diff_summary, args, repo).await {
        Ok(message) => Ok(message),
        Err(e) => {
            println!("{}", format!("⚠️  AI generation failed: {}", e).yellow());
            println!("{}", "📝 Falling back to heuristic generation...".dimmed());

            // Fallback to rule-based generation
            let message_type = args
                .message_type
                .as_deref()
                .or_else(|| infer_message_type(diff_summary))
                .unwrap_or("feat");

            let scope = args.scope.as_deref();

            let subject = if let Some(template) = &args.template {
                format!("{}: {}", message_type, template)
            } else {
                generate_subject_from_changes(diff_summary, message_type, scope)
            };

            let body = generate_body_from_changes(diff_summary);

            Ok(crate::git::commit::CommitMessage {
                subject: subject.chars().take(72).collect(), // Limit to 72 characters
                body: if body.trim().is_empty() {
                    None
                } else {
                    Some(body)
                },
                message_type: message_type.to_string(),
                scope: scope.map(|s| s.to_string()),
            })
        }
    }
}

/// Infer commit message type from changes
fn infer_message_type(diff_summary: &crate::git::diff::DiffSummary) -> Option<&'static str> {
    // Simple heuristics based on file changes
    let has_new_files = diff_summary
        .files
        .iter()
        .any(|f| f.change_type == crate::git::diff::ChangeType::Addition);
    let has_deleted_files = diff_summary
        .files
        .iter()
        .any(|f| f.change_type == crate::git::diff::ChangeType::Deletion);
    let has_test_files = diff_summary.files.iter().any(|f| {
        f.new_path
            .as_ref()
            .or(f.old_path.as_ref())
            .map(|p| p.to_string_lossy().contains("test"))
            .unwrap_or(false)
    });
    let has_doc_files = diff_summary.files.iter().any(|f| {
        f.new_path
            .as_ref()
            .or(f.old_path.as_ref())
            .map(|p| {
                let path_str = p.to_string_lossy();
                path_str.ends_with(".md") || path_str.contains("doc") || path_str.contains("README")
            })
            .unwrap_or(false)
    });

    if has_doc_files {
        Some("docs")
    } else if has_test_files {
        Some("test")
    } else if has_deleted_files {
        Some("refactor")
    } else if has_new_files {
        Some("feat")
    } else {
        Some("fix")
    }
}

/// Generate commit subject from changes
fn generate_subject_from_changes(
    diff_summary: &crate::git::diff::DiffSummary,
    message_type: &str,
    scope: Option<&str>,
) -> String {
    let scope_prefix = if let Some(scope) = scope {
        format!("({})", scope)
    } else {
        String::new()
    };

    if diff_summary.files_changed == 1 {
        if let Some(file) = diff_summary.files.first() {
            let file_name = file
                .new_path
                .as_ref()
                .or(file.old_path.as_ref())
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("file");

            match file.change_type {
                crate::git::diff::ChangeType::Addition | crate::git::diff::ChangeType::Added => {
                    format!("{}{}: add {}", message_type, scope_prefix, file_name)
                }
                crate::git::diff::ChangeType::Deletion | crate::git::diff::ChangeType::Deleted => {
                    format!("{}{}: remove {}", message_type, scope_prefix, file_name)
                }
                crate::git::diff::ChangeType::Modification
                | crate::git::diff::ChangeType::Modified => {
                    format!("{}{}: update {}", message_type, scope_prefix, file_name)
                }
                crate::git::diff::ChangeType::Rename | crate::git::diff::ChangeType::Renamed => {
                    format!("{}{}: rename {}", message_type, scope_prefix, file_name)
                }
                crate::git::diff::ChangeType::Copy | crate::git::diff::ChangeType::Copied => {
                    format!("{}{}: copy {}", message_type, scope_prefix, file_name)
                }
            }
        } else {
            format!("{}{}: update files", message_type, scope_prefix)
        }
    } else {
        format!(
            "{}{}: update {} files",
            message_type, scope_prefix, diff_summary.files_changed
        )
    }
}

/// Generate commit body from changes
fn generate_body_from_changes(diff_summary: &crate::git::diff::DiffSummary) -> String {
    if diff_summary.files_changed <= 1 {
        return String::new();
    }

    let mut body_parts = Vec::new();

    // Summarize changes by type
    let additions = diff_summary
        .files
        .iter()
        .filter(|f| f.change_type == crate::git::diff::ChangeType::Addition)
        .count();
    let modifications = diff_summary
        .files
        .iter()
        .filter(|f| f.change_type == crate::git::diff::ChangeType::Modification)
        .count();
    let deletions = diff_summary
        .files
        .iter()
        .filter(|f| f.change_type == crate::git::diff::ChangeType::Deletion)
        .count();

    if additions > 0 {
        body_parts.push(format!(
            "- Add {} new file{}",
            additions,
            if additions == 1 { "" } else { "s" }
        ));
    }
    if modifications > 0 {
        body_parts.push(format!(
            "- Update {} file{}",
            modifications,
            if modifications == 1 { "" } else { "s" }
        ));
    }
    if deletions > 0 {
        body_parts.push(format!(
            "- Remove {} file{}",
            deletions,
            if deletions == 1 { "" } else { "s" }
        ));
    }

    // Add statistics
    if diff_summary.total_additions > 0 || diff_summary.total_deletions > 0 {
        body_parts.push(format!(
            "- {} insertions(+), {} deletions(-)",
            diff_summary.total_additions, diff_summary.total_deletions
        ));
    }

    body_parts.join("\n")
}

/// Execute the commit with the generated message
async fn execute_commit(
    git_repo: &GitRepository,
    commit_message: &crate::git::commit::CommitMessage,
) -> Result<()> {
    println!("{}", "🚀 Executing commit...".bright_blue());

    // Get Git user configuration
    let user_config = git_repo
        .user_config()
        .context("Failed to get Git user configuration")?;

    // Create the full commit message
    let full_message = if let Some(ref body) = commit_message.body {
        format!("{}\n\n{}", commit_message.subject, body)
    } else {
        commit_message.subject.clone()
    };

    // Get the repository's inner git2::Repository
    let repo = git_repo.inner();

    // Get current index (staged changes)
    let mut index = repo.index().context("Failed to get repository index")?;

    // Write the index to create a tree
    let tree_id = index
        .write_tree()
        .context("Failed to write tree from index")?;
    let tree = repo.find_tree(tree_id).context("Failed to find tree")?;

    // Get the current HEAD commit (parent)
    let parent_commit = if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            Some(commit)
        } else {
            None
        }
    } else {
        None // First commit
    };

    // Create signature for the commit
    let signature = git2::Signature::now(&user_config.name, &user_config.email)
        .context("Failed to create Git signature")?;

    // Create the commit
    let parents = if let Some(ref parent) = parent_commit {
        vec![parent]
    } else {
        vec![]
    };

    let commit_id = repo
        .commit(
            Some("HEAD"),  // Update HEAD
            &signature,    // Author
            &signature,    // Committer
            &full_message, // Commit message
            &tree,         // Tree
            &parents,      // Parents
        )
        .context("Failed to create commit")?;

    println!("{}", "✅ Commit created successfully!".green().bold());
    println!(
        "{}",
        format!(
            "   Commit ID: {}",
            commit_id.to_string()[..8].bright_yellow()
        )
    );
    println!(
        "{}",
        format!("   Message: {}", commit_message.subject.bright_white())
    );

    if let Some(ref body) = commit_message.body {
        println!("{}", format!("   Body: {}", body.dimmed()));
    }

    Ok(())
}
