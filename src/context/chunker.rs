use crate::context::analyzer::{FileScore, FileType, ImportanceFactor};
use crate::path::model::Files;
use anyhow::Result;
use std::path::Path;

/// Different chunking strategies for handling large projects
#[derive(Debug, Clone)]
pub enum ChunkingStrategy {
    /// Divide by project modules/packages
    ModuleBased,
    /// Chunk by functionality (tests, configs, main code)
    FunctionalBased,
    /// Split files by token count to fit within limits
    TokenBased,
    /// Create hierarchical overview first, then detailed chunks
    Hierarchical,
}

/// Represents a chunk of project context
#[derive(Debug, Clone)]
pub struct ContextChunk {
    pub id: String,
    pub name: String,
    pub description: String,
    pub files: Vec<Files>,
    pub estimated_tokens: usize,
    pub chunk_type: ChunkType,
    pub priority: f32, // 0.0 to 1.0
}

/// Type of context chunk
#[derive(Debug, Clone)]
pub enum ChunkType {
    /// High-level project overview (README, main configs)
    Overview,
    /// Core application logic
    Core,
    /// Supporting utilities and helpers
    Utils,
    /// Tests and examples
    Tests,
    /// Configuration and build files
    Config,
    /// Documentation
    Docs,
}

/// Manages chunking of large project contexts
pub struct ContextChunker {
    max_tokens_per_chunk: usize,
    strategy: ChunkingStrategy,
}

impl ContextChunker {
    pub fn new(max_tokens_per_chunk: usize, strategy: ChunkingStrategy) -> Self {
        Self {
            max_tokens_per_chunk,
            strategy,
        }
    }

    /// Create chunks from scored files
    pub async fn create_chunks(&self, files: &[FileScore]) -> Result<Vec<ContextChunk>> {
        match self.strategy {
            ChunkingStrategy::ModuleBased => self.chunk_by_modules(files).await,
            ChunkingStrategy::FunctionalBased => self.chunk_by_function(files).await,
            ChunkingStrategy::TokenBased => self.chunk_by_tokens(files).await,
            ChunkingStrategy::Hierarchical => self.chunk_hierarchically(files).await,
        }
    }

