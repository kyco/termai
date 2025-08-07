use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project configuration schema for .termai.toml files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Project metadata
    pub project: Option<ProjectMetadata>,
    
    /// Context discovery and handling configuration
    pub context: Option<ContextConfig>,
    
    /// Provider and model configuration
    pub providers: Option<ProvidersConfig>,
    
    /// Git integration settings
    pub git: Option<GitConfig>,
    
    /// Output formatting preferences
    pub output: Option<OutputConfig>,
    
    /// Template and preset configuration
    pub templates: Option<TemplatesConfig>,
    
    /// Privacy and redaction settings
    pub redaction: Option<RedactionConfig>,
    
    /// Team collaboration settings
    pub team: Option<TeamConfig>,
    
    /// Environment-specific configurations
    pub env: Option<HashMap<String, EnvironmentConfig>>,
    
    /// Quality and review settings
    pub quality: Option<QualityConfig>,
}

/// Project metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,
    
    /// Project type (rust, javascript, python, etc.)
    #[serde(rename = "type")]
    pub project_type: Option<String>,
    
    /// Project description
    pub description: Option<String>,
    
    /// Configuration version for compatibility
    pub version: Option<String>,
    
    /// Project standards/template reference
    pub standards: Option<String>,
}

/// Context discovery and file handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum tokens to include in context
    pub max_tokens: Option<u32>,
    
    /// File patterns to include in context
    pub include: Option<Vec<String>>,
    
    /// File patterns to exclude from context
    pub exclude: Option<Vec<String>>,
    
    /// Base include patterns for environment overrides
    pub base_include: Option<Vec<String>>,
    
    /// Priority patterns for important files
    pub priority_patterns: Option<Vec<String>>,
    
    /// File type specific handling rules
    pub file_types: Option<HashMap<String, FileTypeConfig>>,
    
    /// Context chunking strategy
    pub chunking: Option<ChunkingConfig>,
}

/// File type specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeConfig {
    /// Maximum size for this file type
    pub max_size: Option<usize>,
    
    /// Priority weight for this file type
    pub priority: Option<f32>,
    
    /// Whether to include in context by default
    pub auto_include: Option<bool>,
}

/// Context chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Chunk size in tokens
    pub chunk_size: Option<u32>,
    
    /// Overlap between chunks
    pub overlap: Option<u32>,
    
    /// Strategy for chunking (smart, fixed, etc.)
    pub strategy: Option<String>,
}

/// Provider and model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    /// Default provider to use
    pub default: Option<String>,
    
    /// Fallback provider if default fails
    pub fallback: Option<String>,
    
    /// Whether configuration is locked (for teams)
    pub locked: Option<bool>,
    
    /// Provider-specific configurations
    pub claude: Option<ProviderSettings>,
    pub openai: Option<ProviderSettings>,
}

/// Individual provider settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// Model to use for this provider
    pub model: Option<String>,
    
    /// Maximum tokens for responses
    pub max_tokens: Option<u32>,
    
    /// Temperature for generation
    pub temperature: Option<f32>,
    
    /// API key (for project-specific keys)
    pub api_key: Option<String>,
    
    /// Custom parameters for the provider
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Git integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Automatically generate commit messages
    pub auto_commit_messages: Option<bool>,
    
    /// Review changes before push
    pub review_on_push: Option<bool>,
    
    /// Use conventional commit format
    pub conventional_commits: Option<bool>,
    
    /// Custom commit message template
    pub commit_template: Option<String>,
    
    /// Branches to exclude from AI operations
    pub excluded_branches: Option<Vec<String>>,
    
    /// Auto-review configuration
    pub auto_review: Option<bool>,
}

/// Output formatting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Theme for output formatting
    pub theme: Option<String>,
    
    /// Enable streaming responses
    pub streaming: Option<bool>,
    
    /// Enable syntax highlighting
    pub syntax_highlighting: Option<bool>,
    
    /// Default export format
    pub export_format: Option<String>,
    
    /// Streaming configuration
    pub streaming_config: Option<StreamingConfig>,
}

/// Streaming response configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Characters per batch for streaming
    pub chars_per_batch: Option<usize>,
    
    /// Delay between batches in milliseconds
    pub batch_delay_ms: Option<u64>,
    
    /// Show typing indicator
    pub show_typing_indicator: Option<bool>,
    
    /// Enable smooth scrolling
    pub enable_smooth_scrolling: Option<bool>,
    
    /// Minimum content length to trigger streaming
    pub min_content_length: Option<usize>,
}

/// Template and preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Default template for code review
    pub default_review: Option<String>,
    
    /// Default template for documentation
    pub default_docs: Option<String>,
    
    /// Default template for testing
    pub default_test: Option<String>,
    
    /// Custom template mappings
    pub custom_templates: Option<HashMap<String, String>>,
    
    /// Template repository URL
    pub repository: Option<String>,
}

/// Privacy and redaction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionConfig {
    /// Patterns to redact from context and responses
    pub patterns: Option<Vec<String>>,
    
    /// Strict mode for enhanced privacy
    pub strict_mode: Option<bool>,
    
    /// Custom redaction rules
    pub custom_rules: Option<HashMap<String, RedactionRule>>,
    
    /// Data retention policy
    pub retention: Option<RetentionConfig>,
}

