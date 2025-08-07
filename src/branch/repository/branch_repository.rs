#![allow(dead_code)]
use crate::branch::entity::branch_entity::BranchEntity;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use chrono::NaiveDateTime;
use rusqlite::{params, Row};

pub struct BranchRepository {
    repo: SqliteRepository,
}

impl BranchRepository {
    pub fn new(repo: SqliteRepository) -> Self {
        Self { repo }
    }

    /// Create a new conversation branch
    pub fn create_branch(&mut self, branch: &BranchEntity) -> Result<()> {
        self.repo.conn.execute(
            "INSERT INTO conversation_branches (id, session_id, parent_branch_id, branch_name, description, created_at, last_activity, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                branch.id,
                branch.session_id,
                branch.parent_branch_id,
                branch.branch_name,
                branch.description,
                branch.created_at.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                branch.last_activity.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                branch.status
            ],
        )?;
        Ok(())
    }

    /// Get branch by ID
    pub fn get_branch_by_id(&self, branch_id: &str) -> Result<Option<BranchEntity>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT id, session_id, parent_branch_id, branch_name, description, created_at, last_activity, status
             FROM conversation_branches WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![branch_id], Self::row_to_branch_entity)?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Get all branches for a session
    pub fn get_branches_for_session(&self, session_id: &str) -> Result<Vec<BranchEntity>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT id, session_id, parent_branch_id, branch_name, description, created_at, last_activity, status
             FROM conversation_branches WHERE session_id = ?1 ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![session_id], Self::row_to_branch_entity)?;

        let mut branches = Vec::new();
        for row in rows {
            branches.push(row?);
        }

        Ok(branches)
    }

    /// Get child branches for a parent branch
    pub fn get_child_branches(&self, parent_branch_id: &str) -> Result<Vec<BranchEntity>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT id, session_id, parent_branch_id, branch_name, description, created_at, last_activity, status
             FROM conversation_branches WHERE parent_branch_id = ?1 ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![parent_branch_id], Self::row_to_branch_entity)?;

        let mut branches = Vec::new();
        for row in rows {
            branches.push(row?);
        }

        Ok(branches)
    }

    /// Get root branches (branches with no parent) for a session
    pub fn get_root_branches(&self, session_id: &str) -> Result<Vec<BranchEntity>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT id, session_id, parent_branch_id, branch_name, description, created_at, last_activity, status
             FROM conversation_branches WHERE session_id = ?1 AND parent_branch_id IS NULL ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map(params![session_id], Self::row_to_branch_entity)?;

        let mut branches = Vec::new();
        for row in rows {
            branches.push(row?);
        }

        Ok(branches)
    }

    /// Update branch information
    pub fn update_branch(&mut self, branch: &BranchEntity) -> Result<()> {
        self.repo.conn.execute(
            "UPDATE conversation_branches SET 
             branch_name = ?1, description = ?2, last_activity = ?3, status = ?4
             WHERE id = ?5",
            params![
                branch.branch_name,
                branch.description,
                branch.last_activity.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                branch.status,
                branch.id
            ],
        )?;
        Ok(())
    }

    /// Update branch activity timestamp
    pub fn update_branch_activity(&mut self, branch_id: &str) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        self.repo.conn.execute(
            "UPDATE conversation_branches SET last_activity = ?1 WHERE id = ?2",
            params![now.format("%Y-%m-%d %H:%M:%S%.f").to_string(), branch_id],
        )?;
        Ok(())
    }

    /// Delete branch (and all child branches recursively)
    pub fn delete_branch(&mut self, branch_id: &str) -> Result<()> {
        // First get all child branches
        let children = self.get_child_branches(branch_id)?;
        
        // Recursively delete child branches
        for child in children {
            self.delete_branch(&child.id)?;
        }

        // Delete branch messages
        self.repo.conn.execute(
            "DELETE FROM branch_messages WHERE branch_id = ?1",
            params![branch_id],
        )?;

        // Delete branch metadata
        self.repo.conn.execute(
            "DELETE FROM branch_metadata WHERE branch_id = ?1",
            params![branch_id],
        )?;

        // Delete the branch itself
        self.repo.conn.execute(
            "DELETE FROM conversation_branches WHERE id = ?1",
            params![branch_id],
        )?;

        Ok(())
    }

    /// Get branch tree depth (number of levels from root)
    pub fn get_branch_depth(&self, branch_id: &str) -> Result<i32> {
        let mut depth = 0;
        let mut current_id = Some(branch_id.to_string());

        while let Some(id) = current_id {
            if let Some(branch) = self.get_branch_by_id(&id)? {
                current_id = branch.parent_branch_id;
                depth += 1;
            } else {
                break;
            }
        }

        Ok(depth)
    }

    /// Count messages in a branch
    pub fn count_branch_messages(&self, branch_id: &str) -> Result<i32> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT COUNT(*) FROM branch_messages WHERE branch_id = ?1"
        )?;

        let count: i32 = stmt.query_row(params![branch_id], |row| row.get(0))?;
        Ok(count)
    }

    /// Helper function to convert database row to BranchEntity
    fn row_to_branch_entity(row: &Row) -> Result<BranchEntity, rusqlite::Error> {
        let created_at_str: String = row.get(5)?;
        let last_activity_str: String = row.get(6)?;
        
        Ok(BranchEntity {
            id: row.get(0)?,
            session_id: row.get(1)?,
            parent_branch_id: row.get(2)?,
            branch_name: row.get(3)?,
            description: row.get(4)?,
            created_at: NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_else(|_| chrono::Local::now().naive_local()),
            last_activity: NaiveDateTime::parse_from_str(&last_activity_str, "%Y-%m-%d %H:%M:%S%.f")
                .unwrap_or_else(|_| chrono::Local::now().naive_local()),
            status: row.get(7)?,
        })
    }
}