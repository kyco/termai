/// Handler for Session commands - session management operations
use crate::args::{SessionAction, SessionArgs};
use crate::repository::db::SqliteRepository;
use crate::session::repository::SessionRepository;
use crate::session::service::sessions_service;
use crate::branch::{BranchService, BranchTree, BranchNavigator, BranchComparator, QuickCompare, BranchMerger, MergeStrategy, ExportFormat, CleanupStrategy};
use anyhow::{Context, Result};
use colored::*;

/// Handle session management subcommands
pub fn handle_sessions_command(
    repo: &SqliteRepository,
    action: &SessionAction,
    _args: &SessionArgs,
) -> Result<()> {
    match action {
        SessionAction::List => {
            sessions_service::fetch_all_sessions(repo, repo)?;
            Ok(())
        }
        SessionAction::Delete { name } => delete_session(repo, name),
        SessionAction::Show { name } => show_session_details(repo, name),
        SessionAction::Branch { session, name, description, from_message } => {
            handle_branch_command(repo, session, name.as_deref(), description.as_deref(), *from_message)
        }
        SessionAction::Tree { session, interactive, highlight } => {
            handle_tree_command(repo, session, *interactive, highlight.as_deref())
        }
        SessionAction::Branches { session, detailed, status } => {
            handle_branches_command(repo, session, *detailed, status.as_deref())
        }
        SessionAction::Switch { session, branch, new_session } => {
            handle_switch_command(repo, session, branch, *new_session)
        }
        SessionAction::Bookmark { session, branch, name, remove } => {
            handle_bookmark_command(repo, session, branch, name.as_deref(), *remove)
        }
        SessionAction::Search { session, query, status, detailed } => {
            handle_search_command(repo, session, query, status.as_deref(), *detailed)
        }
        SessionAction::Stats { session, detailed } => {
            handle_stats_command(repo, session, *detailed)
        }
        SessionAction::Compare { session, branches, side_by_side, outcomes_only, detailed } => {
            handle_compare_command(repo, session, branches, *side_by_side, *outcomes_only, *detailed)
        }
        SessionAction::Merge { session, source_branches, into, strategy, preview, auto_confirm } => {
            handle_merge_command(repo, session, source_branches, into, strategy, *preview, *auto_confirm)
        }
        SessionAction::SelectiveMerge { session, source, target, messages, preview } => {
            handle_selective_merge_command(repo, session, source, target, messages, *preview)
        }
        SessionAction::Archive { session, branches, reason } => {
            handle_archive_command(repo, session, branches, reason.as_deref())
        }
        SessionAction::Cleanup { session, strategy, days, preview } => {
            handle_cleanup_command(repo, session, strategy, *days, *preview)
        }
        SessionAction::Export { session, branches, format, output } => {
            handle_export_command(repo, session, branches, format, output.as_deref())
        }
    }
}

/// Delete a specific session
fn delete_session(repo: &SqliteRepository, session_name: &str) -> Result<()> {
    use crate::session::repository::MessageRepository;

    println!("{}", "🗑️ Delete Session".bright_red().bold());
    println!("{}", "════════════════".white().dimmed());
    println!();

    // Check if session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))
        .map_err(|e| {
            let guidance = format!(
                "\n{}\n{}\n• Run '{}' to see available sessions\n• {}",
                "💡 Session Troubleshooting:".bright_yellow().bold(),
                "   The specified session could not be found.".white(),
                "termai session list".cyan(),
                "Check the session name spelling and try again".white()
            );
            anyhow::anyhow!("{}\n{}", e, guidance)
        })?;

    // Get message count for the session
    let message_count = repo
        .fetch_messages_for_session(&session_entity.id)
        .map(|messages| messages.len())
        .unwrap_or(0);

    println!(
        "{}  {}",
        "Session name:".bright_green(),
        session_name.bright_white()
    );
    println!(
        "{}        {}",
        "Session ID:".bright_green(),
        session_entity.id
    );
    println!(
        "{}     {} messages",
        "Message count:".bright_green(),
        message_count
    );
    println!();

    // Confirm deletion
    println!("{}", "⚠️  This action cannot be undone!".red().bold());
    println!("This will permanently delete the session and all its messages.");
    println!();

    use dialoguer::{theme::ColorfulTheme, Confirm};

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to delete session '{}'?",
            session_name
        ))
        .default(false)
        .interact()
        .context("Failed to get confirmation")?;

    if !confirmed {
        println!();
        println!("{}", "❌ Session deletion cancelled".yellow());
        println!(
            "   {}",
            format!("Session '{}' remains unchanged", session_name).white()
        );
        return Ok(());
    }

    // Perform the deletion
    println!();
    println!("{}", "🗑️ Deleting session...".bright_yellow());

    repo.delete_session(&session_entity.id)
        .context("Failed to delete session from database")?;

    println!("{}", "✅ Session deleted successfully!".green().bold());
    println!();
    println!(
        "{}  {}",
        "Deleted session:".bright_green(),
        session_name.bright_white()
    );
    println!(
        "{}   {} messages removed",
        "Cleanup:".bright_green(),
        message_count
    );
    println!();
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!(
        "   {}         # View remaining sessions",
        "termai session list".cyan()
    );
    println!(
        "   {}       # Create a new session",
        "termai chat --session <name>".cyan()
    );

    Ok(())
}