/// Individual redaction rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    /// Pattern to match
    pub pattern: String,
    
    /// Replacement text
    pub replacement: Option<String>,
    
    /// Whether to apply to context, responses, or both
    pub scope: Option<Vec<String>>,
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// How long to keep session data (days)
    pub session_days: Option<u32>,
    
    /// How long to keep cache data (days) 
    pub cache_days: Option<u32>,
    
    /// Auto-cleanup settings
    pub auto_cleanup: Option<bool>,
}

/// Team collaboration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    /// Shared presets repository URL
    pub shared_presets: Option<String>,
    
    /// Sync frequency for team settings
    pub sync_frequency: Option<String>,
    
    /// Team-specific templates
    pub team_templates: Option<Vec<String>>,
    
    /// Approval workflow for changes
    pub approval_required: Option<bool>,
}

/// Environment-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Context configuration overrides
    pub context: Option<ContextConfig>,
    
    /// Provider overrides
    pub providers: Option<ProvidersConfig>,
    
    /// Git settings overrides
    pub git: Option<GitConfig>,
    
    /// Output format overrides
    pub output: Option<OutputConfig>,
    
    /// Redaction overrides
    pub redaction: Option<RedactionConfig>,
}

/// Quality and review configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Enable automatic code review
    pub auto_review: Option<bool>,
    
    /// Review depth (quick, standard, thorough)
    pub review_depth: Option<String>,
    
    /// Enable security scanning
    pub security_scan: Option<bool>,
    
    /// Performance analysis settings
    pub performance_analysis: Option<bool>,
    
    /// Code quality thresholds
    pub quality_thresholds: Option<QualityThresholds>,
}

/// Code quality threshold settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum test coverage percentage
    pub min_coverage: Option<f32>,
    
    /// Maximum cyclomatic complexity
    pub max_complexity: Option<u32>,
    
    /// Maximum function length (lines)
    pub max_function_length: Option<u32>,
    
    /// Maximum file length (lines)
    pub max_file_length: Option<u32>,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Configuration validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Configuration validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl ProjectConfig {
    /// Create a new empty project configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate context configuration
        if let Some(context) = &self.context {
            if let Some(max_tokens) = context.max_tokens {
                if max_tokens > 32000 {
                    warnings.push(ValidationWarning {
                        field: "context.max_tokens".to_string(),
                        message: "Max tokens exceeds recommended limit".to_string(),
                        suggestion: Some("Consider reducing to 8000-16000 for better performance".to_string()),
                    });
                }
            }
            
            // Check for conflicting include/exclude patterns
            if let (Some(include), Some(exclude)) = (&context.include, &context.exclude) {
                for include_pattern in include {
                    for exclude_pattern in exclude {
                        if include_pattern == exclude_pattern {
                            errors.push(ValidationError {
                                field: "context".to_string(),
                                message: format!("Conflicting pattern: {}", include_pattern),
                                suggestion: Some("Remove duplicate patterns from include or exclude".to_string()),
                            });
                        }
                    }
                }
            }
        }
        
        // Validate provider configuration
        if let Some(providers) = &self.providers {
            if let Some(default) = &providers.default {
                let valid_providers = ["claude", "openai"];
                if !valid_providers.contains(&default.as_str()) {
                    errors.push(ValidationError {
                        field: "providers.default".to_string(),
                        message: format!("Unknown provider: {}", default),
                        suggestion: Some("Use one of: claude, openai".to_string()),
                    });
                }
            }
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
    
    /// Get the effective configuration for a given environment
    pub fn for_environment(&self, env: &str) -> ProjectConfig {
        let mut config = self.clone();
        
        if let Some(env_configs) = &self.env {
            if let Some(env_config) = env_configs.get(env) {
                // Merge environment-specific overrides
                if let Some(env_context) = &env_config.context {
                    config.context = Some(merge_context_config(
                        config.context.as_ref(),
                        env_context
                    ));
                }
                
                if let Some(env_providers) = &env_config.providers {
                    config.providers = Some(merge_providers_config(
                        config.providers.as_ref(),
                        env_providers
                    ));
                }
                
                // Add other environment merging as needed
            }
        }
        
        config
    }
}

/// Merge context configurations (environment overrides base)
fn merge_context_config(base: Option<&ContextConfig>, env: &ContextConfig) -> ContextConfig {
    let mut merged = base.cloned().unwrap_or_default();
    
    if env.max_tokens.is_some() {
        merged.max_tokens = env.max_tokens;
    }
    if env.include.is_some() {
        merged.include = env.include.clone();
    }
    if env.exclude.is_some() {
        merged.exclude = env.exclude.clone();
    }
    
    merged
}

/// Merge provider configurations (environment overrides base)
fn merge_providers_config(base: Option<&ProvidersConfig>, env: &ProvidersConfig) -> ProvidersConfig {
    let mut merged = base.cloned().unwrap_or_default();
    
    if env.default.is_some() {
        merged.default = env.default.clone();
    }
    if env.fallback.is_some() {
        merged.fallback = env.fallback.clone();
    }
    
    merged
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: Some(8000),
            include: None,
            exclude: None,
            base_include: None,
            priority_patterns: None,
            file_types: None,
            chunking: None,
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default: Some("claude".to_string()),
            fallback: Some("openai".to_string()),
            locked: Some(false),
            claude: None,
            openai: None,
        }
    }
}