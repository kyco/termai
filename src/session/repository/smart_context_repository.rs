use crate::common::unique_id;
use crate::session::entity::smart_context_entity::SmartContextEntity;
use anyhow::Result;
use rusqlite::{params, Connection, Row};

/// Repository for managing smart context metadata persistence
/// Stores and retrieves smart context discovery settings and results
/// associated with chat sessions
pub trait SmartContextRepository {
    fn create_smart_context(&self, entity: &SmartContextEntity) -> Result<()>;
    fn find_by_session_id(&self, session_id: &str) -> Result<Option<SmartContextEntity>>;
    fn update_smart_context(&self, entity: &SmartContextEntity) -> Result<()>;
    fn delete_by_session_id(&self, session_id: &str) -> Result<()>;
    fn find_by_project_path(&self, project_path: &str) -> Result<Vec<SmartContextEntity>>;
}

pub struct SmartContextRepositoryImpl {
    connection: Connection,
}

impl SmartContextRepositoryImpl {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    /// Initialize the smart_context table if it doesn't exist
    pub fn initialize(&self) -> Result<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS smart_context (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                project_path TEXT NOT NULL,
                project_type TEXT NOT NULL,
                max_tokens INTEGER,
                chunked_analysis INTEGER NOT NULL DEFAULT 0,
                chunk_strategy TEXT,
                selected_files TEXT NOT NULL,
                total_tokens INTEGER,
                query_hash TEXT,
                config_hash TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indices for common query patterns
        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_smart_context_session_id ON smart_context(session_id)",
            [],
        )?;

        self.connection.execute(
            "CREATE INDEX IF NOT EXISTS idx_smart_context_project_path ON smart_context(project_path)",
            [],
        )?;

        Ok(())
    }

    fn row_to_entity(row: &Row) -> Result<SmartContextEntity, rusqlite::Error> {
        let created_at_str: String = row.get("created_at")?;
        let updated_at_str: String = row.get("updated_at")?;

        let created_at =
            chrono::NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

        let updated_at =
            chrono::NaiveDateTime::parse_from_str(&updated_at_str, "%Y-%m-%d %H:%M:%S%.f")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

        Ok(SmartContextEntity {
            id: row.get("id")?,
            session_id: row.get("session_id")?,
            project_path: row.get("project_path")?,
            project_type: row.get("project_type")?,
            max_tokens: row.get("max_tokens")?,
            chunked_analysis: row.get::<_, i32>("chunked_analysis")? == 1,
            chunk_strategy: row.get("chunk_strategy")?,
            selected_files: row.get("selected_files")?,
            total_tokens: row.get("total_tokens")?,
            query_hash: row.get("query_hash")?,
            config_hash: row.get("config_hash")?,
            created_at,
            updated_at,
        })
    }
}

impl SmartContextRepository for SmartContextRepositoryImpl {
    fn create_smart_context(&self, entity: &SmartContextEntity) -> Result<()> {
        let created_at_str = entity.created_at.format("%Y-%m-%d %H:%M:%S%.f").to_string();
        let updated_at_str = entity.updated_at.format("%Y-%m-%d %H:%M:%S%.f").to_string();

        self.connection.execute(
            "INSERT INTO smart_context (
                id, session_id, project_path, project_type, max_tokens,
                chunked_analysis, chunk_strategy, selected_files, total_tokens,
                query_hash, config_hash, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                entity.id,
                entity.session_id,
                entity.project_path,
                entity.project_type,
                entity.max_tokens,
                if entity.chunked_analysis { 1 } else { 0 },
                entity.chunk_strategy,
                entity.selected_files,
                entity.total_tokens,
                entity.query_hash,
                entity.config_hash,
                created_at_str,
                updated_at_str
            ],
        )?;

        Ok(())
    }

    fn find_by_session_id(&self, session_id: &str) -> Result<Option<SmartContextEntity>> {
        let mut stmt = self.connection.prepare(
            "SELECT * FROM smart_context WHERE session_id = ?1 ORDER BY updated_at DESC LIMIT 1",
        )?;

        let mut rows = stmt.query_map(params![session_id], Self::row_to_entity)?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    fn update_smart_context(&self, entity: &SmartContextEntity) -> Result<()> {
        let updated_at_str = entity.updated_at.format("%Y-%m-%d %H:%M:%S%.f").to_string();

        self.connection.execute(
            "UPDATE smart_context SET
                selected_files = ?1,
                total_tokens = ?2,
                query_hash = ?3,
                updated_at = ?4
            WHERE id = ?5",
            params![
                entity.selected_files,
                entity.total_tokens,
                entity.query_hash,
                updated_at_str,
                entity.id
            ],
        )?;

        Ok(())
    }

    fn delete_by_session_id(&self, session_id: &str) -> Result<()> {
        self.connection.execute(
            "DELETE FROM smart_context WHERE session_id = ?1",
            params![session_id],
        )?;

        Ok(())
    }

    fn find_by_project_path(&self, project_path: &str) -> Result<Vec<SmartContextEntity>> {
        let mut stmt = self.connection.prepare(
            "SELECT * FROM smart_context WHERE project_path = ?1 ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map(params![project_path], Self::row_to_entity)?;

        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }

        Ok(entities)
    }
}

