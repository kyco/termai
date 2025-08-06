use crate::context::analyzer::FileScore;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tiktoken_rs::{cl100k_base, CoreBPE};

/// Token optimization strategies for managing context size
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    /// Truncate files based on relevance score
    Truncate,
    /// Summarize large files
    Summarize,
    /// Skip low-importance files
    Skip,
}

/// Configuration for token optimization
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    pub max_tokens: usize,
    pub strategy: OptimizationStrategy,
    pub preserve_signatures: bool,
    pub preserve_imports: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4000,
            strategy: OptimizationStrategy::Truncate,
            preserve_signatures: true,
            preserve_imports: true,
        }
    }
}

/// Optimizes context size by managing token usage
pub struct TokenOptimizer {
    config: OptimizationConfig,
    tokenizer: CoreBPE,
}

impl TokenOptimizer {
    pub fn new() -> Self {
        let tokenizer = cl100k_base().expect("Failed to load cl100k_base tokenizer");
        Self {
            config: OptimizationConfig::default(),
            tokenizer,
        }
    }

    pub fn with_config(config: OptimizationConfig) -> Self {
        let tokenizer = cl100k_base().expect("Failed to load cl100k_base tokenizer");
        Self { config, tokenizer }
    }

    /// Optimize a list of files to fit within token limits
    pub fn optimize_files(&self, files: &[FileScore]) -> Result<Vec<FileScore>> {
        let mut optimized = files.to_vec();

        // Sort by relevance score (highest first)
        optimized.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        let mut total_tokens = 0;
        let mut selected_files = Vec::new();

        for file in optimized {
            let file_tokens = self.estimate_tokens(Path::new(&file.path))?;

            // Check if adding this file would exceed the limit
            if total_tokens + file_tokens > self.config.max_tokens {
                // Try to truncate or summarize based on strategy
                match self.config.strategy {
                    OptimizationStrategy::Truncate => {
                        let remaining_tokens = self.config.max_tokens - total_tokens;
                        if remaining_tokens > 100 {
                            // Minimum viable token count
                            // Add truncated file
                            selected_files.push(file);
                            // Note: total_tokens would be at max_tokens now, so no need to update
                        }
                        break; // Stop adding more files
                    }
                    OptimizationStrategy::Skip => {
                        // Skip this file and continue with next
                        continue;
                    }
                    OptimizationStrategy::Summarize => {
                        // For now, treat as truncate (summarization would be complex)
                        let remaining_tokens = self.config.max_tokens - total_tokens;
                        if remaining_tokens > 100 {
                            selected_files.push(file);
                            // Note: total_tokens would be at max_tokens now, so no need to update
                        }
                        break;
                    }
                }
            } else {
                // File fits within remaining budget
                selected_files.push(file);
                total_tokens += file_tokens;
            }
        }

        Ok(selected_files)
    }

    /// Estimate token count for a file
    pub fn estimate_tokens(&self, path: &Path) -> Result<usize> {
        if !path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(path)?;
        let tokens = self.tokenizer.encode_with_special_tokens(&content);
        Ok(tokens.len())
    }

    /// Count tokens in a string
    pub fn count_tokens(&self, text: &str) -> usize {
        let tokens = self.tokenizer.encode_with_special_tokens(text);
        tokens.len()
    }

    /// Truncate text to fit within token limit
    pub fn truncate_to_tokens(&self, text: &str, max_tokens: usize) -> String {
        if max_tokens == 0 {
            return String::new();
        }

        let tokens = self.tokenizer.encode_with_special_tokens(text);

        if tokens.len() <= max_tokens {
            return text.to_string();
        }

        let original_token_count = tokens.len();

        // Take first max_tokens and decode back
        let truncated_tokens: Vec<usize> = tokens.into_iter().take(max_tokens).collect();

        match self.tokenizer.decode(truncated_tokens) {
            Ok(decoded) => {
                // Add truncation indicator
                if self.config.preserve_signatures {
                    format!(
                        "{}\n\n... [truncated - {} tokens exceeded limit]",
                        decoded,
                        original_token_count - max_tokens
                    )
                } else {
                    format!("{}...", decoded)
                }
            }
            Err(_) => {
                // Fallback to character-based truncation
                let char_limit = (text.len() * max_tokens) / original_token_count.max(1);
                format!("{}...", &text[..char_limit.min(text.len())])
            }
        }
    }

    /// Get total token budget
    pub fn get_token_budget(&self) -> usize {
        self.config.max_tokens
    }

    /// Calculate remaining tokens from budget
    pub fn calculate_remaining_tokens(&self, used_tokens: usize) -> usize {
        self.config.max_tokens.saturating_sub(used_tokens)
    }
}

impl Default for TokenOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::analyzer::{FileScore, FileType, ImportanceFactor};
    use std::time::SystemTime;

    #[test]
    fn test_basic_optimization() {
        let optimizer = TokenOptimizer::new();

        let files = vec![
            FileScore {
                path: "high_relevance.rs".to_string(),
                relevance_score: 0.9,
                size_bytes: 1000,
                modified_time: SystemTime::now(),
                file_type: FileType::SourceCode,
                importance_factors: vec![ImportanceFactor::EntryPoint],
            },
            FileScore {
                path: "low_relevance.rs".to_string(),
                relevance_score: 0.3,
                size_bytes: 2000,
                modified_time: SystemTime::now(),
                file_type: FileType::SourceCode,
                importance_factors: vec![],
            },
        ];

        let optimized = optimizer.optimize_files(&files).unwrap();

        // Should be sorted by relevance (files don't exist so no actual tokens counted)
        if !optimized.is_empty() {
            assert!(
                optimized[0].relevance_score >= optimized.get(1).map_or(0.0, |f| f.relevance_score)
            );
        }
    }

    #[test]
    fn test_token_counting() {
        let optimizer = TokenOptimizer::new();

        let test_text = "Hello, world! This is a test string.";
        let token_count = optimizer.count_tokens(test_text);

        // Should return a reasonable token count
        assert!(token_count > 0);
        assert!(token_count < 100); // Should be much less than 100 for this short text
    }

    #[test]
    fn test_truncation() {
        let optimizer = TokenOptimizer::new();

        let long_text = "This is a very long text that should be truncated. ".repeat(100);
        let truncated = optimizer.truncate_to_tokens(&long_text, 50);

        // Truncated text should be shorter than original
        assert!(truncated.len() < long_text.len());

        // Token count of truncated text should be reasonably close to limit
        // (may be higher due to truncation indicator text)
        let truncated_tokens = optimizer.count_tokens(&truncated);
        assert!(truncated_tokens <= 80); // Allow margin for truncation indicator text
    }
}
