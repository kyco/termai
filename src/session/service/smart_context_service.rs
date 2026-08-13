use crate::context::SmartContext;
use crate::session::model::smart_context::SessionSmartContext;
use crate::session::repository::smart_context_repository::{
    create_smart_context_entity, SmartContextRepository,
};
use anyhow::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Service for integrating Smart Context Discovery with session management
/// This service handles persistence of smart context metadata and provides
/// high-level operations for session-aware context discovery
pub struct SmartContextService<R: SmartContextRepository> {
    repository: R,
}

impl<R: SmartContextRepository> SmartContextService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Discover smart context and associate it with a session
    #[allow(clippy::too_many_arguments)]
    pub async fn discover_and_store_context(
        &self,
        session_id: String,
        project_path: &Path,
        query: Option<&str>,
        smart_context: &SmartContext,
        max_tokens: Option<usize>,
        chunked_analysis: bool,
        chunk_strategy: Option<String>,
    ) -> Result<SessionSmartContext> {
        // Generate query hash for caching
        let query_hash = query.map(|q| self.hash_string(q));

        // Check if we have existing valid context for this session
        if let Ok(Some(existing_entity)) = self.repository.find_by_session_id(&session_id) {
            if let Ok(existing_context) = SessionSmartContext::from_entity(&existing_entity) {
                // Check if existing context is still valid
                if existing_context.is_valid_for_query(
                    query_hash.as_deref(),
                    None, // TODO: Add config hash support
                ) && existing_context.project_path == project_path.to_string_lossy()
                {
                    return Ok(existing_context);
                }
            }
        }

        // Discover new context
        let context_files = smart_context.discover_context(project_path, query).await?;

        // Convert to file paths and calculate total tokens
        let selected_files: Vec<String> = context_files.iter().map(|f| f.path.clone()).collect();

        let total_tokens = Some(
            context_files
                .iter()
                .map(|f| smart_context.optimizer.count_tokens(&f.content))
                .sum(),
        );

        // Detect project type
        let project_info = smart_context.detect_project(project_path)?;
        let project_type = project_info
            .map(|info| format!("{:?}", info.project_type))
            .unwrap_or_else(|| "generic".to_string());

        // Create session smart context
        let session_context = SessionSmartContext::new(
            session_id.clone(),
            project_path.to_string_lossy().to_string(),
            project_type.clone(),
            max_tokens,
            chunked_analysis,
            chunk_strategy.clone(),
            selected_files.clone(),
            total_tokens,
            query_hash.clone(),
            None, // TODO: Add config hash
        );

        // Store in database
        let entity = create_smart_context_entity(
            session_id,
            project_path.to_string_lossy().to_string(),
            project_type,
            max_tokens,
            chunked_analysis,
            chunk_strategy,
            selected_files,
            total_tokens,
            query_hash,
            None, // TODO: Add config hash
        )?;

        self.repository.create_smart_context(&entity)?;

        Ok(session_context)
    }

    /// Get existing smart context for a session
    pub fn get_session_context(&self, session_id: &str) -> Result<Option<SessionSmartContext>> {
        match self.repository.find_by_session_id(session_id)? {
            Some(entity) => Ok(Some(SessionSmartContext::from_entity(&entity)?)),
            None => Ok(None),
        }
    }

    /// Update smart context results for a session
    pub fn update_session_context(
        &self,
        session_id: &str,
        selected_files: Vec<String>,
        total_tokens: Option<usize>,
        query: Option<&str>,
    ) -> Result<()> {
        if let Some(mut entity) = self.repository.find_by_session_id(session_id)? {
            let query_hash = query.map(|q| self.hash_string(q));

            entity.update(
                serde_json::to_string(&selected_files)?,
                total_tokens,
                query_hash,
            );

            self.repository.update_smart_context(&entity)?;
        }

        Ok(())
    }

    /// Delete smart context for a session
    pub fn delete_session_context(&self, session_id: &str) -> Result<()> {
        self.repository.delete_by_session_id(session_id)
    }

    /// Find sessions that used smart context for a specific project
    pub fn find_sessions_for_project(
        &self,
        project_path: &Path,
    ) -> Result<Vec<SessionSmartContext>> {
        let entities = self
            .repository
            .find_by_project_path(&project_path.to_string_lossy())?;

        let mut contexts = Vec::new();
        for entity in entities {
            contexts.push(SessionSmartContext::from_entity(&entity)?);
        }

        Ok(contexts)
    }

    /// Check if a session has valid smart context for a query
    pub fn is_context_valid(
        &self,
        session_id: &str,
        project_path: &Path,
        query: Option<&str>,
    ) -> Result<bool> {
        if let Some(context) = self.get_session_context(session_id)? {
            // Check project path matches
            if context.project_path != project_path.to_string_lossy() {
                return Ok(false);
            }

            // Check query hash matches
            let query_hash = query.map(|q| self.hash_string(q));
            Ok(context.is_valid_for_query(
                query_hash.as_deref(),
                None, // TODO: Add config hash
            ))
        } else {
            Ok(false)
        }
    }

    /// Get context statistics for a project
    pub fn get_project_context_stats(&self, project_path: &Path) -> Result<ContextStats> {
        let contexts = self.find_sessions_for_project(project_path)?;

        let total_sessions = contexts.len();
        let chunked_sessions = contexts.iter().filter(|c| c.chunked_analysis).count();

        let avg_files = if total_sessions > 0 {
            contexts
                .iter()
                .map(|c| c.selected_files.len())
                .sum::<usize>() as f64
                / total_sessions as f64
        } else {
            0.0
        };

        let avg_tokens = if total_sessions > 0 {
            let total_tokens: usize = contexts.iter().filter_map(|c| c.total_tokens).sum();
            total_tokens as f64 / total_sessions as f64
        } else {
            0.0
        };

        // Get project types used
        let mut project_types = std::collections::HashSet::new();
        for context in &contexts {
            project_types.insert(context.project_type.clone());
        }

        Ok(ContextStats {
            total_sessions,
            chunked_sessions,
            average_files: avg_files,
            average_tokens: avg_tokens,
            project_types: project_types.into_iter().collect(),
        })
    }

    /// Helper to create hash of a string
    fn hash_string(&self, input: &str) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Statistics about smart context usage for a project