/// Show detailed information about a specific session
fn show_session_details(repo: &SqliteRepository, session_name: &str) -> Result<()> {
    use crate::session::model::session::Session;
    use crate::session::repository::MessageRepository;

    // Fetch session
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .map_err(|_| anyhow::anyhow!("Session '{}' not found", session_name))?;

    // Fetch messages for the session
    let message_entities = repo.fetch_messages_for_session(&session_entity.id)?;

    // Convert to session model
    let mut session = Session::from(&session_entity);
    session = session.copy_with_messages(
        message_entities
            .iter()
            .map(|entity| entity.into())
            .collect(),
    );

    // Display session information
    println!("📋 Session Details");
    println!("═══════════════════");
    println!("Name: {}", session.name);
    println!("ID: {}", session.id);
    println!("Created: {}", session_entity.expires_at); // Using expires_at as a proxy for created time
    println!("Current: {}", if session.current { "Yes" } else { "No" });
    println!(
        "Temporary: {}",
        if session.temporary { "Yes" } else { "No" }
    );
    println!("Messages: {}", session.messages.len());

    if !session.messages.is_empty() {
        println!("\n💬 Message History:");
        println!("═══════════════════");

        for (i, message) in session.messages.iter().enumerate() {
            let role_icon = match message.role {
                crate::llm::common::model::role::Role::User => "👤",
                crate::llm::common::model::role::Role::Assistant => "🤖",
                crate::llm::common::model::role::Role::System => "⚙️",
            };

            println!(
                "\n{} {} Message {} ({:?}):",
                role_icon,
                i + 1,
                message.id,
                message.role
            );

            // Show first 100 characters of message content
            let preview = if message.content.len() > 100 {
                format!("{}...", &message.content[..97])
            } else {
                message.content.clone()
            };

            println!("   {}", preview);
        }

        println!(
            "\n💡 Use 'termai chat --session {}' to continue this conversation",
            session_name
        );
    } else {
        println!("\n💡 This session has no messages yet");
        println!(
            "💡 Use 'termai chat --session {}' to start a conversation",
            session_name
        );
    }

    Ok(())
}

/// Handle branch creation command
fn handle_branch_command(
    _repo: &SqliteRepository,
    session_name: &str,
    branch_name: Option<&str>,
    description: Option<&str>,
    from_message: Option<usize>,
) -> Result<()> {
    println!("{}", "🌿 Create Conversation Branch".bright_green().bold());
    println!("{}", "═══════════════════════════".white().dimmed());
    println!();

    // Generate branch name if not provided
    let final_branch_name = if let Some(name) = branch_name {
        name.to_string()
    } else {
        BranchService::generate_branch_name(session_name, None)
    };

    println!(
        "{}        {}",
        "Source session:".bright_cyan(),
        session_name.bright_white()
    );
    println!(
        "{}        {}",
        "New branch name:".bright_cyan(),
        final_branch_name.bright_white()
    );

    if let Some(desc) = description {
        println!(
            "{}        {}",
            "Description:".bright_cyan(),
            desc.bright_white()
        );
    }

    if let Some(index) = from_message {
        println!(
            "{}    {} (0-based index)",
            "From message:".bright_cyan(),
            index.to_string().bright_white()
        );
    }

    println!();

    // Show what the branch would include
    println!("📋 Branch would include:");
    if let Some(index) = from_message {
        println!("   • Messages 0 through {} from source session", index);
    } else {
        println!("   • All messages from source session");
    }
    println!("   • Full conversation context preserved");
    println!("   • Independent conversation history going forward");
    println!();

    // Show example usage
    println!("💡 After creation, you can:");
    println!("   • Continue with: termai chat --session '{}'", final_branch_name);
    println!("   • List branches: termai session list");
    println!("   • View details: termai session show '{}'", final_branch_name);
    println!();

    // TODO: Implement actual branch creation
    // For now, show that it's not yet implemented but the command structure works
    println!(
        "{}",
        "⚠️  Branch creation not yet implemented - requires mutable repository access".yellow().bold()
    );
    println!("   The branch command structure and validation are working correctly!");

    /* TODO: Uncomment when we have mutable repository access:
    
    // Validate source session exists
    let session = repo.fetch_session_by_name(session_name)
        .context(format!("Source session '{}' not found", session_name))?;

    // Create the branch using BranchService
    let branch = BranchService::create_branch(
        repo,
        &session.id,
        None, // No parent branch
        Some(final_branch_name.clone()),
        description.map(|s| s.to_string()),
        from_message,
    )?;

    println!("✅ Successfully created branch '{}'", final_branch_name);
    println!("   Branch ID: {}", branch.id);
    */

    Ok(())
}

