#![allow(dead_code)]
use crate::branch::entity::{BranchEntity, BranchMessageEntity};
use crate::session::model::message::Message;
use crate::session::repository::MessageRepository;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use uuid::Uuid;
use chrono::NaiveDateTime;
use rusqlite::params;

/// Branch statistics for a session
#[derive(Debug, Clone)]
pub struct BranchStats {
    pub total_branches: usize,
    pub active_branches: usize,
    pub archived_branches: usize,
    pub bookmarked_branches: usize,
    pub max_depth: usize,
    pub avg_depth: f64,
}

/// Simplified service for managing conversation branches
/// Uses the shared SqliteRepository directly instead of separate repository instances
pub struct BranchService;

impl BranchService {
    /// Create a new conversation branch
    pub fn create_branch(
        repo: &mut SqliteRepository,
        session_id: &str,
        parent_branch_id: Option<String>,
        branch_name: Option<String>,
        description: Option<String>,
        from_message_index: Option<usize>,
    ) -> Result<BranchEntity> {
        let branch_id = Uuid::new_v4().to_string();
        
        let branch = BranchEntity::new(
            branch_id.clone(),
            session_id.to_string(),
            parent_branch_id.clone(),
            branch_name,
            description,
        );

        // Create the branch in database
        Self::create_branch_in_db(repo, &branch)?;

        // Copy messages if needed
        if let Some(parent_id) = &parent_branch_id {
            Self::copy_branch_messages_to_point(repo, parent_id, &branch_id, from_message_index)?;
        } else {
            Self::copy_session_messages_to_point(repo, session_id, &branch_id, from_message_index)?;
        }

        Ok(branch)
    }

