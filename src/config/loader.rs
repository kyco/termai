use crate::config::schema::ProjectConfig;
use anyhow::{Result, anyhow, Context};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::env;

/// Configuration source priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum ConfigSource {
    /// System-wide configuration (lowest priority)
    System = 0,
    /// User-specific configuration
    User = 1,
    /// Project-specific configuration
    Project = 2,
    /// Environment variables
    Environment = 3,
    /// Command line arguments (highest priority)
    CommandLine = 4,
}

/// Configuration path with metadata
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConfigPath {
    pub source: ConfigSource,
    pub path: PathBuf,
    pub exists: bool,
    pub description: String,
}

/// Configuration loader with discovery and merging capabilities
pub struct ConfigLoader {
    /// Current working directory
    current_dir: PathBuf,
    /// Environment name (development, production, etc.)
    environment: Option<String>,
    /// Command line overrides
    cli_overrides: Option<ProjectConfig>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Result<Self> {
        let current_dir = env::current_dir()
            .context("Failed to get current working directory")?;
            
        Ok(Self {
            current_dir,
            environment: env::var("TERMAI_ENV").ok(),
            cli_overrides: None,
        })
    }
    
    /// Create a configuration loader with a specific working directory
    #[allow(dead_code)]
    pub fn with_current_dir(dir: PathBuf) -> Self {
        Self {
            current_dir: dir,
            environment: env::var("TERMAI_ENV").ok(),
            cli_overrides: None,
        }
    }
    
    /// Set environment name
    #[allow(dead_code)]
    pub fn with_environment(mut self, env: String) -> Self {
        self.environment = Some(env);
        self
    }
    
    /// Set command line configuration overrides
    #[allow(dead_code)]
    pub fn with_cli_overrides(mut self, config: ProjectConfig) -> Self {
        self.cli_overrides = Some(config);
        self
    }
    
    /// Discover all possible configuration file paths in priority order
    pub fn discover_config_paths(&self) -> Vec<ConfigPath> {
        let mut paths = Vec::new();
        
        // 1. System-wide configuration (lowest priority)
        paths.push(self.get_system_config_path());
        
        // 2. User-specific configuration
        paths.push(self.get_user_config_path());
        
        // 3. Project-specific configurations
        paths.extend(self.get_project_config_paths());
        
        // 4. Environment variables (handled separately)
        
        // 5. Command line arguments (highest priority, handled separately)
        
        paths
    }
    
    /// Load the complete configuration by merging all sources
    pub fn load_config(&self) -> Result<ProjectConfig> {
        let paths = self.discover_config_paths();
        let mut merged_config = ProjectConfig::new();
        
        // Load and merge configurations in priority order (lowest to highest)
        for config_path in paths {
            if config_path.exists {
                match self.load_config_file(&config_path.path) {
                    Ok(config) => {
                        merged_config = self.merge_configs(merged_config, config)?;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load config from {}: {}", 
                                config_path.path.display(), e);
                    }
                }
            }
        }
        
        // Apply environment variable overrides
        merged_config = self.apply_environment_overrides(merged_config)?;
        
        // Apply command line overrides (highest priority)
        if let Some(cli_config) = &self.cli_overrides {
            merged_config = self.merge_configs(merged_config, cli_config.clone())?;
        }
        
        // Apply environment-specific configuration if set
        if let Some(env) = &self.environment {
            merged_config = merged_config.for_environment(env);
        }
        
        // Validate the final configuration
        let validation = merged_config.validate();
        if !validation.is_valid {
            for error in &validation.errors {
                eprintln!("Configuration error in {}: {}", error.field, error.message);
                if let Some(suggestion) = &error.suggestion {
                    eprintln!("  Suggestion: {}", suggestion);
                }
            }
            return Err(anyhow!("Configuration validation failed"));
        }
        
        // Show warnings
        for warning in &validation.warnings {
            eprintln!("Configuration warning in {}: {}", warning.field, warning.message);
            if let Some(suggestion) = &warning.suggestion {
                eprintln!("  Suggestion: {}", suggestion);
            }
        }
        