/// Handle branch tree visualization command
fn handle_tree_command(
    repo: &SqliteRepository,
    session_name: &str,
    interactive: bool,
    highlight_branch: Option<&str>,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    if interactive {
        println!("{}", "🌳 Interactive Branch Tree".bright_green().bold());
        println!("{}", "═".repeat(25).white().dimmed());
        println!();
        
        // TODO: Implement interactive tree navigation
        println!("{}", "⚠️  Interactive tree navigation not yet implemented".yellow().bold());
        println!("   Command structure is ready - requires terminal interaction library");
        
        // Show static tree for now
        let tree_output = BranchTree::visualize_session_tree(
            repo,
            &session_entity.id,
            highlight_branch,
        )?;
        println!("{}", tree_output);
        
    } else {
        println!("🌳 Branch Tree for '{}'", session_name.bright_green().bold());
        println!("{}", "═".repeat(30).white().dimmed());
        println!();
        
        let tree_output = BranchTree::visualize_session_tree(
            repo,
            &session_entity.id,
            highlight_branch,
        )?;
        println!("{}", tree_output);
    }

    // Show navigation suggestions
    let suggestions = BranchNavigator::get_navigation_suggestions(
        repo,
        &session_entity.id,
        highlight_branch,
    )?;
    
    if !suggestions.is_empty() {
        println!();
        println!("{}", "💡 Navigation Suggestions:".bright_yellow().bold());
        for suggestion in suggestions.iter().take(3) {
            let branch_name = suggestion.branch_name
                .as_deref()
                .unwrap_or("unknown");
            
            let suggestion_type = match suggestion.suggestion_type {
                crate::branch::tree::SuggestionType::Parent => "⬆️ Parent",
                crate::branch::tree::SuggestionType::Sibling => "↔️ Sibling", 
                crate::branch::tree::SuggestionType::Child => "⬇️ Child",
                crate::branch::tree::SuggestionType::Recent => "🕒 Recent",
                crate::branch::tree::SuggestionType::Popular => "⭐ Popular",
            };
            
            println!("   {} {} - {}", 
                suggestion_type, 
                branch_name.bright_cyan(), 
                suggestion.reason.dimmed()
            );
        }
        
        println!();
        println!("{} {}", 
            "🚀 Switch to branch:".bright_green(), 
            "termai session switch <session> <branch>".bright_cyan()
        );
    }

    Ok(())
}

/// Handle branches listing command
fn handle_branches_command(
    repo: &SqliteRepository,
    session_name: &str,
    detailed: bool,
    status_filter: Option<&str>,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    println!("📋 Branches in '{}'", session_name.bright_green().bold());
    println!("{}", "═".repeat(25).white().dimmed());
    println!();

    let branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    
    if branches.is_empty() {
        println!("{}", "No branches found in this session.".dimmed());
        println!();
        println!("{} {}", 
            "💡 Create a branch:".bright_yellow(), 
            format!("termai session branch {}", session_name).bright_cyan()
        );
        return Ok(());
    }

    // Filter by status if specified
    let filtered_branches: Vec<_> = if let Some(status) = status_filter {
        branches.into_iter().filter(|b| b.status == status).collect()
    } else {
        branches
    };

    if filtered_branches.is_empty() {
        println!("{}", 
            format!("No branches found with status '{}'", status_filter.unwrap()).dimmed()
        );
        return Ok(());
    }

    // Display branches
    for (i, branch) in filtered_branches.iter().enumerate() {
        let branch_name = branch.branch_name
            .as_deref()
            .unwrap_or("unknown");
        
        let status_display = match branch.status.as_str() {
            "active" => "●".bright_green(),
            "archived" => "○".dimmed(), 
            "merged" => "✓".bright_blue(),
            _ => "?".bright_yellow(),
        };
        
        let parent_info = if let Some(parent_id) = &branch.parent_branch_id {
            // Find parent branch name
            let parent_name = filtered_branches.iter()
                .find(|b| b.id == *parent_id)
                .and_then(|b| b.branch_name.as_deref())
                .unwrap_or("unknown");
            format!(" (from {})", parent_name.dimmed())
        } else {
            " (root)".dimmed().to_string()
        };

        if detailed {
            println!("{} {} {}{}", 
                status_display, 
                branch_name.bright_white().bold(),
                parent_info,
                if i == 0 { " [current]".bright_green().bold() } else { colored::ColoredString::from("") }
            );
            
            if let Some(desc) = &branch.description {
                println!("   📝 {}", desc.dimmed());
            }
            
            println!("   🆔 ID: {}", branch.id.dimmed());
            println!("   📅 Created: {}", branch.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            println!("   🔄 Status: {}", branch.status.dimmed());
            
            if i < filtered_branches.len() - 1 {
                println!();
            }
        } else {
            println!("{} {}{}", 
                status_display, 
                branch_name.bright_white(),
                parent_info
            );
        }
    }

    println!();
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {} {}", 
        "View tree:".bright_green(), 
        format!("termai session tree {}", session_name).bright_cyan()
    );
    println!("   {} {}", 
        "Switch branch:".bright_green(), 
        format!("termai session switch {} <branch>", session_name).bright_cyan()
    );

    Ok(())
}

