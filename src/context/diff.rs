use crate::context::analyzer::FileScore;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Context diff detection for incremental updates
/// Tracks changes between context discovery runs to provide efficient incremental updates
#[derive(Debug, Clone)]
pub struct ContextDiff {
    cache_dir: std::path::PathBuf,
}

/// Represents a snapshot of the context state at a specific point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub timestamp: u64,
    pub project_path: String,
    pub query_hash: Option<String>,
    pub config_hash: String,
    pub files: HashMap<String, FileSnapshot>,
    pub selected_files: Vec<String>,
    pub total_tokens: usize,
}

/// Snapshot of a single file's state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: String,
    pub modified_time: u64,
    pub size: u64,
    pub content_hash: String,
    pub relevance_score: f32,
}

/// Types of changes detected between context snapshots
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    RelevanceChanged,
}

/// A detected change in the context
#[derive(Debug, Clone)]
pub struct ContextChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub old_relevance: Option<f32>,
    pub new_relevance: Option<f32>,
    pub impact_score: f32, // 0.0 to 1.0, how significant this change is
}

/// Result of a context diff analysis
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub changes: Vec<ContextChange>,
    pub added_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub relevance_changed_files: Vec<String>,
    pub needs_full_reanalysis: bool,
    pub incremental_update_possible: bool,
}

impl ContextDiff {
    /// Create a new ContextDiff instance
    pub fn new(cache_dir: std::path::PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Create a snapshot of the current context state
    pub fn create_snapshot(
        &self,
        project_path: &Path,
        query_hash: Option<String>,
        config_hash: String,
        file_scores: &[FileScore],
        selected_files: &[String],
        total_tokens: usize,
    ) -> Result<ContextSnapshot> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();

        let mut files = HashMap::new();

        // Create file snapshots for all analyzed files
        for score in file_scores {
            let file_path = Path::new(&score.path);
            if let Ok(metadata) = file_path.metadata() {
                let modified_time = metadata
                    .modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();

                let content_hash = if file_path.exists() && file_path.is_file() {
                    self.calculate_file_hash(&score.path)?
                } else {
                    String::new()
                };

                let snapshot = FileSnapshot {
                    path: score.path.clone(),
                    modified_time,
                    size: metadata.len(),
                    content_hash,
                    relevance_score: score.relevance_score,
                };

                files.insert(score.path.clone(), snapshot);
            }
        }

        Ok(ContextSnapshot {
            timestamp,
            project_path: project_path.to_string_lossy().to_string(),
            query_hash,
            config_hash,
            files,
            selected_files: selected_files.to_vec(),
            total_tokens,
        })
    }

    /// Save a context snapshot to disk
    pub fn save_snapshot(&self, snapshot: &ContextSnapshot) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        
        let snapshot_file = self.cache_dir.join(format!(
            "context_snapshot_{}.json",
            self.project_hash(&snapshot.project_path)
        ));

        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(snapshot_file, json)?;

