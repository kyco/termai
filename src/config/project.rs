use crate::config::schema::{ProjectConfig, ProjectMetadata, ContextConfig, ProvidersConfig};
use crate::config::loader::ConfigLoader;
use anyhow::{Result, Context};
use std::path::PathBuf;
use std::collections::HashMap;

/// Project configuration service for managing .termai.toml files
pub struct ProjectConfigService {
    loader: ConfigLoader,
    cached_config: Option<ProjectConfig>,
}

impl ProjectConfigService {
    /// Create a new project configuration service
    pub fn new() -> Result<Self> {
        let loader = ConfigLoader::new()?;
        Ok(Self {
            loader,
            cached_config: None,
        })
    }
    
    /// Create service with specific working directory
    #[allow(dead_code)]
    pub fn with_current_dir(dir: PathBuf) -> Self {
        let loader = ConfigLoader::with_current_dir(dir);
        Self {
            loader,
            cached_config: None,
        }
    }
    
    /// Load the current project configuration
    pub fn load_config(&mut self) -> Result<&ProjectConfig> {
        if self.cached_config.is_none() {
            let config = self.loader.load_config()
                .context("Failed to load project configuration")?;
            self.cached_config = Some(config);
        }
        
        Ok(self.cached_config.as_ref().unwrap())
    }
    
    /// Force reload the configuration (clears cache)
    #[allow(dead_code)]
    pub fn reload_config(&mut self) -> Result<&ProjectConfig> {
        self.cached_config = None;
        self.load_config()
    }
    
    /// Get configuration with environment override
    #[allow(dead_code)]
    pub fn get_config_for_environment(&mut self, env: &str) -> Result<ProjectConfig> {
        let base_config = self.load_config()?.clone();
        Ok(base_config.for_environment(env))
    }
    