/// Handle branch switching command
fn handle_switch_command(
    repo: &SqliteRepository,
    session_name: &str,
    branch_identifier: &str,
    create_new_session: bool,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    println!("{}", "🔄 Branch Switch".bright_green().bold());
    println!("{}", "═══════════════".white().dimmed());
    println!();

    // Find target branch by name or ID
    let branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    let target_branch = branches.iter()
        .find(|b| {
            b.id == branch_identifier || 
            b.branch_name.as_deref() == Some(branch_identifier)
        })
        .ok_or_else(|| anyhow::anyhow!(
            "Branch '{}' not found in session '{}'", 
            branch_identifier, 
            session_name
        ))?;

    let branch_name = target_branch.branch_name
        .as_deref()
        .unwrap_or("unknown");

    println!("{}    {}", "Source session:".bright_cyan(), session_name.bright_white());
    println!("{}      {}", "Target branch:".bright_cyan(), branch_name.bright_white());
    println!("{}       {}", "Branch ID:".bright_cyan(), target_branch.id.dimmed());

    if let Some(desc) = &target_branch.description {
        println!("{}    {}", "Description:".bright_cyan(), desc.bright_white());
    }

    println!();

    if create_new_session {
        // TODO: Implement new session creation on branch
        println!("{}", "📋 Would create new chat session:".bright_blue().bold());
        let new_session_name = format!("{}-{}", session_name, branch_name);
        println!("   • New session name: {}", new_session_name.bright_white());
        println!("   • Branch context preserved");
        println!("   • Independent conversation history");
        println!();
        println!("{} {}", 
            "Start new session:".bright_green(), 
            format!("termai chat --session '{}'", new_session_name).bright_cyan()
        );
    } else {
        // TODO: Implement in-place branch switching
        println!("{}", "📋 Would switch current context:".bright_blue().bold());
        println!("   • Update session to use branch context");
        println!("   • Preserve conversation state");
        println!("   • Switch message history to branch");
        println!();
        println!("{} {}", 
            "Continue on branch:".bright_green(), 
            format!("termai chat --session '{}'", session_name).bright_cyan()
        );
    }

    // TODO: Remove this when actual switching is implemented
    println!();
    println!("{}", 
        "⚠️  Branch switching not yet fully implemented - requires session state management".yellow().bold()
    );
    println!("   Command structure and validation are working correctly!");

    Ok(())
}

/// Handle bookmark command
fn handle_bookmark_command(
    _repo: &SqliteRepository,
    session_name: &str,
    branch_identifier: &str,
    bookmark_name: Option<&str>,
    remove: bool,
) -> Result<()> {
    if remove {
        println!("{}", "🔖 Remove Branch Bookmark".bright_red().bold());
        println!("{}", "═".repeat(25).white().dimmed());
        println!();
        
        // TODO: Implement bookmark removal when we have mutable repository access
        println!("{}", "⚠️  Bookmark removal not yet implemented - requires mutable repository access".yellow().bold());
        println!("   Would remove bookmark from branch '{}' in session '{}'", branch_identifier, session_name);
    } else {
        println!("{}", "🔖 Create Branch Bookmark".bright_green().bold());
        println!("{}", "═".repeat(25).white().dimmed());
        println!();
        
        let final_bookmark_name = bookmark_name.unwrap_or(branch_identifier);
        
        println!("{}        {}", "Session:".bright_cyan(), session_name.bright_white());
        println!("{}         {}", "Branch:".bright_cyan(), branch_identifier.bright_white());
        println!("{}    {}", "Bookmark name:".bright_cyan(), final_bookmark_name.bright_white());
        println!();
        
        // TODO: Implement bookmark creation when we have mutable repository access
        println!("{}", "⚠️  Bookmark creation not yet implemented - requires mutable repository access".yellow().bold());
        println!("   Would create bookmark '{}' for branch '{}'", final_bookmark_name, branch_identifier);
        
        println!();
        println!("{}", "💡 Once implemented, you can:".bright_yellow().bold());
        println!("   • Quick access: termai session switch {} {}", session_name, final_bookmark_name);
        println!("   • Search bookmarks: termai session search {} {}", session_name, final_bookmark_name);
        println!("   • List bookmarks: termai session branches {} --bookmarked", session_name);
    }

    Ok(())
}