        Ok(merged_config)
    }
    
    /// Load a configuration file from a specific path
    pub fn load_config_file(&self, path: &Path) -> Result<ProjectConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            
        let config: ProjectConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
            
        Ok(config)
    }
    
    /// Save a configuration to a specific path
    pub fn save_config_file(&self, config: &ProjectConfig, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(config)
            .context("Failed to serialize configuration to TOML")?;
            
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
            
        Ok(())
    }
    
    /// Get system-wide configuration path
    fn get_system_config_path(&self) -> ConfigPath {
        let path = PathBuf::from("/etc/termai/config.toml");
        ConfigPath {
            exists: path.exists(),
            source: ConfigSource::System,
            path,
            description: "System-wide configuration".to_string(),
        }
    }
    
    /// Get user-specific configuration path
    fn get_user_config_path(&self) -> ConfigPath {
        let path = dirs::config_dir()
            .map(|dir| dir.join("termai").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/termai/config.toml"));
            
        ConfigPath {
            exists: path.exists(),
            source: ConfigSource::User,
            path,
            description: "User-specific configuration".to_string(),
        }
    }
    
    /// Get all project-specific configuration paths
    fn get_project_config_paths(&self) -> Vec<ConfigPath> {
        let mut paths = Vec::new();
        
        // Walk up the directory tree looking for configuration files
        let mut current = Some(self.current_dir.as_path());
        
        while let Some(dir) = current {
            // Check for .termai.toml in current directory
            let config_path = dir.join(".termai.toml");
            paths.push(ConfigPath {
                exists: config_path.exists(),
                source: ConfigSource::Project,
                path: config_path,
                description: format!("Project config in {}", dir.display()),
            });
            
            // Check for .termai/config.toml
            let config_dir_path = dir.join(".termai").join("config.toml");
            paths.push(ConfigPath {
                exists: config_dir_path.exists(),
                source: ConfigSource::Project,
                path: config_dir_path,
                description: format!("Project config directory in {}", dir.display()),
            });
            
            // Stop at git repository root if we find one
            if dir.join(".git").exists() {
                break;
            }
            
            // Move up to parent directory
            current = dir.parent();
        }
        
        // Sort by proximity to current directory (closer directories have higher priority)
        paths.reverse();
        paths
    }
    
    /// Apply environment variable overrides
    fn apply_environment_overrides(&self, mut config: ProjectConfig) -> Result<ProjectConfig> {
        // Check for TERMAI_* environment variables and apply them
        
        // Provider configuration
        if let Ok(provider) = env::var("TERMAI_PROVIDER") {
            if config.providers.is_none() {
                config.providers = Some(Default::default());
            }
            if let Some(providers) = &mut config.providers {
                providers.default = Some(provider);
            }
        }
        
        // Max tokens
        if let Ok(max_tokens_str) = env::var("TERMAI_MAX_TOKENS") {
            if let Ok(max_tokens) = max_tokens_str.parse::<u32>() {
                if config.context.is_none() {
                    config.context = Some(Default::default());
                }
                if let Some(context) = &mut config.context {
                    context.max_tokens = Some(max_tokens);
                }
            }
        }
        
        // Theme
        if let Ok(theme) = env::var("TERMAI_THEME") {
            if config.output.is_none() {
                config.output = Some(Default::default());
            }
            if let Some(output) = &mut config.output {
                output.theme = Some(theme);
            }
        }
        
        Ok(config)
    }
    
    /// Merge two configurations (second takes priority over first)
    fn merge_configs(&self, base: ProjectConfig, override_config: ProjectConfig) -> Result<ProjectConfig> {
        let mut merged = base;
        
        // Merge project metadata
        if override_config.project.is_some() {
            merged.project = override_config.project;
        }
        
        // Merge context configuration
        if let Some(override_context) = override_config.context {
            let base_context = merged.context.take().unwrap_or_default();
            merged.context = Some(self.merge_context_configs(base_context, override_context));
        }
        
        // Merge provider configuration
        if let Some(override_providers) = override_config.providers {
            let base_providers = merged.providers.take().unwrap_or_default();
            merged.providers = Some(self.merge_provider_configs(base_providers, override_providers));
        }
        
        // Merge other configurations
        if override_config.git.is_some() {
            merged.git = override_config.git;
        }
        
        if override_config.output.is_some() {
            merged.output = override_config.output;
        }
        
        if override_config.templates.is_some() {
            merged.templates = override_config.templates;
        }
        
        if override_config.redaction.is_some() {
            merged.redaction = override_config.redaction;
        }
        
        if override_config.team.is_some() {
            merged.team = override_config.team;
        }
        
        if override_config.quality.is_some() {
            merged.quality = override_config.quality;
        }
        
        // Merge environment configurations
        if let Some(override_envs) = override_config.env {
            if merged.env.is_none() {
                merged.env = Some(HashMap::new());
            }
            if let Some(base_envs) = &mut merged.env {
                for (env_name, env_config) in override_envs {
                    base_envs.insert(env_name, env_config);
                }
            }
        }
        
        Ok(merged)
    }
    
    /// Merge context configurations
    fn merge_context_configs(&self, mut base: crate::config::schema::ContextConfig, override_config: crate::config::schema::ContextConfig) -> crate::config::schema::ContextConfig {
        if override_config.max_tokens.is_some() {
            base.max_tokens = override_config.max_tokens;
        }
        if override_config.include.is_some() {
            base.include = override_config.include;
        }
        if override_config.exclude.is_some() {
            base.exclude = override_config.exclude;
        }
        if override_config.priority_patterns.is_some() {
            base.priority_patterns = override_config.priority_patterns;
        }
        if override_config.file_types.is_some() {
            base.file_types = override_config.file_types;
        }
        if override_config.chunking.is_some() {
            base.chunking = override_config.chunking;
        }
        base
    }
    
    /// Merge provider configurations
    fn merge_provider_configs(&self, mut base: crate::config::schema::ProvidersConfig, override_config: crate::config::schema::ProvidersConfig) -> crate::config::schema::ProvidersConfig {
        if override_config.default.is_some() {
            base.default = override_config.default;
        }
        if override_config.fallback.is_some() {
            base.fallback = override_config.fallback;
        }
        if override_config.locked.is_some() {
            base.locked = override_config.locked;
        }
        if override_config.claude.is_some() {
            base.claude = override_config.claude;
        }
        if override_config.openai.is_some() {
            base.openai = override_config.openai;
        }
        base
    }
    
    /// Get the current project's configuration file path (for saving)
    pub fn get_project_config_path(&self) -> PathBuf {
        self.current_dir.join(".termai.toml")
    }
    
    /// Check if a project configuration exists
    pub fn project_config_exists(&self) -> bool {
        self.get_project_config_path().exists()
    }
    
    /// Find the git repository root
    pub fn find_git_root(&self) -> Option<PathBuf> {
        let mut current = Some(self.current_dir.as_path());
        
        while let Some(dir) = current {
            if dir.join(".git").exists() {
                return Some(dir.to_path_buf());
            }
            current = dir.parent();
        }
        
        None
    }
}