#[allow(dead_code)]
pub mod analyzer;
#[allow(dead_code)]
pub mod cache;
#[allow(dead_code)]
pub mod chunker;
#[allow(dead_code)]
pub mod config;
#[allow(dead_code)]
pub mod detector;
#[allow(dead_code)]
pub mod diff;
#[allow(dead_code)]
pub mod multi_session;
#[allow(dead_code)]
pub mod optimizer;
#[allow(dead_code)]
pub mod template_manager;
#[allow(dead_code)]
pub mod templates;

use crate::path::model::Files;
use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Represents different types of projects that can be detected
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProjectType {
    Rust,
    JavaScript,
    Python,
    Go,
    Kotlin,
    Java,
    Git,
    Generic,
}

/// Information about a detected project
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct ProjectInfo {
    pub project_type: ProjectType,
    pub root_path: String,
    pub entry_points: Vec<String>,
    pub important_files: Vec<String>,
    pub confidence: f32, // 0.0 to 1.0
}

/// Main smart context discovery interface
pub struct SmartContext {
    detectors: Vec<Box<dyn detector::ProjectDetector>>,
    pub analyzer: analyzer::FileAnalyzer,
    pub optimizer: optimizer::TokenOptimizer,
    config: config::ContextConfig,
    cache: Option<cache::ContextCache>,
}

impl SmartContext {
    pub fn new() -> Self {
        Self::with_config(config::ContextConfig::default())
    }

    pub fn with_config(config: config::ContextConfig) -> Self {
        // Initialize with all available project detectors
        let detectors: Vec<Box<dyn detector::ProjectDetector>> = vec![
            Box::new(detector::RustProjectDetector::new()),
            Box::new(detector::JavaScriptProjectDetector::new()),
            Box::new(detector::PythonProjectDetector::new()),
            Box::new(detector::GoProjectDetector::new()),
            Box::new(detector::KotlinProjectDetector::new()),
            Box::new(detector::JavaProjectDetector::new()),
            Box::new(detector::GitProjectDetector::new()),
        ];

        let optimizer_config = optimizer::OptimizationConfig {
            max_tokens: config.context.max_tokens,
            strategy: optimizer::OptimizationStrategy::Truncate,
            preserve_signatures: true,
            preserve_imports: true,
        };

        // Initialize cache if enabled
        let cache = if config.context.enable_cache.unwrap_or(true) {
            cache::ContextCache::default().ok()
        } else {
            None
        };

        Self {
            detectors,
            analyzer: analyzer::FileAnalyzer::new(),
            optimizer: optimizer::TokenOptimizer::with_config(optimizer_config),
            config,
            cache,
        }
    }

    /// Create SmartContext from project directory (loads .termai.toml if exists)
    pub fn from_project(project_path: &Path) -> Result<Self> {
        let config = config::ContextConfig::discover_config(project_path)?;
        Ok(Self::with_config(config))
    }

    /// Discover relevant context for a given directory
    #[allow(dead_code)]
    pub async fn discover_context(&self, path: &Path, query: Option<&str>) -> Result<Vec<Files>> {
        // Generate config hash for cache validation
        let config_hash = self.calculate_config_hash();

        // Try to get cached result first
        if let Some(cache) = &self.cache {
            if let Some(cached_entry) = cache.get_project_analysis(path, &config_hash) {
                // Apply query filtering to cached results
                let filtered_scores = self
                    .analyzer
                    .filter_by_query(&cached_entry.file_scores, query);
                let selected_scores = self.optimizer.optimize_files(&filtered_scores)?;
                return self.scores_to_files(&selected_scores).await;
            }
        }

        // Step 1: Detect project type
        let project_info = self.detect_project(path)?;

        // Step 2: Collect all relevant files
        let candidate_files = self.collect_candidate_files(path, &project_info)?;

        // Step 3: Analyze and score files
        let file_refs: Vec<&Path> = candidate_files.iter().map(|pb| pb.as_path()).collect();
        let mut file_scores = self.analyzer.analyze_files(&file_refs)?;

        // Step 3.5: Analyze dependencies and enhance scores
        self.analyzer.analyze_dependencies(&mut file_scores)?;

        // Note: Cache writing would require a mutable reference,
        // so we'll implement it when we restructure the API later

        // Step 4: Filter by query if provided
        let filtered_scores = self.analyzer.filter_by_query(&file_scores, query);

        // Step 5: Optimize selection based on token limits
        let selected_scores = self.optimizer.optimize_files(&filtered_scores)?;

        // Step 6: Convert to Files format and read content
        self.scores_to_files(&selected_scores).await
    }

    /// Get project information for a directory
    pub fn detect_project(&self, path: &Path) -> Result<Option<ProjectInfo>> {
        for detector in &self.detectors {
            if let Some(info) = detector.detect(path)? {
                return Ok(Some(info));
            }
        }
        Ok(None)
    }

    /// Collect candidate files from the project directory
    pub fn collect_candidate_files(
        &self,
        path: &Path,
        project_info: &Option<ProjectInfo>,
    ) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        // Start with important files from project detection
        if let Some(info) = project_info {
            for file_path in &info.important_files {
                let full_path = path.join(file_path);
                if full_path.exists() && full_path.is_file() {
                    files.push(full_path);
                }
            }

            for entry_point in &info.entry_points {
                let full_path = path.join(entry_point);
                if full_path.exists() && full_path.is_file() {
                    files.push(full_path);
                }
            }
        }