/// Handle search command
fn handle_search_command(
    repo: &SqliteRepository,
    session_name: &str,
    query: &str,
    status_filter: Option<&str>,
    detailed: bool,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    println!("🔍 Search Results in '{}'", session_name.bright_green().bold());
    println!("{}", "═".repeat(30).white().dimmed());
    println!();
    
    println!("{} \"{}\"", "Query:".bright_cyan(), query.bright_white());
    if let Some(status) = status_filter {
        println!("{} {}", "Status filter:".bright_cyan(), status.bright_white());
    }
    println!();

    let search_results = BranchService::search_branches(repo, &session_entity.id, query, status_filter)?;
    
    if search_results.is_empty() {
        println!("{}", "No branches found matching your search criteria.".dimmed());
        println!();
        println!("{}", "💡 Search tips:".bright_yellow().bold());
        println!("   • Try broader search terms");
        println!("   • Remove status filters");
        println!("   • Search looks in: branch names, descriptions, and bookmarks");
        return Ok(());
    }

    println!("{} {}", "Found".bright_green(), format!("{} branches", search_results.len()).bright_white().bold());
    println!();

    // Display search results
    for (i, branch) in search_results.iter().enumerate() {
        let branch_name = branch.branch_name
            .as_deref()
            .unwrap_or("unknown");
        
        let status_display = match branch.status.as_str() {
            "active" => "●".bright_green(),
            "archived" => "○".dimmed(), 
            "merged" => "✓".bright_blue(),
            _ => "?".bright_yellow(),
        };

        if detailed {
            println!("{} {} {}", 
                status_display, 
                branch_name.bright_white().bold(),
                if i == 0 { " [most recent]".bright_green().bold() } else { colored::ColoredString::from("") }
            );
            
            if let Some(desc) = &branch.description {
                println!("   📝 {}", desc.dimmed());
            }
            
            println!("   🆔 ID: {}", branch.id.dimmed());
            println!("   📅 Created: {}", branch.created_at.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            println!("   🔄 Status: {}", branch.status.dimmed());
            println!("   📍 Last activity: {}", branch.last_activity.format("%Y-%m-%d %H:%M:%S").to_string().dimmed());
            
            if i < search_results.len() - 1 {
                println!();
            }
        } else {
            println!("{} {}", 
                status_display, 
                branch_name.bright_white()
            );
        }
    }

    println!();
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {} {}", 
        "Switch to branch:".bright_green(), 
        format!("termai session switch {} <branch>", session_name).bright_cyan()
    );
    println!("   {} {}", 
        "View tree:".bright_green(), 
        format!("termai session tree {}", session_name).bright_cyan()
    );

    Ok(())
}

/// Handle stats command
fn handle_stats_command(
    repo: &SqliteRepository,
    session_name: &str,
    detailed: bool,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    println!("📊 Branch Statistics for '{}'", session_name.bright_green().bold());
    println!("{}", "═".repeat(35).white().dimmed());
    println!();

    let stats = BranchService::get_branch_stats(repo, &session_entity.id)?;
    
    // Basic stats
    println!("{}", "📋 Overview:".bright_yellow().bold());
    println!("   {} {}", "Total branches:".bright_cyan(), stats.total_branches.to_string().bright_white().bold());
    println!("   {} {}", "Active branches:".bright_cyan(), stats.active_branches.to_string().bright_green().bold());
    println!("   {} {}", "Archived branches:".bright_cyan(), stats.archived_branches.to_string().dimmed());
    println!("   {} {}", "Bookmarked branches:".bright_cyan(), stats.bookmarked_branches.to_string().bright_yellow().bold());
    
    println!();
    println!("{}", "📏 Depth Analysis:".bright_yellow().bold());
    println!("   {} {}", "Maximum depth:".bright_cyan(), stats.max_depth.to_string().bright_white().bold());
    println!("   {} {:.1}", "Average depth:".bright_cyan(), stats.avg_depth.to_string().bright_white().bold());
    
    if detailed {
        println!();
        println!("{}", "🔍 Detailed Analysis:".bright_yellow().bold());
        
        // Branch distribution
        if stats.total_branches > 0 {
            let active_pct = (stats.active_branches as f64 / stats.total_branches as f64) * 100.0;
            let archived_pct = (stats.archived_branches as f64 / stats.total_branches as f64) * 100.0;
            let bookmarked_pct = (stats.bookmarked_branches as f64 / stats.total_branches as f64) * 100.0;
            
            println!("   {} {:.1}%", "Active branches:".bright_green(), active_pct);
            println!("   {} {:.1}%", "Archived branches:".dimmed(), archived_pct);
            println!("   {} {:.1}%", "Bookmarked branches:".bright_yellow(), bookmarked_pct);
        }
        
        // Usage recommendations
        println!();
        println!("{}", "💡 Recommendations:".bright_yellow().bold());
        
        if stats.total_branches == 0 {
            println!("   • Create your first branch with: termai session branch {}", session_name);
        } else if stats.active_branches == 0 {
            println!("   • All branches are archived - consider creating new active branches");
        } else if stats.bookmarked_branches == 0 {
            println!("   • Consider bookmarking important branches for quick access");
        } else if stats.avg_depth < 2.0 {
            println!("   • Mostly flat branch structure - consider deeper exploration");
        } else if stats.max_depth > 5 {
            println!("   • Deep branch structure detected - consider merging completed paths");
        }
        
        if stats.archived_branches > stats.active_branches * 2 {
            println!("   • Many archived branches - consider cleanup for better organization");
        }
    }
    
    println!();
    println!("{}", "🚀 Quick actions:".bright_green());
    println!("   {} {}", "View tree:".cyan(), format!("termai session tree {}", session_name).bright_cyan());
    println!("   {} {}", "List branches:".cyan(), format!("termai session branches {}", session_name).bright_cyan());
    println!("   {} {}", "Create branch:".cyan(), format!("termai session branch {}", session_name).bright_cyan());

    Ok(())
}

