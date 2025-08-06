use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Entity representing smart context metadata stored with sessions
/// This allows persisting smart context discovery settings and results
/// for reuse across session restarts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContextEntity {
    pub id: String,
    pub session_id: String,
    pub project_path: String,
    pub project_type: String,
    pub max_tokens: Option<usize>,
    pub chunked_analysis: bool,
    pub chunk_strategy: Option<String>,
    pub selected_files: String, // JSON serialized Vec<String>
    pub total_tokens: Option<usize>,
    pub query_hash: Option<String>, // Hash of the query for cache invalidation
    pub config_hash: Option<String>, // Hash of .termai.toml for cache invalidation
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl SmartContextEntity {
    #[allow(dead_code)]
    pub fn new(
        id: String,
        session_id: String,
        project_path: String,
        project_type: String,
        max_tokens: Option<usize>,
        chunked_analysis: bool,
        chunk_strategy: Option<String>,
        selected_files: String,
        total_tokens: Option<usize>,
        query_hash: Option<String>,
        config_hash: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            id,
            session_id,
            project_path,
            project_type,
            max_tokens,
            chunked_analysis,
            chunk_strategy,
            selected_files,
            total_tokens,
            query_hash,
            config_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the entity with new data
    #[allow(dead_code)]
    pub fn update(
        &mut self,
        selected_files: String,
        total_tokens: Option<usize>,
        query_hash: Option<String>,
    ) {
        self.selected_files = selected_files;
        self.total_tokens = total_tokens;
        self.query_hash = query_hash;
        self.updated_at = chrono::Utc::now().naive_utc();
    }
}