#[derive(Debug, Clone)]
pub struct ContextStats {
    pub total_sessions: usize,
    pub chunked_sessions: usize,
    pub average_files: f64,
    pub average_tokens: f64,
    pub project_types: Vec<String>,
}

impl ContextStats {
    /// Generate a summary report
    pub fn summary(&self) -> String {
        if self.total_sessions == 0 {
            return "No smart context sessions found for this project.".to_string();
        }

        let chunked_percentage = if self.total_sessions > 0 {
            (self.chunked_sessions as f64 / self.total_sessions as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "Smart Context Stats:\n\
             • Total sessions: {}\n\
             • Chunked sessions: {} ({:.1}%)\n\
             • Average files per session: {:.1}\n\
             • Average tokens per session: {:.0}\n\
             • Project types: {}",
            self.total_sessions,
            self.chunked_sessions,
            chunked_percentage,
            self.average_files,
            self.average_tokens,
            self.project_types.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::entity::smart_context_entity::SmartContextEntity;
    #[allow(unused_imports)]
    use crate::session::repository::smart_context_repository::SmartContextRepositoryImpl;
    #[allow(unused_imports)]
    use rusqlite::Connection;
    use std::path::PathBuf;
    #[allow(unused_imports)]
    use tempfile::TempDir;

    // Mock repository for testing
    struct MockSmartContextRepository {
        entities: std::sync::Mutex<Vec<SmartContextEntity>>,
    }

    impl MockSmartContextRepository {
        fn new() -> Self {
            Self {
                entities: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    impl SmartContextRepository for MockSmartContextRepository {
        fn create_smart_context(&self, entity: &SmartContextEntity) -> Result<()> {
            let mut entities = self.entities.lock().unwrap();
            entities.push(entity.clone());
            Ok(())
        }

        fn find_by_session_id(&self, session_id: &str) -> Result<Option<SmartContextEntity>> {
            let entities = self.entities.lock().unwrap();
            let found = entities
                .iter()
                .find(|e| e.session_id == session_id)
                .cloned();
            Ok(found)
        }

        fn update_smart_context(&self, entity: &SmartContextEntity) -> Result<()> {
            let mut entities = self.entities.lock().unwrap();
            if let Some(index) = entities.iter().position(|e| e.id == entity.id) {
                entities[index] = entity.clone();
            }
            Ok(())
        }

        fn delete_by_session_id(&self, session_id: &str) -> Result<()> {
            let mut entities = self.entities.lock().unwrap();
            entities.retain(|e| e.session_id != session_id);
            Ok(())
        }

        fn find_by_project_path(&self, project_path: &str) -> Result<Vec<SmartContextEntity>> {
            let entities = self.entities.lock().unwrap();
            let found = entities
                .iter()
                .filter(|e| e.project_path == project_path)
                .cloned()
                .collect();
            Ok(found)
        }
    }

    #[test]
    fn test_hash_string() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        let hash1 = service.hash_string("test query");
        let hash2 = service.hash_string("test query");
        let hash3 = service.hash_string("different query");

        // Same strings should have same hash
        assert_eq!(hash1, hash2);
        // Different strings should have different hashes
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_context_validity_check() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        let project_path = PathBuf::from("/test/project");

        // No context exists initially
        let is_valid = service
            .is_context_valid("session1", &project_path, Some("query"))
            .unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_get_session_context() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        // No context initially
        let context = service.get_session_context("session1").unwrap();
        assert!(context.is_none());
    }

    #[test]
    fn test_project_context_stats_empty() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        let project_path = PathBuf::from("/empty/project");
        let stats = service.get_project_context_stats(&project_path).unwrap();

        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.chunked_sessions, 0);
        assert_eq!(stats.average_files, 0.0);
        assert_eq!(stats.average_tokens, 0.0);
        assert!(stats.project_types.is_empty());

        let summary = stats.summary();
        assert!(summary.contains("No smart context sessions"));
    }

    #[test]
    fn test_update_session_context() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        // Update should work even if no entity exists (graceful handling)
        let result = service.update_session_context(
            "nonexistent_session",
            vec!["file1.rs".to_string()],
            Some(1000),
            Some("test query"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_session_context() {
        let repo = MockSmartContextRepository::new();
        let service = SmartContextService::new(repo);

        // Delete should work even if no entity exists
        let result = service.delete_session_context("nonexistent_session");
        assert!(result.is_ok());
    }
}