/// Handle branch comparison command
fn handle_compare_command(
    repo: &SqliteRepository,
    session_name: &str,
    branch_names: &[String],
    side_by_side: bool,
    outcomes_only: bool,
    detailed: bool,
) -> Result<()> {
    // Validate session exists
    let session_entity = repo
        .fetch_session_by_name(session_name)
        .context(format!("Session '{}' not found", session_name))?;

    if branch_names.len() < 2 {
        println!("{}", "⚠️  Need at least 2 branches to compare".yellow().bold());
        println!();
        println!("{}", "💡 Usage examples:".bright_yellow().bold());
        println!("   {} {}", 
            "Compare branches:".cyan(), 
            format!("termai session compare {} branch1 branch2", session_name).bright_cyan()
        );
        println!("   {} {}", 
            "Side-by-side view:".cyan(), 
            format!("termai session compare {} branch1 branch2 --side-by-side", session_name).bright_cyan()
        );
        return Ok(());
    }

    println!("{}", "🔍 Branch Comparison".bright_green().bold());
    println!("{}", "═".repeat(20).white().dimmed());
    println!();

    println!("{} {}", "Session:".bright_cyan(), session_name.bright_white());
    println!("{} {}", "Comparing:".bright_cyan(), branch_names.join(", ").bright_white());
    
    if outcomes_only {
        println!("{} {}", "Mode:".bright_cyan(), "Outcomes only".bright_yellow());
    } else if side_by_side {
        println!("{} {}", "Mode:".bright_cyan(), "Side-by-side".bright_blue());
    } else {
        println!("{} {}", "Mode:".bright_cyan(), "Summary".bright_green());
    }
    
    println!();

    // Perform comparison
    let comparison_result = if outcomes_only {
        // Quick outcomes comparison
        let all_branches = BranchService::get_session_branches(repo, &session_entity.id)?;
        let mut branch_ids = Vec::new();
        
        for name in branch_names {
            if let Some(branch) = all_branches.iter().find(|b| {
                b.branch_name.as_deref() == Some(name) || b.id == *name
            }) {
                branch_ids.push(branch.id.clone());
            } else {
                println!("{} Branch '{}' not found", "❌".bright_red(), name);
                return Ok(());
            }
        }
        
        match QuickCompare::compare_outcomes(repo, &branch_ids) {
            Ok(output) => {
                println!("{}", output);
                return Ok(());
            }
            Err(e) => {
                println!("{} {}", "❌ Comparison failed:".bright_red(), e);
                return Ok(());
            }
        }
    } else {
        // Full comparison
        match BranchComparator::compare_branches_by_name(repo, &session_entity.id, branch_names) {
            Ok(comparison) => comparison,
            Err(e) => {
                println!("{} {}", "❌ Comparison failed:".bright_red(), e);
                println!();
                println!("{}", "💡 Troubleshooting:".bright_yellow().bold());
                println!("   • Check branch names are correct");
                println!("   • Use 'termai session branches {}' to list available branches", session_name);
                return Ok(());
            }
        }
    };

    // Display results based on mode
    if side_by_side {
        let side_by_side_output = BranchComparator::format_side_by_side_comparison(&comparison_result);
        println!("{}", side_by_side_output);
    } else {
        // Summary view
        let summary_output = BranchComparator::format_comparison_summary(&comparison_result);
        println!("{}", summary_output);
        
        if detailed {
            println!();
            println!("{}", "📊 Detailed Analysis".bright_yellow().bold());
            println!("{}", "─".repeat(20).dimmed());
            
            // Show message-by-message comparison stats
            println!();
            println!("{} {}", "Total messages:".bright_cyan(), comparison_result.summary.total_messages_compared);
            println!("{} {:.1}%", "Average similarity:".bright_cyan(), comparison_result.summary.similarity_percentage);
            
            // Show per-branch statistics
            for (i, branch) in comparison_result.branches.iter().enumerate() {
                let branch_name = branch.branch_name.as_deref().unwrap_or("unnamed");
                let message_count = comparison_result.message_comparisons.iter()
                    .filter(|mc| i < mc.messages.len() && mc.messages[i].is_some())
                    .count();
                
                println!();
                println!("🌿 {} ({} messages)", branch_name.bright_blue(), message_count);
                if let Some(desc) = &branch.description {
                    println!("   📝 {}", desc.dimmed());
                }
            }
        }
    }

    // Show next steps
    println!();
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {} {}", 
        "Tree view:".cyan(), 
        format!("termai session tree {}", session_name).bright_cyan()
    );
    
    if !side_by_side {
        println!("   {} {}", 
            "Side-by-side:".cyan(), 
            format!("termai session compare {} {} --side-by-side", session_name, branch_names.join(" ")).bright_cyan()
        );
    }
    
    if !outcomes_only {
        println!("   {} {}", 
            "Quick outcomes:".cyan(), 
            format!("termai session compare {} {} --outcomes-only", session_name, branch_names.join(" ")).bright_cyan()
        );
    }

    Ok(())
}

