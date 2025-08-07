#![allow(dead_code)]
use chrono::{Local, NaiveDateTime};

/// Database entity representing a conversation branch
#[derive(Debug, Clone)]
pub struct BranchEntity {
    pub id: String,
    pub session_id: String,
    pub parent_branch_id: Option<String>,
    pub branch_name: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_activity: NaiveDateTime,
    pub status: String,
}

impl BranchEntity {
    pub fn new(
        id: String,
        session_id: String,
        parent_branch_id: Option<String>,
        branch_name: Option<String>,
        description: Option<String>,
    ) -> Self {
        let now = Local::now().naive_local();
        Self {
            id,
            session_id,
            parent_branch_id,
            branch_name,
            description,
            created_at: now,
            last_activity: now,
            status: "active".to_string(),
        }
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Local::now().naive_local();
    }

    /// Mark branch as archived
    pub fn archive(&mut self) {
        self.status = "archived".to_string();
        self.touch();
    }

    /// Mark branch as merged
    pub fn mark_merged(&mut self) {
        self.status = "merged".to_string();
        self.touch();
    }

    /// Check if branch is active
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    /// Check if branch is the root branch (has no parent)
    pub fn is_root(&self) -> bool {
        self.parent_branch_id.is_none()
    }

    /// Get display name (branch_name or generated from id)
    pub fn display_name(&self) -> String {
        self.branch_name
            .clone()
            .unwrap_or_else(|| format!("branch-{}", &self.id[..8]))
    }
}