    /// Create a project overview chunk (always first)
    pub async fn create_overview_chunk(&self, files: &[FileScore]) -> Result<ContextChunk> {
        let mut overview_files = Vec::new();
        let mut total_tokens = 0;

        // Prioritize overview files
        let overview_priorities = [
            "README.md",
            "CHANGELOG.md",
            "LICENSE",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "main.rs",
            "index.js",
            "main.py",
            "__init__.py",
        ];

        for file in files {
            let file_name = Path::new(&file.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            if overview_priorities
                .iter()
                .any(|&priority| file_name.contains(priority))
            {
                if let Ok(content) = std::fs::read_to_string(&file.path) {
                    // Estimate tokens (rough approximation: 1 token ≈ 4 characters)
                    let file_tokens = content.len() / 4;

                    if total_tokens + file_tokens <= self.max_tokens_per_chunk {
                        overview_files.push(Files {
                            path: file.path.clone(),
                            content,
                        });
                        total_tokens += file_tokens;
                    } else {
                        // Truncate file to fit
                        let available_chars = (self.max_tokens_per_chunk - total_tokens) * 4;
                        if available_chars > 100 {
                            // Minimum viable content
                            let truncated = if content.len() > available_chars {
                                format!(
                                    "{}...\n\n[FILE TRUNCATED - {} chars total]",
                                    &content[..available_chars],
                                    content.len()
                                )
                            } else {
                                content
                            };

                            overview_files.push(Files {
                                path: file.path.clone(),
                                content: truncated,
                            });
                            break; // Chunk is full
                        }
                    }
                }
            }
        }

        Ok(ContextChunk {
            id: "overview".to_string(),
            name: "Project Overview".to_string(),
            description: "Main project files, configuration, and entry points".to_string(),
            files: overview_files,
            estimated_tokens: total_tokens,
            chunk_type: ChunkType::Overview,
            priority: 1.0,
        })
    }

    /// Chunk by project modules/directories
    async fn chunk_by_modules(&self, files: &[FileScore]) -> Result<Vec<ContextChunk>> {
        let mut chunks = Vec::new();
        let mut modules: std::collections::HashMap<String, Vec<&FileScore>> =
            std::collections::HashMap::new();

        // Group files by their directory structure
        for file in files {
            let path = Path::new(&file.path);
            let module = if let Some(parent) = path.parent() {
                parent.to_string_lossy().to_string()
            } else {
                "root".to_string()
            };

            modules.entry(module).or_insert_with(Vec::new).push(file);
        }

        // Create chunks for each module
        for (module_name, module_files) in modules {
            let chunk = self
                .create_module_chunk(&module_name, &module_files)
                .await?;
            chunks.push(chunk);
        }

        // Sort by priority
        chunks.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        Ok(chunks)
    }

    /// Create a chunk for a specific module
    async fn create_module_chunk(
        &self,
        module_name: &str,
        files: &[&FileScore],
    ) -> Result<ContextChunk> {
        let mut chunk_files = Vec::new();
        let mut total_tokens = 0;

        // Sort files by relevance within the module
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        for file in sorted_files {
            if let Ok(content) = std::fs::read_to_string(&file.path) {
                let file_tokens = content.len() / 4; // Rough approximation

                if total_tokens + file_tokens <= self.max_tokens_per_chunk {
                    chunk_files.push(Files {
                        path: file.path.clone(),
                        content,
                    });
                    total_tokens += file_tokens;
                } else if chunk_files.is_empty() {
                    // Always include at least one file, truncated if necessary
                    let available_chars = self.max_tokens_per_chunk * 4;
                    let truncated = if content.len() > available_chars {
                        format!(
                            "{}...\n\n[FILE TRUNCATED - {} chars total]",
                            &content[..available_chars],
                            content.len()
                        )
                    } else {
                        content
                    };

                    chunk_files.push(Files {
                        path: file.path.clone(),
                        content: truncated,
                    });
                    break;
                }
            }
        }

        // Calculate priority based on file types and importance
        let priority = self.calculate_module_priority(files);
        let chunk_type = self.determine_chunk_type(module_name, files);

        Ok(ContextChunk {
            id: format!("module_{}", module_name.replace(['/', '\\'], "_")),
            name: format!("Module: {}", module_name),
            description: format!(
                "Files from {} directory ({} files)",
                module_name,
                chunk_files.len()
            ),
            files: chunk_files,
            estimated_tokens: total_tokens,
            chunk_type,
            priority,
        })
    }

    /// Chunk by functionality (core, utils, tests, etc.)
    async fn chunk_by_function(&self, files: &[FileScore]) -> Result<Vec<ContextChunk>> {
        let mut core_files = Vec::new();
        let mut test_files = Vec::new();
        let mut config_files = Vec::new();
        let mut util_files = Vec::new();
        let mut doc_files = Vec::new();

        // Classify files by function
        for file in files {
            match &file.file_type {
                FileType::Test => test_files.push(file),
                FileType::Configuration => config_files.push(file),
                FileType::Documentation => doc_files.push(file),
                FileType::SourceCode => {
                    if file
                        .importance_factors
                        .contains(&ImportanceFactor::EntryPoint)
                        || file
                            .importance_factors
                            .contains(&ImportanceFactor::MainModule)
                    {
                        core_files.push(file);
                    } else {
                        util_files.push(file);
                    }
                }
                _ => util_files.push(file),
            }
        }

        let mut chunks = Vec::new();

        // Create functional chunks
        if !core_files.is_empty() {
            chunks.push(
                self.create_functional_chunk(
                    "core",
                    "Core Application",
                    &core_files,
                    ChunkType::Core,
                    0.9,
                )
                .await?,
            );
        }
        if !config_files.is_empty() {
            chunks.push(
                self.create_functional_chunk(
                    "config",
                    "Configuration",
                    &config_files,
                    ChunkType::Config,
                    0.7,
                )
                .await?,
            );
        }
        if !util_files.is_empty() {
            chunks.push(
                self.create_functional_chunk(
                    "utils",
                    "Utilities & Helpers",
                    &util_files,
                    ChunkType::Utils,
                    0.5,
                )
                .await?,
            );
        }
        if !test_files.is_empty() {
            chunks.push(
                self.create_functional_chunk(
                    "tests",
                    "Tests & Examples",
                    &test_files,
                    ChunkType::Tests,
                    0.3,
                )
                .await?,
            );
        }
        if !doc_files.is_empty() {
            chunks.push(
                self.create_functional_chunk(
                    "docs",
                    "Documentation",
                    &doc_files,
                    ChunkType::Docs,
                    0.4,
                )
                .await?,
            );
        }

        Ok(chunks)
    }

    async fn create_functional_chunk(
        &self,
        id: &str,
        name: &str,
        files: &[&FileScore],
        chunk_type: ChunkType,
        priority: f32,
    ) -> Result<ContextChunk> {
        let mut chunk_files = Vec::new();
        let mut total_tokens = 0;

        // Sort by relevance
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        for file in sorted_files.iter().take(20) {
            // Limit to top 20 files per chunk
            if let Ok(content) = std::fs::read_to_string(&file.path) {
                let file_tokens = content.len() / 4;

                if total_tokens + file_tokens <= self.max_tokens_per_chunk {
                    chunk_files.push(Files {
                        path: file.path.clone(),
                        content,
                    });
                    total_tokens += file_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(ContextChunk {
            id: id.to_string(),
            name: name.to_string(),
            description: format!("{} files ({} items)", name, chunk_files.len()),
            files: chunk_files,
            estimated_tokens: total_tokens,
            chunk_type,
            priority,
        })
    }

    /// Simple token-based chunking
    async fn chunk_by_tokens(&self, files: &[FileScore]) -> Result<Vec<ContextChunk>> {
        let mut chunks = Vec::new();
        let mut current_files = Vec::new();
        let mut current_tokens = 0;
        let mut chunk_id = 0;

        // Sort files by relevance (highest first)
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        for file in sorted_files {
            if let Ok(content) = std::fs::read_to_string(&file.path) {
                let file_tokens = content.len() / 4;

                if current_tokens + file_tokens > self.max_tokens_per_chunk
                    && !current_files.is_empty()
                {
                    // Create chunk from current files
                    chunks.push(ContextChunk {
                        id: format!("chunk_{}", chunk_id),
                        name: format!("Context Chunk {}", chunk_id + 1),
                        description: format!(
                            "Token-based chunk ({} files, ~{} tokens)",
                            current_files.len(),
                            current_tokens
                        ),
                        files: std::mem::take(&mut current_files), // Take ownership and leave empty Vec
                        estimated_tokens: current_tokens,
                        chunk_type: ChunkType::Core,
                        priority: 1.0 - (chunk_id as f32 * 0.1), // Decreasing priority
                    });

                    current_tokens = 0;
                    chunk_id += 1;
                }

                current_files.push(Files {
                    path: file.path.clone(),
                    content,
                });
                current_tokens += file_tokens;
            }
        }

        // Add remaining files as final chunk
        if !current_files.is_empty() {
            chunks.push(ContextChunk {
                id: format!("chunk_{}", chunk_id),
                name: format!("Context Chunk {}", chunk_id + 1),
                description: format!(
                    "Token-based chunk ({} files, ~{} tokens)",
                    current_files.len(),
                    current_tokens
                ),
                files: current_files,
                estimated_tokens: current_tokens,
                chunk_type: ChunkType::Core,
                priority: 1.0 - (chunk_id as f32 * 0.1),
            });
        }

        Ok(chunks)
    }

    /// Advanced hierarchical chunking
    async fn chunk_hierarchically(&self, files: &[FileScore]) -> Result<Vec<ContextChunk>> {
        let mut chunks = Vec::new();

        // 1. Always start with overview
        let overview = self.create_overview_chunk(files).await?;
        chunks.push(overview);

        // 2. Create functional chunks for remaining files
        let remaining_files: Vec<&FileScore> =
            files.iter().filter(|f| !self.is_overview_file(f)).collect();

        // Convert Vec<&FileScore> to Vec<FileScore> for chunk_by_function
        let remaining_scores: Vec<FileScore> = remaining_files.into_iter().cloned().collect();
        let mut functional_chunks = self.chunk_by_function(&remaining_scores).await?;
        chunks.append(&mut functional_chunks);

        Ok(chunks)
    }

    /// Helper methods
    fn calculate_module_priority(&self, files: &[&FileScore]) -> f32 {
        let avg_relevance: f32 =
            files.iter().map(|f| f.relevance_score).sum::<f32>() / files.len() as f32;

        // Boost priority if module contains important files
        let has_important_files = files.iter().any(|f| {
            f.importance_factors.contains(&ImportanceFactor::EntryPoint)
                || f.importance_factors.contains(&ImportanceFactor::MainModule)
        });

        if has_important_files {
            (avg_relevance + 0.3).min(1.0)
        } else {
            avg_relevance
        }
    }

    fn determine_chunk_type(&self, module_name: &str, files: &[&FileScore]) -> ChunkType {
        let name_lower = module_name.to_lowercase();

        if name_lower.contains("test") {
            ChunkType::Tests
        } else if name_lower.contains("config") || name_lower.contains("settings") {
            ChunkType::Config
        } else if name_lower.contains("doc") || name_lower.contains("readme") {
            ChunkType::Docs
        } else if name_lower.contains("util") || name_lower.contains("helper") {
            ChunkType::Utils
        } else if files
            .iter()
            .any(|f| f.importance_factors.contains(&ImportanceFactor::EntryPoint))
        {
            ChunkType::Core
        } else {
            ChunkType::Utils
        }
    }

    fn is_overview_file(&self, file: &FileScore) -> bool {
        let file_name = Path::new(&file.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        let overview_patterns = [
            "README",
            "CHANGELOG",
            "LICENSE",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "main.rs",
        ];

        overview_patterns
            .iter()
            .any(|pattern| file_name.contains(pattern))
    }
}

impl Default for ContextChunker {
    fn default() -> Self {
        Self::new(3000, ChunkingStrategy::Hierarchical)
    }
}