        Ok(())
    }

    /// Load the most recent context snapshot for a project
    pub fn load_snapshot(&self, project_path: &Path) -> Result<Option<ContextSnapshot>> {
        let snapshot_file = self.cache_dir.join(format!(
            "context_snapshot_{}.json",
            self.project_hash(&project_path.to_string_lossy())
        ));

        if !snapshot_file.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(snapshot_file)?;
        let snapshot: ContextSnapshot = serde_json::from_str(&json)?;

        Ok(Some(snapshot))
    }

    /// Compare current file scores against a previous snapshot to detect changes
    pub fn diff_against_snapshot(
        &self,
        snapshot: &ContextSnapshot,
        current_scores: &[FileScore],
        query_hash: Option<&str>,
        config_hash: &str,
    ) -> Result<DiffResult> {
        let mut changes = Vec::new();
        let mut added_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut relevance_changed_files = Vec::new();

        // Check if query or config changed (would require full reanalysis)
        let query_changed = match (&snapshot.query_hash, query_hash) {
            (Some(old), Some(new)) => old != new,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };

        let config_changed = snapshot.config_hash != config_hash;

        let needs_full_reanalysis = query_changed || config_changed;

        if needs_full_reanalysis {
            return Ok(DiffResult {
                changes,
                added_files,
                modified_files,
                deleted_files,
                relevance_changed_files,
                needs_full_reanalysis: true,
                incremental_update_possible: false,
            });
        }

        // Create maps for efficient lookup
        let current_files: HashMap<String, &FileScore> = current_scores
            .iter()
            .map(|score| (score.path.clone(), score))
            .collect();

        let snapshot_files: HashSet<String> = snapshot.files.keys().cloned().collect();
        let current_file_paths: HashSet<String> = current_files.keys().cloned().collect();

        // Find added files
        for path in current_file_paths.difference(&snapshot_files) {
            added_files.push(path.clone());
            if let Some(score) = current_files.get(path) {
                changes.push(ContextChange {
                    file_path: path.clone(),
                    change_type: ChangeType::Added,
                    old_relevance: None,
                    new_relevance: Some(score.relevance_score),
                    impact_score: score.relevance_score, // New files have impact equal to their relevance
                });
            }
        }

        // Find deleted files
        for path in snapshot_files.difference(&current_file_paths) {
            deleted_files.push(path.clone());
            if let Some(old_snapshot) = snapshot.files.get(path) {
                changes.push(ContextChange {
                    file_path: path.clone(),
                    change_type: ChangeType::Deleted,
                    old_relevance: Some(old_snapshot.relevance_score),
                    new_relevance: None,
                    impact_score: old_snapshot.relevance_score, // Impact based on what we're losing
                });
            }
        }

        // Find modified or relevance-changed files
        for path in current_file_paths.intersection(&snapshot_files) {
            if let (Some(current_score), Some(old_snapshot)) = 
                (current_files.get(path), snapshot.files.get(path)) {
                
                let file_path = Path::new(path);
                let mut file_modified = false;
                let mut relevance_changed = false;

                // Check if file was physically modified
                if let Ok(metadata) = file_path.metadata() {
                    let current_modified = metadata
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let current_size = metadata.len();
                    
                    if current_modified != old_snapshot.modified_time || current_size != old_snapshot.size {
                        file_modified = true;
                        modified_files.push(path.clone());
                    }
                }

                // Check if relevance score changed significantly
                let relevance_threshold = 0.1; // 10% change threshold
                let relevance_diff = (current_score.relevance_score - old_snapshot.relevance_score).abs();
                
                if relevance_diff > relevance_threshold {
                    relevance_changed = true;
                    relevance_changed_files.push(path.clone());
                }

                // Create change entry if anything changed
                if file_modified || relevance_changed {
                    let change_type = if file_modified {
                        ChangeType::Modified
                    } else {
                        ChangeType::RelevanceChanged
                    };

                    let impact_score = if file_modified {
                        // File content changed - impact based on current relevance
                        current_score.relevance_score
                    } else {
                        // Only relevance changed - impact based on the magnitude of change
                        relevance_diff
                    };

                    changes.push(ContextChange {
                        file_path: path.clone(),
                        change_type,
                        old_relevance: Some(old_snapshot.relevance_score),
                        new_relevance: Some(current_score.relevance_score),
                        impact_score,
                    });
                }
            }
        }

        // Determine if incremental update is possible
        // Large numbers of changes or high-impact changes might require full reanalysis
        let total_changes = added_files.len() + deleted_files.len() + modified_files.len();
        let high_impact_changes = changes.iter().filter(|c| c.impact_score > 0.7).count();
        
        let incremental_update_possible = total_changes < 20 && high_impact_changes < 5;

        Ok(DiffResult {
            changes,
            added_files,
            modified_files,
            deleted_files,
            relevance_changed_files,
            needs_full_reanalysis,
            incremental_update_possible,
        })
    }

    /// Get a summary of changes for display to user
    pub fn get_change_summary(&self, diff: &DiffResult) -> String {
        if diff.needs_full_reanalysis {
            return "🔄 Configuration or query changed - full reanalysis required".to_string();
        }

        if diff.changes.is_empty() {
            return "✅ No changes detected - using cached context".to_string();
        }

        let mut summary = String::new();
        summary.push_str("📊 Context Changes Detected:\n");

        if !diff.added_files.is_empty() {
            summary.push_str(&format!("  ➕ {} files added\n", diff.added_files.len()));
        }

        if !diff.modified_files.is_empty() {
            summary.push_str(&format!("  ✏️  {} files modified\n", diff.modified_files.len()));
        }

        if !diff.deleted_files.is_empty() {
            summary.push_str(&format!("  ➖ {} files deleted\n", diff.deleted_files.len()));
        }

        if !diff.relevance_changed_files.is_empty() {
            summary.push_str(&format!("  🎯 {} files with relevance changes\n", diff.relevance_changed_files.len()));
        }

        if diff.incremental_update_possible {
            summary.push_str("✨ Incremental update possible - using optimized context discovery\n");
        } else {
            summary.push_str("🔄 Significant changes detected - performing full reanalysis\n");
        }

        // Show most impactful changes
        let mut high_impact: Vec<_> = diff.changes.iter()
            .filter(|c| c.impact_score > 0.5)
            .collect();
        high_impact.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));

        if !high_impact.is_empty() {
            summary.push_str("\n🎯 High Impact Changes:\n");
            for change in high_impact.iter().take(5) {
                let file_name = Path::new(&change.file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&change.file_path);
                
                let change_icon = match change.change_type {
                    ChangeType::Added => "➕",
                    ChangeType::Modified => "✏️",
                    ChangeType::Deleted => "➖",
                    ChangeType::RelevanceChanged => "🎯",
                };

                summary.push_str(&format!(
                    "  {} {} (impact: {:.1}%)\n",
                    change_icon,
                    file_name,
                    change.impact_score * 100.0
                ));
            }
        }

        summary
    }

    /// Apply incremental updates to a previous file selection
    pub fn apply_incremental_update(
        &self,
        previous_selection: &[String],
        diff: &DiffResult,
        current_scores: &[FileScore],
    ) -> Vec<String> {
        if !diff.incremental_update_possible || diff.needs_full_reanalysis {
            // Fall back to full reselection
            return current_scores.iter().map(|s| s.path.clone()).collect();
        }

        let mut updated_selection: HashSet<String> = previous_selection.iter().cloned().collect();
        let score_map: HashMap<String, f32> = current_scores
            .iter()
            .map(|s| (s.path.clone(), s.relevance_score))
            .collect();

        // Remove deleted files
        for deleted in &diff.deleted_files {
            updated_selection.remove(deleted);
        }

        // Add high-relevance new files
        for added in &diff.added_files {
            if let Some(&relevance) = score_map.get(added) {
                if relevance > 0.6 { // Only add files with decent relevance
                    updated_selection.insert(added.clone());
                }
            }
        }

        // Re-evaluate files with significant relevance changes
        for change in &diff.changes {
            if matches!(change.change_type, ChangeType::RelevanceChanged) {
                if let Some(&new_relevance) = score_map.get(&change.file_path) {
                    if new_relevance > 0.6 {
                        updated_selection.insert(change.file_path.clone());
                    } else if new_relevance < 0.3 {
                        updated_selection.remove(&change.file_path);
                    }
                }
            }
        }

        updated_selection.into_iter().collect()
    }

    /// Calculate hash for a file's content (for change detection)
    fn calculate_file_hash(&self, file_path: &str) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let path = Path::new(file_path);
        if !path.exists() || !path.is_file() {
            return Ok(String::new());
        }

        // For performance, hash file metadata and first/last chunks instead of entire content
        let metadata = path.metadata()?;
        let mut hasher = DefaultHasher::new();
        
        metadata.len().hash(&mut hasher);
        metadata.modified()?.hash(&mut hasher);

        // Hash first and last 1KB of file content for change detection
        if let Ok(content) = fs::read_to_string(path) {
            let content_len = content.len();
            if content_len > 2048 {
                content[..1024].hash(&mut hasher);
                content[content_len - 1024..].hash(&mut hasher);
            } else {
                content.hash(&mut hasher);
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    /// Generate a hash for the project path to use as cache key
    fn project_hash(&self, project_path: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        project_path.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Clean up old snapshots (keep only the most recent N snapshots per project)
    pub fn cleanup_old_snapshots(&self, keep_count: usize) -> Result<()> {
        if !self.cache_dir.exists() {
            return Ok(());
        }

        let mut snapshot_files = Vec::new();
        
        for entry in fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("context_snapshot_") && file_name.ends_with(".json") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            snapshot_files.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        snapshot_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old snapshots beyond keep_count
        for (path, _) in snapshot_files.into_iter().skip(keep_count) {
            let _ = fs::remove_file(path); // Ignore errors for cleanup
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::analyzer::ImportanceFactor;
    use tempfile::TempDir;

    fn create_test_file_score(path: &str, relevance: f32) -> FileScore {
        FileScore {
            path: path.to_string(),
            relevance_score: relevance,
            file_type: crate::context::analyzer::FileType::SourceCode,
            importance_factors: vec![ImportanceFactor::MainModule],
            modified_time: std::time::SystemTime::now(),
            size_bytes: 1000,
        }
    }

    #[test]
    fn test_snapshot_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let project_path = temp_dir.path().join("project");
        let scores = vec![
            create_test_file_score("src/main.rs", 0.9),
            create_test_file_score("src/lib.rs", 0.8),
        ];

        let snapshot = diff.create_snapshot(
            &project_path,
            Some("query_hash_123".to_string()),
            "config_hash_456".to_string(),
            &scores,
            &["src/main.rs".to_string(), "src/lib.rs".to_string()],
            2000,
        );

        assert!(snapshot.is_ok());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.selected_files.len(), 2);
        assert_eq!(snapshot.total_tokens, 2000);
        assert_eq!(snapshot.query_hash, Some("query_hash_123".to_string()));
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let project_path = temp_dir.path().join("project");
        let scores = vec![create_test_file_score("src/main.rs", 0.9)];

        let snapshot = diff.create_snapshot(
            &project_path,
            Some("query_hash".to_string()),
            "config_hash".to_string(),
            &scores,
            &["src/main.rs".to_string()],
            1000,
        ).unwrap();

        // Save snapshot
        let save_result = diff.save_snapshot(&snapshot);
        assert!(save_result.is_ok());

        // Load snapshot
        let loaded = diff.load_snapshot(&project_path).unwrap();
        assert!(loaded.is_some());

        let loaded_snapshot = loaded.unwrap();
        assert_eq!(loaded_snapshot.total_tokens, 1000);
        assert_eq!(loaded_snapshot.query_hash, Some("query_hash".to_string()));
    }

    #[test]
    fn test_diff_detection() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let project_path = temp_dir.path().join("project");

        // Create initial snapshot
        let initial_scores = vec![
            create_test_file_score("src/main.rs", 0.9),
            create_test_file_score("src/lib.rs", 0.8),
        ];

        let snapshot = diff.create_snapshot(
            &project_path,
            Some("query_hash".to_string()),
            "config_hash".to_string(),
            &initial_scores,
            &["src/main.rs".to_string(), "src/lib.rs".to_string()],
            1500,
        ).unwrap();

        // Create current scores with changes
        let current_scores = vec![
            create_test_file_score("src/main.rs", 0.9), // Unchanged
            create_test_file_score("src/lib.rs", 0.6), // Relevance decreased
            create_test_file_score("src/new.rs", 0.7), // New file added
        ];
        // Note: src/utils.rs was deleted (not in current_scores)

        let diff_result = diff.diff_against_snapshot(
            &snapshot,
            &current_scores,
            Some("query_hash"),
            "config_hash",
        ).unwrap();

        assert!(!diff_result.needs_full_reanalysis);
        // Check that we found the new file
        assert!(diff_result.added_files.contains(&"src/new.rs".to_string()));
        
        // Check that we have some changes
        assert!(!diff_result.changes.is_empty());

        // Should have changes for added file and relevance change
        assert!(diff_result.changes.len() >= 2);
        
        let added_change = diff_result.changes.iter()
            .find(|c| c.change_type == ChangeType::Added && c.file_path == "src/new.rs")
            .unwrap();
        assert_eq!(added_change.new_relevance, Some(0.7));
    }

    #[test]
    fn test_query_change_requires_full_reanalysis() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let project_path = temp_dir.path().join("project");
        let scores = vec![create_test_file_score("src/main.rs", 0.9)];

        let snapshot = diff.create_snapshot(
            &project_path,
            Some("old_query".to_string()),
            "config_hash".to_string(),
            &scores,
            &["src/main.rs".to_string()],
            1000,
        ).unwrap();

        let diff_result = diff.diff_against_snapshot(
            &snapshot,
            &scores,
            Some("new_query"), // Different query
            "config_hash",
        ).unwrap();

        assert!(diff_result.needs_full_reanalysis);
        assert!(!diff_result.incremental_update_possible);
    }

    #[test]
    fn test_change_summary_generation() {
        let diff_result = DiffResult {
            changes: vec![
                ContextChange {
                    file_path: "src/main.rs".to_string(),
                    change_type: ChangeType::Added,
                    old_relevance: None,
                    new_relevance: Some(0.9),
                    impact_score: 0.9,
                },
                ContextChange {
                    file_path: "src/lib.rs".to_string(),
                    change_type: ChangeType::Modified,
                    old_relevance: Some(0.7),
                    new_relevance: Some(0.8),
                    impact_score: 0.8,
                },
            ],
            added_files: vec!["src/main.rs".to_string()],
            modified_files: vec!["src/lib.rs".to_string()],
            deleted_files: Vec::new(),
            relevance_changed_files: Vec::new(),
            needs_full_reanalysis: false,
            incremental_update_possible: true,
        };

        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let summary = diff.get_change_summary(&diff_result);
        assert!(summary.contains("Context Changes Detected"));
        assert!(summary.contains("1 files added"));
        assert!(summary.contains("1 files modified"));
        assert!(summary.contains("Incremental update possible"));
        assert!(summary.contains("High Impact Changes"));
    }

    #[test]
    fn test_incremental_update_application() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        let previous_selection = vec!["src/main.rs".to_string(), "src/old.rs".to_string()];
        
        let diff_result = DiffResult {
            changes: Vec::new(),
            added_files: vec!["src/new.rs".to_string()],
            modified_files: Vec::new(),
            deleted_files: vec!["src/old.rs".to_string()],
            relevance_changed_files: Vec::new(),
            needs_full_reanalysis: false,
            incremental_update_possible: true,
        };

        let current_scores = vec![
            create_test_file_score("src/main.rs", 0.9),
            create_test_file_score("src/new.rs", 0.8), // High relevance - should be included
        ];

        let updated_selection = diff.apply_incremental_update(
            &previous_selection,
            &diff_result,
            &current_scores,
        );

        assert!(updated_selection.contains(&"src/main.rs".to_string()));
        assert!(updated_selection.contains(&"src/new.rs".to_string()));
        assert!(!updated_selection.contains(&"src/old.rs".to_string())); // Should be removed
    }

    #[test]
    fn test_file_hash_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        let diff = ContextDiff::new(cache_dir);

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!").unwrap();

        let hash1 = diff.calculate_file_hash(&test_file.to_string_lossy()).unwrap();
        assert!(!hash1.is_empty());

        // Same file should produce same hash
        let hash2 = diff.calculate_file_hash(&test_file.to_string_lossy()).unwrap();
        assert_eq!(hash1, hash2);

        // Modified file should produce different hash
        fs::write(&test_file, "Hello, modified world!").unwrap();
        let hash3 = diff.calculate_file_hash(&test_file.to_string_lossy()).unwrap();
        assert_ne!(hash1, hash3);
    }
}