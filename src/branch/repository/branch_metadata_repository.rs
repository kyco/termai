#![allow(dead_code)]
use crate::repository::db::SqliteRepository;
use anyhow::Result;
use rusqlite::params;
use std::collections::HashMap;

pub struct BranchMetadataRepository {
    repo: SqliteRepository,
}

impl BranchMetadataRepository {
    pub fn new(repo: SqliteRepository) -> Self {
        Self { repo }
    }

    /// Set metadata for a branch
    pub fn set_metadata(&mut self, branch_id: &str, key: &str, value: &str) -> Result<()> {
        self.repo.conn.execute(
            "INSERT OR REPLACE INTO branch_metadata (branch_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![branch_id, key, value],
        )?;
        Ok(())
    }

    /// Get metadata value for a branch
    pub fn get_metadata(&self, branch_id: &str, key: &str) -> Result<Option<String>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT value FROM branch_metadata WHERE branch_id = ?1 AND key = ?2"
        )?;

        let mut rows = stmt.query_map(params![branch_id, key], |row| {
            row.get::<_, String>(0)
        })?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Get all metadata for a branch
    pub fn get_all_metadata(&self, branch_id: &str) -> Result<HashMap<String, String>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT key, value FROM branch_metadata WHERE branch_id = ?1"
        )?;

        let rows = stmt.query_map(params![branch_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut metadata = HashMap::new();
        for row in rows {
            let (key, value) = row?;
            metadata.insert(key, value);
        }

        Ok(metadata)
    }

    /// Remove metadata key for a branch
    pub fn remove_metadata(&mut self, branch_id: &str, key: &str) -> Result<()> {
        self.repo.conn.execute(
            "DELETE FROM branch_metadata WHERE branch_id = ?1 AND key = ?2",
            params![branch_id, key],
        )?;
        Ok(())
    }

    /// Remove all metadata for a branch
    pub fn clear_branch_metadata(&mut self, branch_id: &str) -> Result<()> {
        self.repo.conn.execute(
            "DELETE FROM branch_metadata WHERE branch_id = ?1",
            params![branch_id],
        )?;
        Ok(())
    }

    /// Check if metadata key exists for a branch
    pub fn metadata_exists(&self, branch_id: &str, key: &str) -> Result<bool> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT COUNT(*) FROM branch_metadata WHERE branch_id = ?1 AND key = ?2"
        )?;

        let count: i32 = stmt.query_row(params![branch_id, key], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Get branches with specific metadata key-value pair
    pub fn find_branches_by_metadata(&self, key: &str, value: &str) -> Result<Vec<String>> {
        let mut stmt = self.repo.conn.prepare(
            "SELECT branch_id FROM branch_metadata WHERE key = ?1 AND value = ?2"
        )?;

        let rows = stmt.query_map(params![key, value], |row| {
            row.get::<_, String>(0)
        })?;

        let mut branch_ids = Vec::new();
        for row in rows {
            branch_ids.push(row?);
        }

        Ok(branch_ids)
    }

    /// Copy metadata from one branch to another
    pub fn copy_metadata(&mut self, from_branch_id: &str, to_branch_id: &str) -> Result<()> {
        let metadata = self.get_all_metadata(from_branch_id)?;
        
        for (key, value) in metadata {
            self.set_metadata(to_branch_id, &key, &value)?;
        }

        Ok(())
    }
}