        // Walk the directory tree to find additional relevant files
        for entry in WalkDir::new(path)
            .follow_links(false)
            .max_depth(10) // Reasonable depth limit
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path().to_path_buf();

            // Skip if already included (use canonical paths for comparison)
            let canonical_path = file_path.canonicalize().unwrap_or(file_path.clone());
            if files.iter().any(|existing| {
                existing.canonicalize().unwrap_or_else(|_| existing.clone()) == canonical_path
            }) {
                continue;
            }

            // Apply basic filtering
            if self.should_include_file(&file_path) {
                files.push(file_path);
            }
        }

        Ok(files)
    }

    /// Determine if a file should be included in analysis
    fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Use config-based filtering
        if !self.config.should_include(&path_str) {
            return false;
        }

        // Skip hidden files and directories (unless specifically included)
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
        {
            // Allow if explicitly in include patterns
            if !self
                .config
                .context
                .include
                .iter()
                .any(|pattern| self.config.matches_pattern(&path_str, pattern))
            {
                return false;
            }
        }

        // Skip binary and large files
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let skip_extensions = [
                "exe", "bin", "so", "dll", "dylib", "a", "lib", "jpg", "jpeg", "png", "gif", "bmp",
                "svg", "mp3", "mp4", "avi", "mov", "pdf", "zip", "tar", "gz", "7z", "rar", "class",
                "jar",
            ];

            if skip_extensions.contains(&extension.to_lowercase().as_str()) {
                return false;
            }
        }

        // Check file size (skip very large files)
        if let Ok(metadata) = path.metadata() {
            if metadata.len() > 1_000_000 {
                // Skip files larger than 1MB
                return false;
            }
        }

        true
    }

    /// Convert FileScore results to Files format
    pub async fn scores_to_files(&self, scores: &[analyzer::FileScore]) -> Result<Vec<Files>> {
        let mut files = Vec::new();

        for score in scores {
            let path = Path::new(&score.path);
            if let Ok(content) = fs::read_to_string(path) {
                files.push(Files {
                    path: score.path.clone(),
                    content,
                });
            }
        }

        Ok(files)
    }

    /// Preview context selection without loading file contents
    pub fn preview_context_selection(&self, scores: &[analyzer::FileScore]) -> String {
        let mut preview = String::new();
        preview.push_str("📊 Smart Context Selection Summary\n");
        preview.push_str("═══════════════════════════════════\n\n");

        let total_estimated_tokens: usize = scores
            .iter()
            .map(|score| {
                self.optimizer
                    .estimate_tokens(Path::new(&score.path))
                    .unwrap_or(0)
            })
            .sum();

        preview.push_str(&format!("🎯 Selected {} files\n", scores.len()));
        preview.push_str(&format!(
            "📝 Estimated tokens: ~{}\n",
            total_estimated_tokens
        ));
        preview.push_str(&format!(
            "💾 Token budget: {}\n\n",
            self.optimizer.get_token_budget()
        ));

        preview.push_str("📁 Selected Files (by relevance):\n");
        preview.push_str("─────────────────────────────────\n");

        for (i, score) in scores.iter().enumerate() {
            let relevance_bar = "█".repeat((score.relevance_score * 10.0) as usize);
            let file_name = Path::new(&score.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&score.path);

            preview.push_str(&format!(
                "{:2}. {} ({:.1}%) {}\n    💬 {}\n",
                i + 1,
                relevance_bar,
                score.relevance_score * 100.0,
                file_name,
                score.path
            ));

            if !score.importance_factors.is_empty() {
                preview.push_str("    🏷️  ");
                for (j, factor) in score.importance_factors.iter().enumerate() {
                    if j > 0 {
                        preview.push_str(", ");
                    }
                    preview.push_str(&format!("{:?}", factor));
                }
                preview.push('\n');
            }

            preview.push('\n');
        }

        preview
    }

    /// Calculate a hash of the current configuration for cache validation
    fn calculate_config_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash key configuration values
        self.config.context.max_tokens.hash(&mut hasher);
        self.config.context.include.hash(&mut hasher);
        self.config.context.exclude.hash(&mut hasher);
        self.config.context.priority_patterns.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Invalidate cache for a specific project (public method)
    #[allow(dead_code)]
    pub fn invalidate_cache(&mut self, project_path: &Path) {
        if let Some(cache) = &mut self.cache {
            cache.invalidate_project(project_path);
        }
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn cache_stats(&self) -> Option<cache::CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    /// Clear all cache entries
    #[allow(dead_code)]
    pub fn clear_cache(&mut self) -> Result<()> {
        if let Some(cache) = &mut self.cache {
            cache.clear()?;
        }
        Ok(())
    }

    /// Save cache to disk
    #[allow(dead_code)]
    pub fn save_cache(&self) -> Result<()> {
        if let Some(cache) = &self.cache {
            cache.save()?;
        }
        Ok(())
    }
}

impl Default for SmartContext {
    fn default() -> Self {
        Self::new()
    }
}