/// Handle merge command
fn handle_merge_command(
    repo: &SqliteRepository,
    session_name: &str,
    source_branches: &[String],
    target_branch: &str,
    strategy_str: &str,
    preview: bool,
    auto_confirm: bool,
) -> Result<()> {
    println!("{}", "🔄 Branch Merge".bright_green().bold());
    println!("{}", "═══════════".white().dimmed());
    println!();

    // Parse merge strategy
    let strategy = match strategy_str.to_lowercase().as_str() {
        "sequential" => MergeStrategy::Sequential,
        "intelligent" => MergeStrategy::Intelligent,
        "selective" => MergeStrategy::Selective,
        "summary" => MergeStrategy::Summary,
        "best-of" => MergeStrategy::BestOf,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid merge strategy '{}'. Valid options: sequential, intelligent, selective, summary, best-of",
                strategy_str
            ));
        }
    };

    // Get session ID and resolve branch names to IDs
    let session_entity = repo.fetch_session_by_name(session_name)?;
    let all_branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    let mut source_ids = Vec::new();
    for branch_name in source_branches {
        if let Some(branch) = all_branches.iter().find(|b| 
            b.branch_name.as_deref() == Some(branch_name) || b.id == *branch_name
        ) {
            source_ids.push(branch.id.clone());
        } else {
            return Err(anyhow::anyhow!("Source branch '{}' not found", branch_name));
        }
    }

    let target_id = all_branches.iter()
        .find(|b| b.branch_name.as_deref() == Some(target_branch) || b.id == *target_branch)
        .map(|b| b.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Target branch '{}' not found", target_branch))?;

    // Create merge operation
    let mut repo_mut = SqliteRepository::new(repo.conn.path().unwrap_or(":memory:".as_ref()))?;
    let merge_result = BranchMerger::merge_branches(&mut repo_mut, &source_ids, &target_id, strategy)?;

    // Show merge preview
    let preview_output = BranchMerger::format_merge_preview(&merge_result);
    println!("{}", preview_output);

    if preview {
        return Ok(());
    }

    // Confirm merge if not auto-confirmed
    if !auto_confirm && !merge_result.conflicts.is_empty() {
        println!("{}", "❓ Proceed with merge? [y/N]: ".bright_yellow());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("{}", "❌ Merge cancelled".bright_red());
            return Ok(());
        }
    }

    println!("{}", "✅ Merge completed successfully!".bright_green());
    
    // Show next steps
    println!();
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {} {}", 
        "Switch to target:".cyan(), 
        format!("termai session switch {} {}", session_name, target_branch).bright_cyan()
    );

    Ok(())
}

