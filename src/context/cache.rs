use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};

use crate::context::{ProjectInfo, analyzer::FileScore};

/// Cache entry for project analysis results
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectCacheEntry {
    pub project_info: ProjectInfo,
    pub file_scores: Vec<FileScore>,
    pub created_at: SystemTime,
    pub directory_hash: String,
    pub config_hash: String,
}

/// Cache entry for individual file analysis
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileCacheEntry {
    pub file_score: FileScore,
    pub content_hash: String,
    pub analyzed_at: SystemTime,
}

/// Manages caching of context analysis results
pub struct ContextCache {
    cache_dir: PathBuf,
    max_age: Duration,
    project_cache: HashMap<String, ProjectCacheEntry>,
    file_cache: HashMap<String, FileCacheEntry>,
}

impl ContextCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            max_age: Duration::from_secs(24 * 3600), // 24 hours default
            project_cache: HashMap::new(),
            file_cache: HashMap::new(),
        }
    }

    /// Create cache in the standard location
    pub fn default() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine cache directory"))?
            .join("termai")
            .join("context");
        
        fs::create_dir_all(&cache_dir)?;
        Ok(Self::new(cache_dir))
    }

    /// Set maximum age for cache entries
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Load cache from disk
    pub fn load(&mut self) -> Result<()> {
        let project_cache_path = self.cache_dir.join("projects.json");
        if project_cache_path.exists() {
            let content = fs::read_to_string(&project_cache_path)?;
            self.project_cache = serde_json::from_str(&content).unwrap_or_default();
        }

        let file_cache_path = self.cache_dir.join("files.json");
        if file_cache_path.exists() {
            let content = fs::read_to_string(&file_cache_path)?;
            self.file_cache = serde_json::from_str(&content).unwrap_or_default();
        }

        // Clean up expired entries
        self.cleanup_expired();

        Ok(())
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let project_cache_path = self.cache_dir.join("projects.json");
        let project_json = serde_json::to_string_pretty(&self.project_cache)?;
        fs::write(&project_cache_path, project_json)?;

        let file_cache_path = self.cache_dir.join("files.json");
        let file_json = serde_json::to_string_pretty(&self.file_cache)?;
        fs::write(&file_cache_path, file_json)?;

        Ok(())
    }

    /// Get cached project analysis results
    pub fn get_project_analysis(
        &self, 
        project_path: &Path, 
        config_hash: &str
    ) -> Option<&ProjectCacheEntry> {
        let key = self.project_key(project_path);
        
        if let Some(entry) = self.project_cache.get(&key) {
            // Check if cache is still valid
            if self.is_cache_valid(&entry.created_at) && 
               entry.config_hash == config_hash &&
               self.is_directory_unchanged(project_path, &entry.directory_hash) {
                return Some(entry);
            }
        }
        
        None
    }

    /// Cache project analysis results
    pub fn cache_project_analysis(
        &mut self,
        project_path: &Path,
        project_info: ProjectInfo,
        file_scores: Vec<FileScore>,
        config_hash: String,
    ) -> Result<()> {
        let key = self.project_key(project_path);
        let directory_hash = self.calculate_directory_hash(project_path)?;
        
        let entry = ProjectCacheEntry {
            project_info,
            file_scores,
            created_at: SystemTime::now(),
            directory_hash,
            config_hash,
        };
        
        self.project_cache.insert(key, entry);
        Ok(())
    }

    /// Get cached file analysis result
    pub fn get_file_analysis(&self, file_path: &Path) -> Option<&FileCacheEntry> {
        let key = file_path.to_string_lossy().to_string();
        
        if let Some(entry) = self.file_cache.get(&key) {
            if self.is_cache_valid(&entry.analyzed_at) && 
               self.is_file_unchanged(file_path, &entry.content_hash) {
                return Some(entry);
            }
        }
        
        None
    }

    /// Cache file analysis result
    pub fn cache_file_analysis(
        &mut self,
        file_path: &Path,
        file_score: FileScore,
    ) -> Result<()> {
        let key = file_path.to_string_lossy().to_string();
        let content_hash = self.calculate_file_hash(file_path)?;
        
        let entry = FileCacheEntry {
            file_score,
            content_hash,
            analyzed_at: SystemTime::now(),
        };
        
        self.file_cache.insert(key, entry);
        Ok(())
    }

    /// Invalidate cache entries for a specific project
    pub fn invalidate_project(&mut self, project_path: &Path) {
        let key = self.project_key(project_path);
        self.project_cache.remove(&key);
        
        // Also invalidate related file entries
        let project_str = project_path.to_string_lossy().to_string();
        self.file_cache.retain(|file_path, _| {
            !file_path.starts_with(&project_str)
        });
    }

    /// Invalidate cache entry for a specific file
    pub fn invalidate_file(&mut self, file_path: &Path) {
        let key = file_path.to_string_lossy().to_string();
        self.file_cache.remove(&key);
    }

    /// Remove expired cache entries
    fn cleanup_expired(&mut self) {
        let now = SystemTime::now();
        
        self.project_cache.retain(|_, entry| {
            now.duration_since(entry.created_at)
                .map(|age| age < self.max_age)
                .unwrap_or(false)
        });
        
        self.file_cache.retain(|_, entry| {
            now.duration_since(entry.analyzed_at)
                .map(|age| age < self.max_age)
                .unwrap_or(false)
        });
    }

    /// Check if cache entry is still valid by age
    fn is_cache_valid(&self, created_at: &SystemTime) -> bool {
        SystemTime::now()
            .duration_since(*created_at)
            .map(|age| age < self.max_age)
            .unwrap_or(false)
    }

    /// Check if directory structure has changed
    fn is_directory_unchanged(&self, project_path: &Path, cached_hash: &str) -> bool {
        self.calculate_directory_hash(project_path)
            .map(|current_hash| current_hash == cached_hash)
            .unwrap_or(false)
    }

    /// Check if file content has changed
    fn is_file_unchanged(&self, file_path: &Path, cached_hash: &str) -> bool {
        self.calculate_file_hash(file_path)
            .map(|current_hash| current_hash == cached_hash)
            .unwrap_or(false)
    }

    /// Generate a cache key for a project
    fn project_key(&self, project_path: &Path) -> String {
        project_path.to_string_lossy().to_string()
    }

    /// Calculate a hash representing the directory structure and modification times
    fn calculate_directory_hash(&self, project_path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Include important files and their modification times
        let important_files = [
            "Cargo.toml", "package.json", "pyproject.toml", "go.mod",
            "build.gradle", "pom.xml", ".termai.toml"
        ];
        
        for file_name in &important_files {
            let file_path = project_path.join(file_name);
            if file_path.exists() {
                file_name.hash(&mut hasher);
                if let Ok(metadata) = file_path.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        modified.hash(&mut hasher);
                    }
                }
            }
        }
        
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Calculate a hash for a file's content and metadata
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        if let Ok(metadata) = file_path.metadata() {
            metadata.len().hash(&mut hasher);
            if let Ok(modified) = metadata.modified() {
                modified.hash(&mut hasher);
            }
        }
        
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            project_entries: self.project_cache.len(),
            file_entries: self.file_cache.len(),
            cache_dir: self.cache_dir.clone(),
        }
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        self.project_cache.clear();
        self.file_cache.clear();
        
        // Remove cache files
        let project_cache_path = self.cache_dir.join("projects.json");
        if project_cache_path.exists() {
            fs::remove_file(project_cache_path)?;
        }
        
        let file_cache_path = self.cache_dir.join("files.json");
        if file_cache_path.exists() {
            fs::remove_file(file_cache_path)?;
        }
        
        Ok(())
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub project_entries: usize,
    pub file_entries: usize,
    pub cache_dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::context::{ProjectType, analyzer::{FileType, ImportanceFactor}};

    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ContextCache::new(temp_dir.path().to_path_buf());
        
        assert_eq!(cache.project_cache.len(), 0);
        assert_eq!(cache.file_cache.len(), 0);
    }

    #[test]
    fn test_project_cache() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = ContextCache::new(temp_dir.path().to_path_buf());
        
        let project_info = ProjectInfo {
            project_type: ProjectType::Rust,
            root_path: "/test".to_string(),
            entry_points: vec!["main.rs".to_string()],
            important_files: vec!["Cargo.toml".to_string()],
            confidence: 0.9,
        };
        
        let file_scores = vec![FileScore {
            path: "main.rs".to_string(),
            relevance_score: 0.9,
            size_bytes: 100,
            modified_time: SystemTime::now(),
            file_type: FileType::SourceCode,
            importance_factors: vec![ImportanceFactor::EntryPoint],
        }];
        
        // Cache the analysis
        cache.cache_project_analysis(
            Path::new("/test"),
            project_info.clone(),
            file_scores.clone(),
            "config_hash".to_string(),
        ).unwrap();
        
        // Retrieve from cache
        let cached = cache.get_project_analysis(Path::new("/test"), "config_hash");
        assert!(cached.is_some());
        
        let cached_entry = cached.unwrap();
        assert_eq!(cached_entry.project_info.project_type, ProjectType::Rust);
        assert_eq!(cached_entry.file_scores.len(), 1);
    }
}