#![allow(dead_code)]
use crate::branch::entity::BranchEntity;
use crate::branch::service::BranchService;
use crate::session::model::message::Message;
use crate::repository::db::SqliteRepository;
use anyhow::{Result, Context};

/// High-level branch management operations
pub struct BranchManager;

impl BranchManager {
    /// Create a new branch with intelligent naming
    pub fn create_branch_with_auto_name(
        repo: &mut SqliteRepository,
        session_id: &str,
        parent_branch_id: Option<String>,
        description: Option<String>,
        from_message_index: Option<usize>,
        context_hint: Option<&str>,
    ) -> Result<BranchEntity> {
        let branch_name = BranchService::generate_branch_name(session_id, context_hint);
        
        BranchService::create_branch(
            repo,
            session_id,
            parent_branch_id,
            Some(branch_name),
            description,
            from_message_index,
        )
    }

    /// Create a branch for exploring alternatives
    pub fn create_exploration_branch(
        repo: &mut SqliteRepository,
        session_id: &str,
        topic: &str,
        from_message_index: Option<usize>,
    ) -> Result<BranchEntity> {
        let branch_name = format!("explore-{}", topic.replace(" ", "-").to_lowercase());
        let description = format!("Exploring different approaches to {}", topic);
        
        BranchService::create_branch(
            repo,
            session_id,
            None,
            Some(branch_name),
            Some(description),
            from_message_index,
        )
    }

    /// Create a branch for debugging
    pub fn create_debug_branch(
        repo: &mut SqliteRepository,
        session_id: &str,
        issue: &str,
        from_message_index: Option<usize>,
    ) -> Result<BranchEntity> {
        let branch_name = format!("debug-{}", issue.replace(" ", "-").to_lowercase());
        let description = format!("Debugging session for {}", issue);
        
        BranchService::create_branch(
            repo,
            session_id,
            None,
            Some(branch_name),
            Some(description),
            from_message_index,
        )
    }

    /// List branches with summaries for a session
    pub fn list_session_branches(repo: &SqliteRepository, session_id: &str) -> Result<Vec<BranchSummary>> {
        let branches = BranchService::get_session_branches(repo, session_id)?;
        let mut summaries = Vec::new();

        for branch in branches {
            let messages = BranchService::get_branch_messages(repo, &branch.id)?;
            
            let summary = BranchSummary {
                id: branch.id.clone(),
                name: branch.display_name(),
                description: branch.description.clone(),
                status: branch.status.clone(),
                message_count: messages.len() as i32,
                child_count: 0, // Could be enhanced to count children
                created_at: branch.created_at,
                last_activity: branch.last_activity,
                is_root: branch.is_root(),
                parent_id: branch.parent_branch_id.clone(),
            };

            summaries.push(summary);
        }

        // Sort by creation time
        summaries.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(summaries)
    }

    /// Get branch with context information
    pub fn get_branch_with_context(repo: &SqliteRepository, branch_id: &str) -> Result<BranchContext> {
        let branch = BranchService::get_branch(repo, branch_id)?
            .context("Branch not found")?;
        
        let messages = BranchService::get_branch_messages(repo, branch_id)?;

        Ok(BranchContext {
            branch,
            messages,
        })
    }
}

/// Context information for a branch
#[derive(Debug)]
pub struct BranchContext {
    pub branch: BranchEntity,
    pub messages: Vec<Message>,
}

/// Summary information for a branch
#[derive(Debug, Clone)]
pub struct BranchSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub message_count: i32,
    pub child_count: i32,
    pub created_at: chrono::NaiveDateTime,
    pub last_activity: chrono::NaiveDateTime,
    pub is_root: bool,
    pub parent_id: Option<String>,
}