    /// Get branch by ID
    pub fn get_branch(repo: &SqliteRepository, branch_id: &str) -> Result<Option<BranchEntity>> {
        let mut stmt = repo.conn.prepare(
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
    pub fn get_session_branches(repo: &SqliteRepository, session_id: &str) -> Result<Vec<BranchEntity>> {
        let mut stmt = repo.conn.prepare(
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

    /// Get messages for a branch
    pub fn get_branch_messages(repo: &SqliteRepository, branch_id: &str) -> Result<Vec<Message>> {
        let message_ids = Self::get_branch_message_ids(repo, branch_id)?;
        let mut messages = Vec::new();

        for message_id in message_ids {
            if let Some(message_entity) = Self::get_message_by_id(repo, &message_id)? {
                messages.push(Message::from(&message_entity));
            }
        }

        Ok(messages)
    }

    /// Add message to branch
    pub fn add_message_to_branch(repo: &mut SqliteRepository, branch_id: &str, message: &Message) -> Result<()> {
        // Save the message first
        let message_entity = message.to_entity("temp_session_for_branch");
        repo.add_message_to_session(&message_entity)?;

        let sequence_number = Self::get_next_sequence_number(repo, branch_id)?;
        let branch_message = BranchMessageEntity::new(
            Uuid::new_v4().to_string(),
            branch_id.to_string(),
            message.id.clone(),
            sequence_number,
        );

        Self::add_message_to_branch_in_db(repo, &branch_message)?;
        Self::update_branch_activity(repo, branch_id)?;

        Ok(())
    }

    /// Generate automatic branch name based on context
    pub fn generate_branch_name(_session_id: &str, context_hint: Option<&str>) -> String {
        let timestamp = chrono::Local::now().format("%m%d-%H%M");
        
        match context_hint {
            Some(hint) => {
                // Clean the hint for use in branch name
                let clean_hint = hint
                    .chars()
                    .take(20)
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
                    .to_lowercase();
                
                if clean_hint.is_empty() {
                    format!("branch-{}", timestamp)
                } else {
                    format!("{}-{}", clean_hint, timestamp)
                }
            }
            None => format!("branch-{}", timestamp),
        }
    }

    // Internal helper methods
    fn create_branch_in_db(repo: &mut SqliteRepository, branch: &BranchEntity) -> Result<()> {
        repo.conn.execute(
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

    fn get_branch_message_ids(repo: &SqliteRepository, branch_id: &str) -> Result<Vec<String>> {
        let mut stmt = repo.conn.prepare(
            "SELECT message_id FROM branch_messages 
             WHERE branch_id = ?1 
             ORDER BY sequence_number ASC"
        )?;

        let rows = stmt.query_map(params![branch_id], |row| {
            row.get::<_, String>(0)
        })?;

        let mut message_ids = Vec::new();
        for row in rows {
            message_ids.push(row?);
        }

        Ok(message_ids)
    }

    fn get_message_by_id(repo: &SqliteRepository, message_id: &str) -> Result<Option<crate::session::entity::message_entity::MessageEntity>> {
        let mut stmt = repo.conn.prepare(
            "SELECT id, session_id, role, content FROM messages WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![message_id], |row| {
            Ok(crate::session::entity::message_entity::MessageEntity::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn get_next_sequence_number(repo: &SqliteRepository, branch_id: &str) -> Result<i32> {
        let mut stmt = repo.conn.prepare(
            "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM branch_messages WHERE branch_id = ?1"
        )?;

        let next_seq: i32 = stmt.query_row(params![branch_id], |row| row.get(0))?;
        Ok(next_seq)
    }

    fn add_message_to_branch_in_db(repo: &mut SqliteRepository, branch_message: &BranchMessageEntity) -> Result<()> {
        repo.conn.execute(
            "INSERT INTO branch_messages (id, branch_id, message_id, sequence_number)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                branch_message.id,
                branch_message.branch_id,
                branch_message.message_id,
                branch_message.sequence_number
            ],
        )?;
        Ok(())
    }

    fn update_branch_activity(repo: &mut SqliteRepository, branch_id: &str) -> Result<()> {
        let now = chrono::Local::now().naive_local();
        repo.conn.execute(
            "UPDATE conversation_branches SET last_activity = ?1 WHERE id = ?2",
            params![now.format("%Y-%m-%d %H:%M:%S%.f").to_string(), branch_id],
        )?;
        Ok(())
    }

    fn copy_session_messages_to_point(
        repo: &mut SqliteRepository,
        session_id: &str,
        new_branch_id: &str,
        up_to_index: Option<usize>,
    ) -> Result<()> {
        let session_messages = repo.fetch_messages_for_session(session_id)?;
        let messages_to_copy: Vec<_> = match up_to_index {
            Some(index) => session_messages.into_iter().take(index + 1).collect(),
            None => session_messages,
        };

        for (sequence, message_entity) in messages_to_copy.iter().enumerate() {
            let branch_message = BranchMessageEntity::new(
                Uuid::new_v4().to_string(),
                new_branch_id.to_string(),
                message_entity.id.clone(),
                sequence as i32 + 1,
            );
            Self::add_message_to_branch_in_db(repo, &branch_message)?;
        }

        Ok(())
    }

    fn copy_branch_messages_to_point(
        repo: &mut SqliteRepository,
        parent_branch_id: &str,
        new_branch_id: &str,
        up_to_index: Option<usize>,
    ) -> Result<()> {
        let parent_message_ids = Self::get_branch_message_ids(repo, parent_branch_id)?;
        let message_ids_to_copy: Vec<_> = match up_to_index {
            Some(index) => parent_message_ids.into_iter().take(index + 1).collect(),
            None => parent_message_ids,
        };

        for (sequence, message_id) in message_ids_to_copy.iter().enumerate() {
            let branch_message = BranchMessageEntity::new(
                Uuid::new_v4().to_string(),
                new_branch_id.to_string(),
                message_id.clone(),
                sequence as i32 + 1,
            );
            Self::add_message_to_branch_in_db(repo, &branch_message)?;
        }

        Ok(())
    }

    fn row_to_branch_entity(row: &rusqlite::Row) -> Result<BranchEntity, rusqlite::Error> {
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

    /// Add a bookmark to a branch for quick access
    pub fn bookmark_branch(
        repo: &mut SqliteRepository, 
        branch_id: &str, 
        bookmark_name: &str
    ) -> Result<()> {
        // Add bookmark metadata
        repo.conn.execute(
            "INSERT OR REPLACE INTO branch_metadata (branch_id, key, value) VALUES (?1, ?2, ?3)",
            params![branch_id, "bookmark", bookmark_name],
        )?;
        
        // Update last activity
        repo.conn.execute(
            "UPDATE conversation_branches SET last_activity = ?1 WHERE id = ?2",
            params![chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.f").to_string(), branch_id],
        )?;
        
        Ok(())
    }
    
    /// Remove a bookmark from a branch
    pub fn remove_bookmark(repo: &mut SqliteRepository, branch_id: &str) -> Result<()> {
        repo.conn.execute(
            "DELETE FROM branch_metadata WHERE branch_id = ?1 AND key = 'bookmark'",
            params![branch_id],
        )?;
        Ok(())
    }
    
    /// Get all bookmarked branches for a session
    pub fn get_bookmarked_branches(repo: &SqliteRepository, session_id: &str) -> Result<Vec<(BranchEntity, String)>> {
        let mut stmt = repo.conn.prepare(
            "SELECT b.id, b.session_id, b.parent_branch_id, b.branch_name, b.description, 
                    b.created_at, b.last_activity, b.status, m.value as bookmark_name
             FROM conversation_branches b 
             INNER JOIN branch_metadata m ON b.id = m.branch_id 
             WHERE b.session_id = ?1 AND m.key = 'bookmark'
             ORDER BY m.value"
        )?;
        
        let branch_iter = stmt.query_map([session_id], |row| {
            let branch = BranchEntity {
                id: row.get(0)?,
                session_id: row.get(1)?,
                parent_branch_id: row.get(2)?,
                branch_name: row.get(3)?,
                description: row.get(4)?,
                created_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(5)?, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                last_activity: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                status: row.get(7)?,
            };
            let bookmark_name: String = row.get(8)?;
            Ok((branch, bookmark_name))
        })?;
        
        let mut branches = Vec::new();
        for branch_result in branch_iter {
            branches.push(branch_result?);
        }
        
        Ok(branches)
    }
    
    /// Search branches by name, description, or bookmark
    pub fn search_branches(
        repo: &SqliteRepository, 
        session_id: &str, 
        query: &str,
        filter_status: Option<&str>
    ) -> Result<Vec<BranchEntity>> {
        let query_pattern = format!("%{}%", query);
        let mut sql = "SELECT DISTINCT b.id, b.session_id, b.parent_branch_id, b.branch_name, b.description, 
                              b.created_at, b.last_activity, b.status
                       FROM conversation_branches b 
                       LEFT JOIN branch_metadata m ON b.id = m.branch_id 
                       WHERE b.session_id = ?1 AND (
                           LOWER(b.branch_name) LIKE LOWER(?2) OR 
                           LOWER(b.description) LIKE LOWER(?2) OR
                           (m.key = 'bookmark' AND LOWER(m.value) LIKE LOWER(?2))
                       )".to_string();
        
        let mut params = vec![session_id.to_string(), query_pattern];
        
        if let Some(status) = filter_status {
            sql.push_str(" AND b.status = ?3");
            params.push(status.to_string());
        }
        
        sql.push_str(" ORDER BY b.last_activity DESC");
        
        let mut stmt = repo.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let branch_iter = stmt.query_map(&param_refs[..], |row| {
            Ok(BranchEntity {
                id: row.get(0)?,
                session_id: row.get(1)?,
                parent_branch_id: row.get(2)?,
                branch_name: row.get(3)?,
                description: row.get(4)?,
                created_at: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(5)?, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                last_activity: chrono::NaiveDateTime::parse_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d %H:%M:%S%.f").unwrap(),
                status: row.get(7)?,
            })
        })?;
        
        let mut branches = Vec::new();
        for branch_result in branch_iter {
            branches.push(branch_result?);
        }
        
        Ok(branches)
    }
    
    /// Get branch statistics for a session
    pub fn get_branch_stats(repo: &SqliteRepository, session_id: &str) -> Result<BranchStats> {
        let branches = Self::get_session_branches(repo, session_id)?;
        
        let total_branches = branches.len();
        let active_branches = branches.iter().filter(|b| b.status == "active").count();
        let archived_branches = branches.iter().filter(|b| b.status == "archived").count();
        let bookmarked_branches = Self::get_bookmarked_branches(repo, session_id)?.len();
        
        let max_depth = Self::calculate_max_branch_depth(&branches);
        let avg_depth = if total_branches > 0 {
            Self::calculate_average_branch_depth(&branches)
        } else {
            0.0
        };
        
        Ok(BranchStats {
            total_branches,
            active_branches,
            archived_branches,
            bookmarked_branches,
            max_depth,
            avg_depth,
        })
    }
    
    fn calculate_max_branch_depth(branches: &[BranchEntity]) -> usize {
        if branches.is_empty() {
            return 0;
        }
        
        // Build parent-child map
        let mut children_map: std::collections::HashMap<String, Vec<&BranchEntity>> = std::collections::HashMap::new();
        for branch in branches {
            if let Some(parent_id) = &branch.parent_branch_id {
                children_map.entry(parent_id.clone()).or_default().push(branch);
            }
        }
        
        // Find root branches
        let root_branches: Vec<&BranchEntity> = branches.iter()
            .filter(|b| b.parent_branch_id.is_none())
            .collect();
        
        // Calculate max depth recursively
        root_branches.iter()
            .map(|root| Self::branch_depth(root, &children_map, 1))
            .max()
            .unwrap_or(0)
    }
    
    fn branch_depth(
        branch: &BranchEntity, 
        children_map: &std::collections::HashMap<String, Vec<&BranchEntity>>,
        current_depth: usize
    ) -> usize {
        if let Some(children) = children_map.get(&branch.id) {
            children.iter()
                .map(|child| Self::branch_depth(child, children_map, current_depth + 1))
                .max()
                .unwrap_or(current_depth)
        } else {
            current_depth
        }
    }
    
    fn calculate_average_branch_depth(branches: &[BranchEntity]) -> f64 {
        if branches.is_empty() {
            return 0.0;
        }
        
        // Build parent-child map
        let mut children_map: std::collections::HashMap<String, Vec<&BranchEntity>> = std::collections::HashMap::new();
        for branch in branches {
            if let Some(parent_id) = &branch.parent_branch_id {
                children_map.entry(parent_id.clone()).or_default().push(branch);
            }
        }
        
        // Calculate depth for each branch
        let total_depth: usize = branches.iter()
            .map(|branch| Self::calculate_branch_depth_from_root(branch, branches))
            .sum();
        
        total_depth as f64 / branches.len() as f64
    }
    
    fn calculate_branch_depth_from_root(branch: &BranchEntity, all_branches: &[BranchEntity]) -> usize {
        let mut depth = 1;
        let mut current_branch = branch;
        
        while let Some(parent_id) = &current_branch.parent_branch_id {
            if let Some(parent) = all_branches.iter().find(|b| b.id == *parent_id) {
                depth += 1;
                current_branch = parent;
            } else {
                break;
            }
        }
        
        depth
    }
}