    /// Initialize a new project configuration
    pub fn init_project_config(&self, project_type: Option<String>) -> Result<ProjectConfig> {
        let detected_type = project_type.or_else(|| self.detect_project_type());
        
        let mut config = ProjectConfig::new();
        
        // Set project metadata
        let project_name = self.get_project_name();
        config.project = Some(ProjectMetadata {
            name: project_name,
            project_type: detected_type.clone(),
            description: None,
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        // Set context configuration based on project type
        config.context = Some(self.get_default_context_for_type(&detected_type));
        
        // Set default providers
        config.providers = Some(ProvidersConfig::default());
        
        // Set project-type specific defaults
        if let Some(project_type) = &detected_type {
            self.apply_project_type_defaults(&mut config, project_type);
        }
        
        Ok(config)
    }
    
    /// Save configuration to project file
    pub fn save_project_config(&self, config: &ProjectConfig) -> Result<()> {
        let path = self.loader.get_project_config_path();
        self.loader.save_config_file(config, &path)
            .context("Failed to save project configuration")
    }
    
    /// Check if project configuration exists
    pub fn project_config_exists(&self) -> bool {
        self.loader.project_config_exists()
    }
    
    /// Load configuration from a specific file
    pub fn load_config_file(&self, path: &std::path::Path) -> Result<ProjectConfig> {
        self.loader.load_config_file(path)
    }
    
    /// Get the project configuration file path
    pub fn get_config_file_path(&self) -> PathBuf {
        self.loader.get_project_config_path()
    }
    
    /// Detect project type based on files in directory
    fn detect_project_type(&self) -> Option<String> {
        let current_dir = std::env::current_dir().ok()?;
        
        // Check for specific files that indicate project type
        if current_dir.join("Cargo.toml").exists() {
            return Some("rust".to_string());
        }
        
        if current_dir.join("package.json").exists() {
            // Check if it's a specific JS framework
            if current_dir.join("next.config.js").exists() || current_dir.join("next.config.ts").exists() {
                return Some("nextjs".to_string());
            }
            if current_dir.join("src").join("App.tsx").exists() || current_dir.join("src").join("App.jsx").exists() {
                return Some("react".to_string());
            }
            if current_dir.join("angular.json").exists() {
                return Some("angular".to_string());
            }
            if current_dir.join("vue.config.js").exists() || current_dir.join("src").join("App.vue").exists() {
                return Some("vue".to_string());
            }
            return Some("javascript".to_string());
        }
        
        if current_dir.join("pyproject.toml").exists() || current_dir.join("setup.py").exists() || current_dir.join("requirements.txt").exists() {
            return Some("python".to_string());
        }
        
        if current_dir.join("go.mod").exists() {
            return Some("go".to_string());
        }
        
        if current_dir.join("pom.xml").exists() || current_dir.join("build.gradle").exists() {
            return Some("java".to_string());
        }
        
        if current_dir.join("Gemfile").exists() {
            return Some("ruby".to_string());
        }
        
        if current_dir.join("composer.json").exists() {
            return Some("php".to_string());
        }
        
        if current_dir.join("mix.exs").exists() {
            return Some("elixir".to_string());
        }
        
        if current_dir.join("pubspec.yaml").exists() {
            return Some("dart".to_string());
        }
        
        None
    }
    
    /// Get project name from directory or git repository
    fn get_project_name(&self) -> String {
        // Try to get name from git remote
        if let Some(git_root) = self.loader.find_git_root() {
            if let Ok(repo) = git2::Repository::open(&git_root) {
                if let Ok(remote) = repo.find_remote("origin") {
                    if let Some(url) = remote.url() {
                        // Extract repository name from URL
                        if let Some(name) = self.extract_repo_name_from_url(url) {
                            return name;
                        }
                    }
                }
            }
        }
        
        // Fall back to directory name
        std::env::current_dir()
            .ok()
            .and_then(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    /// Extract repository name from git URL
    fn extract_repo_name_from_url(&self, url: &str) -> Option<String> {
        // Handle different URL formats
        let clean_url = if url.ends_with(".git") {
            &url[..url.len() - 4]
        } else {
            url
        };
        
        // Extract last part of path
        clean_url.split('/').last().map(|s| s.to_string())
    }
    
    /// Get default context configuration for project type
    fn get_default_context_for_type(&self, project_type: &Option<String>) -> ContextConfig {
        let mut context = ContextConfig::default();
        
        match project_type.as_deref() {
            Some("rust") => {
                context.include = Some(vec![
                    "src/**/*.rs".to_string(),
                    "tests/**/*.rs".to_string(),
                    "examples/**/*.rs".to_string(),
                    "Cargo.toml".to_string(),
                    "Cargo.lock".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "target/**".to_string(),
                    "**/*.rlib".to_string(),
                    "**/.*".to_string(),
                ]);
                context.priority_patterns = Some(vec![
                    "main.rs".to_string(),
                    "lib.rs".to_string(),
                    "mod.rs".to_string(),
                ]);
            },
            Some("javascript") | Some("typescript") | Some("react") | Some("nextjs") | Some("vue") | Some("angular") => {
                context.include = Some(vec![
                    "src/**/*.{js,ts,jsx,tsx}".to_string(),
                    "pages/**/*.{js,ts,jsx,tsx}".to_string(),
                    "components/**/*.{js,ts,jsx,tsx}".to_string(),
                    "lib/**/*.{js,ts}".to_string(),
                    "utils/**/*.{js,ts}".to_string(),
                    "tests/**/*.{js,ts,jsx,tsx}".to_string(),
                    "package.json".to_string(),
                    "tsconfig.json".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "node_modules/**".to_string(),
                    "dist/**".to_string(),
                    "build/**".to_string(),
                    ".next/**".to_string(),
                    "coverage/**".to_string(),
                    "**/*.min.js".to_string(),
                    "**/.*".to_string(),
                ]);
                context.priority_patterns = Some(vec![
                    "index.{js,ts,jsx,tsx}".to_string(),
                    "App.{js,ts,jsx,tsx}".to_string(),
                    "main.{js,ts}".to_string(),
                ]);
            },
            Some("python") => {
                context.include = Some(vec![
                    "src/**/*.py".to_string(),
                    "tests/**/*.py".to_string(),
                    "*.py".to_string(),
                    "pyproject.toml".to_string(),
                    "setup.py".to_string(),
                    "requirements.txt".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "__pycache__/**".to_string(),
                    "*.pyc".to_string(),
                    "venv/**".to_string(),
                    ".venv/**".to_string(),
                    "env/**".to_string(),
                    ".env/**".to_string(),
                    "dist/**".to_string(),
                    "build/**".to_string(),
                    "**/.*".to_string(),
                ]);
                context.priority_patterns = Some(vec![
                    "main.py".to_string(),
                    "__init__.py".to_string(),
                    "app.py".to_string(),
                ]);
            },
            Some("go") => {
                context.include = Some(vec![
                    "**/*.go".to_string(),
                    "go.mod".to_string(),
                    "go.sum".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "vendor/**".to_string(),
                    "bin/**".to_string(),
                    "**/.*".to_string(),
                ]);
                context.priority_patterns = Some(vec![
                    "main.go".to_string(),
                    "cmd/**/*.go".to_string(),
                ]);
            },
            Some("java") => {
                context.include = Some(vec![
                    "src/**/*.java".to_string(),
                    "src/**/*.kt".to_string(),
                    "test/**/*.java".to_string(),
                    "pom.xml".to_string(),
                    "build.gradle".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "target/**".to_string(),
                    "build/**".to_string(),
                    ".gradle/**".to_string(),
                    "**/*.class".to_string(),
                    "**/.*".to_string(),
                ]);
            },
            _ => {
                // Generic defaults
                context.include = Some(vec![
                    "src/**".to_string(),
                    "lib/**".to_string(),
                    "tests/**".to_string(),
                    "README.md".to_string(),
                ]);
                context.exclude = Some(vec![
                    "target/**".to_string(),
                    "build/**".to_string(),
                    "dist/**".to_string(),
                    "node_modules/**".to_string(),
                    "**/.*".to_string(),
                ]);
            }
        }
        
        context
    }
    
    /// Apply project-type specific configuration defaults
    fn apply_project_type_defaults(&self, config: &mut ProjectConfig, project_type: &str) {
        match project_type {
            "rust" => {
                // Rust-specific defaults
                if let Some(git_config) = &mut config.git {
                    git_config.conventional_commits = Some(true);
                }
                
                if let Some(templates) = &mut config.templates {
                    templates.default_review = Some("rust-code-review".to_string());
                    templates.default_docs = Some("rust-documentation".to_string());
                }
            },
            "javascript" | "typescript" | "react" | "nextjs" => {
                // JavaScript/TypeScript defaults
                if let Some(git_config) = &mut config.git {
                    git_config.conventional_commits = Some(true);
                }
                
                if let Some(templates) = &mut config.templates {
                    templates.default_review = Some("javascript-code-review".to_string());
                    templates.default_docs = Some("javascript-documentation".to_string());
                }
            },
            "python" => {
                // Python-specific defaults
                if let Some(quality) = &mut config.quality {
                    quality.security_scan = Some(true);
                }
                
                if let Some(templates) = &mut config.templates {
                    templates.default_review = Some("python-code-review".to_string());
                    templates.default_docs = Some("python-documentation".to_string());
                }
            },
            _ => {}
        }
    }
    
    /// Get configuration templates for different project types
    pub fn get_project_type_templates() -> HashMap<String, ProjectConfig> {
        let mut templates = HashMap::new();
        
        // Add templates for each supported project type
        templates.insert("rust".to_string(), Self::create_rust_template());
        templates.insert("javascript".to_string(), Self::create_javascript_template());
        templates.insert("python".to_string(), Self::create_python_template());
        templates.insert("go".to_string(), Self::create_go_template());
        templates.insert("java".to_string(), Self::create_java_template());
        
        templates
    }
    
    /// Create Rust project template
    fn create_rust_template() -> ProjectConfig {
        let mut config = ProjectConfig::new();
        
        config.project = Some(ProjectMetadata {
            name: "rust-project".to_string(),
            project_type: Some("rust".to_string()),
            description: Some("Rust project configuration template".to_string()),
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        config.context = Some(ContextConfig {
            max_tokens: Some(8000),
            include: Some(vec![
                "src/**/*.rs".to_string(),
                "tests/**/*.rs".to_string(),
                "examples/**/*.rs".to_string(),
                "Cargo.toml".to_string(),
                "README.md".to_string(),
            ]),
            exclude: Some(vec![
                "target/**".to_string(),
                "**/*.rlib".to_string(),
            ]),
            priority_patterns: Some(vec![
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "mod.rs".to_string(),
            ]),
            base_include: None,
            file_types: None,
            chunking: None,
        });
        
        config
    }
    
    /// Create JavaScript project template
    fn create_javascript_template() -> ProjectConfig {
        let mut config = ProjectConfig::new();
        
        config.project = Some(ProjectMetadata {
            name: "javascript-project".to_string(),
            project_type: Some("javascript".to_string()),
            description: Some("JavaScript project configuration template".to_string()),
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        config.context = Some(ContextConfig {
            max_tokens: Some(6000),
            include: Some(vec![
                "src/**/*.{js,ts,jsx,tsx}".to_string(),
                "tests/**/*.{js,ts}".to_string(),
                "package.json".to_string(),
                "README.md".to_string(),
            ]),
            exclude: Some(vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
                "coverage/**".to_string(),
            ]),
            priority_patterns: Some(vec![
                "index.{js,ts,jsx,tsx}".to_string(),
                "App.{js,ts,jsx,tsx}".to_string(),
            ]),
            base_include: None,
            file_types: None,
            chunking: None,
        });
        
        config
    }
    
    /// Create Python project template
    fn create_python_template() -> ProjectConfig {
        let mut config = ProjectConfig::new();
        
        config.project = Some(ProjectMetadata {
            name: "python-project".to_string(),
            project_type: Some("python".to_string()),
            description: Some("Python project configuration template".to_string()),
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        config.context = Some(ContextConfig {
            max_tokens: Some(8000),
            include: Some(vec![
                "src/**/*.py".to_string(),
                "tests/**/*.py".to_string(),
                "*.py".to_string(),
                "pyproject.toml".to_string(),
                "requirements.txt".to_string(),
                "README.md".to_string(),
            ]),
            exclude: Some(vec![
                "__pycache__/**".to_string(),
                "*.pyc".to_string(),
                "venv/**".to_string(),
                ".venv/**".to_string(),
                "dist/**".to_string(),
            ]),
            priority_patterns: Some(vec![
                "main.py".to_string(),
                "__init__.py".to_string(),
                "app.py".to_string(),
            ]),
            base_include: None,
            file_types: None,
            chunking: None,
        });
        
        config
    }
    
    /// Create Go project template
    fn create_go_template() -> ProjectConfig {
        let mut config = ProjectConfig::new();
        
        config.project = Some(ProjectMetadata {
            name: "go-project".to_string(),
            project_type: Some("go".to_string()),
            description: Some("Go project configuration template".to_string()),
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        config.context = Some(ContextConfig {
            max_tokens: Some(8000),
            include: Some(vec![
                "**/*.go".to_string(),
                "go.mod".to_string(),
                "go.sum".to_string(),
                "README.md".to_string(),
            ]),
            exclude: Some(vec![
                "vendor/**".to_string(),
                "bin/**".to_string(),
            ]),
            priority_patterns: Some(vec![
                "main.go".to_string(),
                "cmd/**/*.go".to_string(),
            ]),
            base_include: None,
            file_types: None,
            chunking: None,
        });
        
        config
    }
    
    /// Create Java project template
    fn create_java_template() -> ProjectConfig {
        let mut config = ProjectConfig::new();
        
        config.project = Some(ProjectMetadata {
            name: "java-project".to_string(),
            project_type: Some("java".to_string()),
            description: Some("Java project configuration template".to_string()),
            version: Some("1.0".to_string()),
            standards: None,
        });
        
        config.context = Some(ContextConfig {
            max_tokens: Some(8000),
            include: Some(vec![
                "src/**/*.java".to_string(),
                "test/**/*.java".to_string(),
                "pom.xml".to_string(),
                "build.gradle".to_string(),
                "README.md".to_string(),
            ]),
            exclude: Some(vec![
                "target/**".to_string(),
                "build/**".to_string(),
                ".gradle/**".to_string(),
                "**/*.class".to_string(),
            ]),
            priority_patterns: Some(vec![
                "Main.java".to_string(),
                "Application.java".to_string(),
            ]),
            base_include: None,
            file_types: None,
            chunking: None,
        });
        
        config
    }
}