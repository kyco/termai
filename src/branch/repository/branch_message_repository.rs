#![allow(dead_code)]
use crate::branch::entity::branch_message_entity::BranchMessageEntity;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use rusqlite::params;

pub struct BranchMessageRepository {
    repo: SqliteRepository,
}

impl BranchMessageRepository {
    pub fn new(repo: SqliteRepository) -> Self {
        Self { repo }
    }

    /// Add message to branch
    pub fn add_message_to_branch(&mut self, branch_message: &BranchMessageEntity) -> Result<()> {
        self.repo.conn.execute(
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

    /// Get messages for a branch in sequence order
    pub fn get_branch_messages(&self, branch_id: &str) -> Result<Vec<BranchMessageEntity>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT id, branch_id, message_id, sequence_number
             FROM branch_messages 
             WHERE branch_id = ?1 
             ORDER BY sequence_number ASC"
        )?;

        let rows = stmt.query_map(params![branch_id], |row| {
            Ok(BranchMessageEntity {
                id: row.get(0)?,
                branch_id: row.get(1)?,
                message_id: row.get(2)?,
                sequence_number: row.get(3)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
    }

    /// Get message IDs for a branch in sequence order
    pub fn get_branch_message_ids(&self, branch_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.repo.conn.prepare(
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

    /// Remove message from branch
    pub fn remove_message_from_branch(&mut self, branch_id: &str, message_id: &str) -> Result<()> {
        self.repo.conn.execute(
            "DELETE FROM branch_messages WHERE branch_id = ?1 AND message_id = ?2",
            params![branch_id, message_id],
        )?;

        // Resequence remaining messages
        self.resequence_branch_messages(branch_id)?;
        Ok(())
    }

    /// Clear all messages from a branch
    pub fn clear_branch_messages(&mut self, branch_id: &str) -> Result<()> {
        self.repo.conn.execute(
            "DELETE FROM branch_messages WHERE branch_id = ?1",
            params![branch_id],
        )?;
        Ok(())
    }

    /// Get the next sequence number for a branch
    pub fn get_next_sequence_number(&self, branch_id: &str) -> Result<i32> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT COALESCE(MAX(sequence_number), 0) + 1 FROM branch_messages WHERE branch_id = ?1"
        )?;

        let next_seq: i32 = stmt.query_row(params![branch_id], |row| row.get(0))?;
        Ok(next_seq)
    }

    /// Resequence messages in a branch (fix gaps in sequence)
    fn resequence_branch_messages(&mut self, branch_id: &str) -> Result<()> {
        let messages = self.get_branch_messages(branch_id)?;
        
        for (index, message) in messages.iter().enumerate() {
            let new_sequence = index as i32 + 1;
            if message.sequence_number != new_sequence {
                self.repo.conn.execute(
                    "UPDATE branch_messages SET sequence_number = ?1 WHERE id = ?2",
                    params![new_sequence, message.id],
                )?;
            }
        }

        Ok(())
    }

    /// Check if message exists in branch
    pub fn message_exists_in_branch(&self, branch_id: &str, message_id: &str) -> Result<bool> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT COUNT(*) FROM branch_messages WHERE branch_id = ?1 AND message_id = ?2"
        )?;

        let count: i32 = stmt.query_row(params![branch_id, message_id], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Get branches containing a specific message
    pub fn get_branches_for_message(&self, message_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT DISTINCT branch_id FROM branch_messages WHERE message_id = ?1"
        )?;

        let rows = stmt.query_map(params![message_id], |row| {
            row.get::<_, String>(0)
        })?;

        let mut branch_ids = Vec::new();
        for row in rows {
            branch_ids.push(row?);
        }

        Ok(branch_ids)
    }

    /// Copy messages from one branch to another
    pub fn copy_messages_to_branch(&mut self, from_branch_id: &str, to_branch_id: &str) -> Result<()> {
        let messages = self.get_branch_messages(from_branch_id)?;
        let next_seq = self.get_next_sequence_number(to_branch_id)?;

        for (index, message) in messages.iter().enumerate() {
            let new_branch_message = BranchMessageEntity::new(
                uuid::Uuid::new_v4().to_string(),
                to_branch_id.to_string(),
                message.message_id.clone(),
                next_seq + index as i32,
            );
            self.add_message_to_branch(&new_branch_message)?;
        }

        Ok(())
    }
}