/// Helper function to create a new SmartContextEntity with generated ID
pub fn create_smart_context_entity(
    session_id: String,
    project_path: String,
    project_type: String,
    max_tokens: Option<usize>,
    chunked_analysis: bool,
    chunk_strategy: Option<String>,
    selected_files: Vec<String>,
    total_tokens: Option<usize>,
    query_hash: Option<String>,
    config_hash: Option<String>,
) -> Result<SmartContextEntity> {
    let id = unique_id::generate_uuid_v4().to_string();
    let selected_files_json = serde_json::to_string(&selected_files)?;

    Ok(SmartContextEntity::new(
        id,
        session_id,
        project_path,
        project_type,
        max_tokens,
        chunked_analysis,
        chunk_strategy,
        selected_files_json,
        total_tokens,
        query_hash,
        config_hash,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    fn create_test_repo() -> (NamedTempFile, SmartContextRepositoryImpl) {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = Connection::open(&temp_db).unwrap();
        let repo = SmartContextRepositoryImpl::new(conn);
        repo.initialize().unwrap();
        (temp_db, repo)
    }

    #[test]
    fn test_create_and_find_smart_context() {
        let (_temp_db, repo) = create_test_repo();

        let entity = create_smart_context_entity(
            "session123".to_string(),
            "/path/to/project".to_string(),
            "rust".to_string(),
            Some(4000),
            false,
            None,
            vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            Some(1500),
            Some("query_hash_123".to_string()),
            Some("config_hash_456".to_string()),
        )
        .unwrap();

        // Create the entity
        repo.create_smart_context(&entity).unwrap();

        // Find it by session ID
        let found = repo.find_by_session_id("session123").unwrap();
        assert!(found.is_some());

        let found_entity = found.unwrap();
        assert_eq!(found_entity.session_id, "session123");
        assert_eq!(found_entity.project_path, "/path/to/project");
        assert_eq!(found_entity.project_type, "rust");
        assert_eq!(found_entity.max_tokens, Some(4000));
        assert_eq!(found_entity.chunked_analysis, false);
        assert_eq!(found_entity.total_tokens, Some(1500));
    }

    #[test]
    fn test_update_smart_context() {
        let (_temp_db, repo) = create_test_repo();

        let mut entity = create_smart_context_entity(
            "session456".to_string(),
            "/path/to/project".to_string(),
            "javascript".to_string(),
            Some(3000),
            true,
            Some("hierarchical".to_string()),
            vec!["index.js".to_string()],
            Some(800),
            Some("old_query_hash".to_string()),
            None,
        )
        .unwrap();

        repo.create_smart_context(&entity).unwrap();

        // Update the entity
        entity.update(
            serde_json::to_string(&vec!["index.js", "utils.js"]).unwrap(),
            Some(1200),
            Some("new_query_hash".to_string()),
        );

        repo.update_smart_context(&entity).unwrap();

        // Verify the update
        let found = repo.find_by_session_id("session456").unwrap().unwrap();
        assert_eq!(found.total_tokens, Some(1200));
        assert_eq!(found.query_hash, Some("new_query_hash".to_string()));

        let selected_files: Vec<String> = serde_json::from_str(&found.selected_files).unwrap();
        assert_eq!(selected_files, vec!["index.js", "utils.js"]);
    }

    #[test]
    fn test_find_by_project_path() {
        let (_temp_db, repo) = create_test_repo();

        // Create multiple entities for the same project path
        let entity1 = create_smart_context_entity(
            "session1".to_string(),
            "/shared/project".to_string(),
            "rust".to_string(),
            Some(4000),
            false,
            None,
            vec!["src/main.rs".to_string()],
            Some(1000),
            None,
            None,
        )
        .unwrap();

        let entity2 = create_smart_context_entity(
            "session2".to_string(),
            "/shared/project".to_string(),
            "rust".to_string(),
            Some(5000),
            true,
            Some("functional".to_string()),
            vec!["src/lib.rs".to_string()],
            Some(2000),
            None,
            None,
        )
        .unwrap();

        repo.create_smart_context(&entity1).unwrap();
        repo.create_smart_context(&entity2).unwrap();

        // Find by project path
        let found = repo.find_by_project_path("/shared/project").unwrap();
        assert_eq!(found.len(), 2);

        // Should be ordered by updated_at DESC (most recent first)
        assert_eq!(found[0].session_id, "session2"); // More recent
        assert_eq!(found[1].session_id, "session1");
    }

    #[test]
    fn test_delete_by_session_id() {
        let (_temp_db, repo) = create_test_repo();

        let entity = create_smart_context_entity(
            "session_to_delete".to_string(),
            "/path/to/project".to_string(),
            "python".to_string(),
            Some(2000),
            false,
            None,
            vec!["main.py".to_string()],
            Some(500),
            None,
            None,
        )
        .unwrap();

        repo.create_smart_context(&entity).unwrap();

        // Verify it exists
        let found = repo.find_by_session_id("session_to_delete").unwrap();
        assert!(found.is_some());

        // Delete it
        repo.delete_by_session_id("session_to_delete").unwrap();

        // Verify it's gone
        let found = repo.find_by_session_id("session_to_delete").unwrap();
        assert!(found.is_none());
    }
}