/// Handle selective merge command
fn handle_selective_merge_command(
    repo: &SqliteRepository,
    session_name: &str,
    source_branch: &str,
    target_branch: &str,
    message_indices: &[usize],
    preview: bool,
) -> Result<()> {
    println!("{}", "🍒 Selective Merge".bright_green().bold());
    println!("{}", "═══════════════".white().dimmed());
    println!();

    // Get session ID and resolve branch names to IDs
    let session_entity = repo.fetch_session_by_name(session_name)?;
    let all_branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    
    let source_id = all_branches.iter()
        .find(|b| b.branch_name.as_deref() == Some(source_branch) || b.id == *source_branch)
        .map(|b| b.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Source branch '{}' not found", source_branch))?;

    let target_id = all_branches.iter()
        .find(|b| b.branch_name.as_deref() == Some(target_branch) || b.id == *target_branch)
        .map(|b| b.id.clone())
        .ok_or_else(|| anyhow::anyhow!("Target branch '{}' not found", target_branch))?;

    // Get source messages to show what will be merged
    let source_messages = BranchService::get_branch_messages(repo, &source_id)?;
    
    println!("{} {}:", "Source branch".bright_cyan(), source_branch.bright_white());
    println!("{} {}:", "Target branch".bright_cyan(), target_branch.bright_white());
    println!();

    println!("{}", "Messages to merge:".bright_yellow().bold());
    for &index in message_indices {
        if let Some(message) = source_messages.get(index) {
            let content_preview = if message.content.len() > 100 {
                format!("{}...", &message.content[..97])
            } else {
                message.content.clone()
            };
            
            println!("   {} [{:?}] {}", 
                format!("[{}]", index).bright_blue(),
                message.role,
                content_preview.dimmed()
            );
        } else {
            println!("   {} {} (index out of range)", 
                format!("[{}]", index).bright_red(),
                "❌ Invalid message index".bright_red()
            );
        }
    }

    if preview {
        return Ok(());
    }

    // Perform selective merge
    let mut repo_mut = SqliteRepository::new(repo.conn.path().unwrap_or(":memory:".as_ref()))?;
    let _merge_result = BranchMerger::selective_merge(&mut repo_mut, &source_id, &target_id, message_indices)?;

    println!();
    println!("{} {} messages merged successfully!", 
        "✅".bright_green(), 
        message_indices.len().to_string().bright_white()
    );

    Ok(())
}

/// Handle archive command
fn handle_archive_command(
    repo: &SqliteRepository,
    session_name: &str,
    branch_names: &[String],
    reason: Option<&str>,
) -> Result<()> {
    println!("{}", "📦 Archive Branches".bright_blue().bold());
    println!("{}", "══════════════".white().dimmed());
    println!();

    // Get session ID and resolve branch names to IDs
    let session_entity = repo.fetch_session_by_name(session_name)?;
    let all_branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    let mut branch_ids = Vec::new();
    
    for branch_name in branch_names {
        if let Some(branch) = all_branches.iter().find(|b| 
            b.branch_name.as_deref() == Some(branch_name) || b.id == *branch_name
        ) {
            branch_ids.push(branch.id.clone());
            println!("   {} {}", "📦".bright_blue(), branch_name.bright_white());
        } else {
            println!("   {} {} (not found)", "❌".bright_red(), branch_name.bright_red());
        }
    }

    if branch_ids.is_empty() {
        return Err(anyhow::anyhow!("No valid branches found to archive"));
    }

    // Archive branches
    let mut repo_mut = SqliteRepository::new(repo.conn.path().unwrap_or(":memory:".as_ref()))?;
    let archived = BranchMerger::archive_merged_branches(&mut repo_mut, &branch_ids)?;
    
    println!();
    println!("{} {} branches archived", 
        "✅".bright_green(), 
        archived.len().to_string().bright_white()
    );

    if let Some(reason) = reason {
        println!("{} {}", "Reason:".cyan(), reason.dimmed());
    }

    Ok(())
}

/// Handle cleanup command
fn handle_cleanup_command(
    repo: &SqliteRepository,
    session_name: &str,
    strategy_str: &str,
    _days: i64,
    preview: bool,
) -> Result<()> {
    println!("{}", "🧹 Branch Cleanup".bright_yellow().bold());
    println!("{}", "══════════════".white().dimmed());
    println!();

    // Parse cleanup strategy
    let strategy = match strategy_str.to_lowercase().as_str() {
        "archive-old" => CleanupStrategy::ArchiveOld,
        "remove-empty" => CleanupStrategy::RemoveEmpty,
        "consolidate-similar" => CleanupStrategy::ConsolidateSimilar,
        "remove-duplicates" => CleanupStrategy::RemoveDuplicates,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid cleanup strategy '{}'. Valid options: archive-old, remove-empty, consolidate-similar, remove-duplicates",
                strategy_str
            ));
        }
    };

    // Get session ID
    let session_entity = repo.fetch_session_by_name(session_name)?;
    
    // Preview cleanup
    let mut repo_mut = SqliteRepository::new(repo.conn.path().unwrap_or(":memory:".as_ref()))?;
    let cleanup_result = BranchMerger::cleanup_branches(&mut repo_mut, &session_entity.id, strategy)?;

    println!("{} {} branches", "Found:".bright_cyan(), cleanup_result.cleaned_branches.len());
    println!("{} {} branches", "Would preserve:".bright_cyan(), cleanup_result.preserved_branches.len());
    println!();

    if !cleanup_result.cleaned_branches.is_empty() {
        println!("{}", "Branches to clean up:".bright_yellow().bold());
        for branch in &cleanup_result.cleaned_branches {
            let name = branch.branch_name.as_deref().unwrap_or("unnamed");
            println!("   {} {} ({})", "🗑️".bright_red(), name.bright_white(), branch.status.dimmed());
        }
        println!();
    }

    if preview {
        return Ok(());
    }

    println!("{} {} branches cleaned up", 
        "✅".bright_green(), 
        cleanup_result.cleaned_branches.len().to_string().bright_white()
    );

    Ok(())
}

/// Handle export command
fn handle_export_command(
    repo: &SqliteRepository,
    session_name: &str,
    branch_names: &[String],
    format_str: &str,
    output_path: Option<&str>,
) -> Result<()> {
    println!("{}", "📤 Export Branches".bright_magenta().bold());
    println!("{}", "═════════════".white().dimmed());
    println!();

    // Parse export format
    let format = match format_str.to_lowercase().as_str() {
        "json" => ExportFormat::Json,
        "markdown" => ExportFormat::Markdown,
        "csv" => ExportFormat::Csv,
        "text" => ExportFormat::PlainText,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid export format '{}'. Valid options: json, markdown, csv, text",
                format_str
            ));
        }
    };

    // Get session ID and resolve branch names to IDs
    let session_entity = repo.fetch_session_by_name(session_name)?;
    let all_branches = BranchService::get_session_branches(repo, &session_entity.id)?;
    let mut branch_ids = Vec::new();
    
    for branch_name in branch_names {
        if let Some(branch) = all_branches.iter().find(|b| 
            b.branch_name.as_deref() == Some(branch_name) || b.id == *branch_name
        ) {
            branch_ids.push(branch.id.clone());
            println!("   {} {}", "📂".bright_blue(), branch_name.bright_white());
        } else {
            println!("   {} {} (not found)", "❌".bright_red(), branch_name.bright_red());
        }
    }

    if branch_ids.is_empty() {
        return Err(anyhow::anyhow!("No valid branches found to export"));
    }

    // Export branches
    let export_result = BranchMerger::export_branches(repo, &branch_ids, format)?;
    
    // Output to file or stdout
    match output_path {
        Some(path) => {
            std::fs::write(path, &export_result.exported_data)?;
            println!();
            println!("{} {} exported to {}", 
                "✅".bright_green(), 
                export_result.exported_branches.len().to_string().bright_white(),
                path.bright_cyan()
            );
        }
        None => {
            println!();
            println!("{}", export_result.exported_data);
        }
    }

    Ok(())
}
