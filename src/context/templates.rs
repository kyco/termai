use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context templates for common development workflows
/// These provide pre-configured smart context discovery settings
/// for typical development scenarios and tasks

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTemplate {
    pub name: String,
    pub description: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub priority_patterns: Vec<String>,
    pub max_tokens: Option<usize>,
    pub focus_areas: Vec<String>,
    pub suggested_queries: Vec<String>,
}

impl ContextTemplate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        description: String,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        priority_patterns: Vec<String>,
        max_tokens: Option<usize>,
        focus_areas: Vec<String>,
        suggested_queries: Vec<String>,
    ) -> Self {
        Self {
            name,
            description,
            include_patterns,
            exclude_patterns,
            priority_patterns,
            max_tokens,
            focus_areas,
            suggested_queries,
        }
    }

    /// Apply this template to a ContextConfig
    pub fn apply_to_config(&self, config: &mut crate::context::config::ContextConfig) {
        // Merge include patterns (template patterns take priority)
        let mut combined_includes = self.include_patterns.clone();
        for pattern in &config.context.include {
            if !combined_includes.contains(pattern) {
                combined_includes.push(pattern.clone());
            }
        }
        config.context.include = combined_includes;

        // Merge exclude patterns
        let mut combined_excludes = self.exclude_patterns.clone();
        for pattern in &config.context.exclude {
            if !combined_excludes.contains(pattern) {
                combined_excludes.push(pattern.clone());
            }
        }
        config.context.exclude = combined_excludes;

        // Merge priority patterns
        let mut combined_priorities = self.priority_patterns.clone();
        for pattern in &config.context.priority_patterns {
            if !combined_priorities.contains(pattern) {
                combined_priorities.push(pattern.clone());
            }
        }
        config.context.priority_patterns = combined_priorities;

        // Override max tokens if template specifies one
        if let Some(max_tokens) = self.max_tokens {
            config.context.max_tokens = max_tokens;
        }
    }
}

/// Collection of built-in context templates for common workflows
pub struct ContextTemplateLibrary;

impl ContextTemplateLibrary {
    /// Get all available templates
    pub fn get_all_templates() -> HashMap<String, ContextTemplate> {
        let mut templates = HashMap::new();

        templates.insert("security".to_string(), Self::security_template());
        templates.insert("refactoring".to_string(), Self::refactoring_template());
        templates.insert("testing".to_string(), Self::testing_template());
        templates.insert("api".to_string(), Self::api_template());
        templates.insert("database".to_string(), Self::database_template());
        templates.insert("performance".to_string(), Self::performance_template());
        templates.insert("documentation".to_string(), Self::documentation_template());
        templates.insert("debugging".to_string(), Self::debugging_template());
        templates.insert("architecture".to_string(), Self::architecture_template());
        templates.insert("deployment".to_string(), Self::deployment_template());

        templates
    }

    /// Get a specific template by name
    pub fn get_template(name: &str) -> Option<ContextTemplate> {
        Self::get_all_templates().get(name).cloned()
    }

    /// Security-focused template for code reviews and vulnerability analysis
    fn security_template() -> ContextTemplate {
        ContextTemplate::new(
            "Security Analysis".to_string(),
            "Focus on security-related code, authentication, authorization, input validation, and potential vulnerabilities".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.js".to_string(),
                "src/**/*.ts".to_string(),
                "src/**/*.py".to_string(),
                "**/auth*/**".to_string(),
                "**/security/**".to_string(),
                "**/middleware/**".to_string(),
                "**/*auth*.rs".to_string(),
                "**/*security*.rs".to_string(),
                "**/*crypto*.rs".to_string(),
                "**/*token*.rs".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "requirements.txt".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/test*/**".to_string(),
                "**/example*/**".to_string(),
            ],
            vec![
                "*auth*".to_string(),
                "*security*".to_string(),
                "*crypto*".to_string(),
                "*token*".to_string(),
                "middleware".to_string(),
                "main.rs".to_string(),
                "lib.rs".to_string(),
            ],
            Some(5000),
            vec![
                "Authentication".to_string(),
                "Authorization".to_string(),
                "Input validation".to_string(),
                "Cryptography".to_string(),
                "Session management".to_string(),
            ],
            vec![
                "Review this code for security vulnerabilities".to_string(),
                "Analyze authentication implementation".to_string(),
                "Check for input validation issues".to_string(),
                "Review cryptographic implementations".to_string(),
                "Identify potential security risks".to_string(),
            ],
        )
    }

    /// Refactoring template for code restructuring and improvements
    fn refactoring_template() -> ContextTemplate {
        ContextTemplate::new(
            "Code Refactoring".to_string(),
            "Focus on code structure, design patterns, modularity, and code quality improvements"
                .to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.js".to_string(),
                "src/**/*.ts".to_string(),
                "src/**/*.py".to_string(),
                "src/**/*.go".to_string(),
                "src/**/*.java".to_string(),
                "src/**/*.kt".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/vendor/**".to_string(),
                "**/build/**".to_string(),
                "**/dist/**".to_string(),
            ],
            vec![
                "main.*".to_string(),
                "lib.*".to_string(),
                "mod.rs".to_string(),
                "index.*".to_string(),
                "__init__.py".to_string(),
            ],
            Some(6000),
            vec![
                "Code structure".to_string(),
                "Design patterns".to_string(),
                "Code duplication".to_string(),
                "Modularity".to_string(),
                "Function extraction".to_string(),
            ],
            vec![
                "Refactor this code to improve readability".to_string(),
                "Extract common functionality into modules".to_string(),
                "Improve the structure of this codebase".to_string(),
                "Suggest design pattern improvements".to_string(),
                "Reduce code duplication".to_string(),
            ],
        )
    }

    /// Testing template focused on test code and testing strategies
    fn testing_template() -> ContextTemplate {
        ContextTemplate::new(
            "Testing & Quality Assurance".to_string(),
            "Focus on test files, testing frameworks, and code coverage analysis".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "tests/**/*.rs".to_string(),
                "test/**/*.js".to_string(),
                "**/*test*.py".to_string(),
                "**/*spec*.js".to_string(),
                "__tests__/**".to_string(),
                "spec/**".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "pytest.ini".to_string(),
                "jest.config.js".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "coverage/**".to_string(),
                "**/*.log".to_string(),
            ],
            vec![
                "*test*".to_string(),
                "*spec*".to_string(),
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "mod.rs".to_string(),
            ],
            Some(4500),
            vec![
                "Unit testing".to_string(),
                "Integration testing".to_string(),
                "Test coverage".to_string(),
                "Mocking".to_string(),
                "Test automation".to_string(),
            ],
            vec![
                "Write comprehensive tests for this module".to_string(),
                "Improve test coverage".to_string(),
                "Review and refactor existing tests".to_string(),
                "Add integration tests".to_string(),
                "Suggest testing strategies".to_string(),
            ],
        )
    }

    /// API development template for REST APIs and web services
    fn api_template() -> ContextTemplate {
        ContextTemplate::new(
            "API Development".to_string(),
            "Focus on API routes, controllers, middleware, request/response handling, and API documentation".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "**/routes/**".to_string(),
                "**/controllers/**".to_string(),
                "**/handlers/**".to_string(),
                "**/middleware/**".to_string(),
                "**/api/**".to_string(),
                "**/*route*".to_string(),
                "**/*handler*".to_string(),
                "**/*controller*".to_string(),
                "openapi.yaml".to_string(),
                "swagger.yaml".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/static/**".to_string(),
                "**/assets/**".to_string(),
            ],
            vec![
                "*route*".to_string(),
                "*handler*".to_string(),
                "*controller*".to_string(),
                "*api*".to_string(),
                "main.rs".to_string(),
                "server.rs".to_string(),
            ],
            Some(5500),
            vec![
                "REST API design".to_string(),
                "Request handling".to_string(),
                "Response formatting".to_string(),
                "Error handling".to_string(),
                "API documentation".to_string(),
            ],
            vec![
                "Design RESTful API endpoints".to_string(),
                "Implement proper error handling for APIs".to_string(),
                "Add API documentation".to_string(),
                "Review API security measures".to_string(),
                "Optimize API performance".to_string(),
            ],
        )
    }

    /// Database template for data access and persistence layer
    fn database_template() -> ContextTemplate {
        ContextTemplate::new(
            "Database & Data Layer".to_string(),
            "Focus on database schemas, queries, ORM models, migrations, and data access patterns"
                .to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "**/models/**".to_string(),
                "**/entities/**".to_string(),
                "**/repositories/**".to_string(),
                "**/migrations/**".to_string(),
                "**/database/**".to_string(),
                "**/*model*".to_string(),
                "**/*repository*".to_string(),
                "**/*entity*".to_string(),
                "schema.sql".to_string(),
                "migrations/**".to_string(),
                "diesel.toml".to_string(),
                "Cargo.toml".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "**/*.log".to_string(),
                "**/temp/**".to_string(),
                "**/*.db-wal".to_string(),
                "**/*.db-shm".to_string(),
            ],
            vec![
                "*model*".to_string(),
                "*entity*".to_string(),
                "*repository*".to_string(),
                "*database*".to_string(),
                "schema*".to_string(),
                "migration*".to_string(),
            ],
            Some(4000),
            vec![
                "Database schema".to_string(),
                "Data modeling".to_string(),
                "Query optimization".to_string(),
                "Migrations".to_string(),
                "Repository pattern".to_string(),
            ],
            vec![
                "Design database schema".to_string(),
                "Optimize database queries".to_string(),
                "Create migration scripts".to_string(),
                "Review data access patterns".to_string(),
                "Add database validation".to_string(),
            ],
        )
    }

    /// Performance optimization template
    fn performance_template() -> ContextTemplate {
        ContextTemplate::new(
            "Performance Optimization".to_string(),
            "Focus on performance-critical code, algorithms, caching, and optimization opportunities".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.js".to_string(),
                "src/**/*.py".to_string(),
                "**/*perf*".to_string(),
                "**/*optimize*".to_string(),
                "**/*cache*".to_string(),
                "**/*algorithm*".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/test*/**".to_string(),
                "**/benchmark*/**".to_string(),
            ],
            vec![
                "*perf*".to_string(),
                "*optimize*".to_string(),
                "*cache*".to_string(),
                "*algorithm*".to_string(),
                "main.rs".to_string(),
                "core*".to_string(),
            ],
            Some(4500),
            vec![
                "Algorithm optimization".to_string(),
                "Memory usage".to_string(),
                "Caching strategies".to_string(),
                "Concurrency".to_string(),
                "Profiling".to_string(),
            ],
            vec![
                "Optimize this code for performance".to_string(),
                "Identify performance bottlenecks".to_string(),
                "Suggest caching strategies".to_string(),
                "Improve algorithm efficiency".to_string(),
                "Reduce memory usage".to_string(),
            ],
        )
    }

    /// Documentation template for generating and improving documentation
    fn documentation_template() -> ContextTemplate {
        ContextTemplate::new(
            "Documentation".to_string(),
            "Focus on code documentation, README files, API docs, and code comments".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.js".to_string(),
                "src/**/*.py".to_string(),
                "README.md".to_string(),
                "CHANGELOG.md".to_string(),
                "docs/**".to_string(),
                "**/*.md".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/build/**".to_string(),
                "**/.git/**".to_string(),
            ],
            vec![
                "README*".to_string(),
                "CHANGELOG*".to_string(),
                "lib.rs".to_string(),
                "main.rs".to_string(),
                "mod.rs".to_string(),
                "index.*".to_string(),
            ],
            Some(5000),
            vec![
                "Code comments".to_string(),
                "API documentation".to_string(),
                "User guides".to_string(),
                "Examples".to_string(),
                "Architecture docs".to_string(),
            ],
            vec![
                "Generate comprehensive documentation".to_string(),
                "Improve code comments and docstrings".to_string(),
                "Create user-friendly README".to_string(),
                "Document API endpoints".to_string(),
                "Add usage examples".to_string(),
            ],
        )
    }

    /// Debugging template for troubleshooting and error investigation
    fn debugging_template() -> ContextTemplate {
        ContextTemplate::new(
            "Debugging & Troubleshooting".to_string(),
            "Focus on error handling, logging, debugging utilities, and problem investigation"
                .to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/*.js".to_string(),
                "src/**/*.py".to_string(),
                "**/*error*".to_string(),
                "**/*log*".to_string(),
                "**/*debug*".to_string(),
                "**/*trace*".to_string(),
                "logs/**".to_string(),
                "Cargo.toml".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.tmp".to_string(),
                "**/coverage/**".to_string(),
            ],
            vec![
                "*error*".to_string(),
                "*log*".to_string(),
                "*debug*".to_string(),
                "main.rs".to_string(),
                "lib.rs".to_string(),
            ],
            Some(4000),
            vec![
                "Error handling".to_string(),
                "Logging".to_string(),
                "Stack traces".to_string(),
                "Debug utilities".to_string(),
                "Error reporting".to_string(),
            ],
            vec![
                "Debug this error or issue".to_string(),
                "Improve error handling and logging".to_string(),
                "Trace the source of this problem".to_string(),
                "Add better debugging information".to_string(),
                "Fix runtime errors".to_string(),
            ],
        )
    }

    /// Architecture template for system design and architectural decisions
    fn architecture_template() -> ContextTemplate {
        ContextTemplate::new(
            "Architecture & Design".to_string(),
            "Focus on system architecture, design patterns, module structure, and high-level organization".to_string(),
            vec![
                "src/**/*.rs".to_string(),
                "src/**/mod.rs".to_string(),
                "src/**/lib.rs".to_string(),
                "src/**/main.rs".to_string(),
                "**/*service*".to_string(),
                "**/*module*".to_string(),
                "**/*component*".to_string(),
                "docs/architecture/**".to_string(),
                "Cargo.toml".to_string(),
                "README.md".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "**/*.log".to_string(),
                "**/test*/**".to_string(),
                "**/example*/**".to_string(),
            ],
            vec![
                "main.rs".to_string(),
                "lib.rs".to_string(),
                "mod.rs".to_string(),
                "*service*".to_string(),
                "*module*".to_string(),
                "*component*".to_string(),
            ],
            Some(6000),
            vec![
                "System architecture".to_string(),
                "Design patterns".to_string(),
                "Module organization".to_string(),
                "Component design".to_string(),
                "Architectural decisions".to_string(),
            ],
            vec![
                "Review system architecture".to_string(),
                "Improve module organization".to_string(),
                "Suggest architectural improvements".to_string(),
                "Analyze design patterns".to_string(),
                "Plan system refactoring".to_string(),
            ],
        )
    }

    /// Deployment template for CI/CD and deployment configurations
    fn deployment_template() -> ContextTemplate {
        ContextTemplate::new(
            "Deployment & DevOps".to_string(),
            "Focus on deployment scripts, CI/CD pipelines, Docker configurations, and infrastructure code".to_string(),
            vec![
                "Dockerfile".to_string(),
                "docker-compose.yml".to_string(),
                ".github/workflows/**".to_string(),
                ".gitlab-ci.yml".to_string(),
                "deploy/**".to_string(),
                "scripts/**".to_string(),
                "Cargo.toml".to_string(),
                "package.json".to_string(),
                "README.md".to_string(),
                "**/*deploy*".to_string(),
                "**/*build*".to_string(),
            ],
            vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.log".to_string(),
                "**/coverage/**".to_string(),
                "src/**".to_string(), // Less focus on source code
            ],
            vec![
                "Dockerfile".to_string(),
                "*deploy*".to_string(),
                "*build*".to_string(),
                "*.yml".to_string(),
                "*.yaml".to_string(),
                "*.sh".to_string(),
            ],
            Some(3500),
            vec![
                "CI/CD pipelines".to_string(),
                "Container configuration".to_string(),
                "Deployment scripts".to_string(),
                "Infrastructure as code".to_string(),
                "Build automation".to_string(),
            ],
            vec![
                "Set up CI/CD pipeline".to_string(),
                "Improve deployment process".to_string(),
                "Containerize this application".to_string(),
                "Automate build and deployment".to_string(),
                "Review deployment configuration".to_string(),
            ],
        )
    }

    /// Get template names and descriptions
    pub fn list_templates() -> Vec<(String, String)> {
        let templates = Self::get_all_templates();
        templates
            .iter()
            .map(|(name, template)| (name.clone(), template.description.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_library() {
        let templates = ContextTemplateLibrary::get_all_templates();

        // Should have all expected templates
        assert!(templates.contains_key("security"));
        assert!(templates.contains_key("refactoring"));
        assert!(templates.contains_key("testing"));
        assert!(templates.contains_key("api"));
        assert!(templates.contains_key("database"));

        // Test specific template
        let security_template = templates.get("security").unwrap();
        assert_eq!(security_template.name, "Security Analysis");
        assert!(!security_template.include_patterns.is_empty());
        assert!(!security_template.suggested_queries.is_empty());
    }

    #[test]
    fn test_template_application() {
        let mut config = crate::context::config::ContextConfig::default();
        let original_includes = config.context.include.clone();

        let security_template = ContextTemplateLibrary::security_template();
        security_template.apply_to_config(&mut config);

        // Should have merged includes
        assert!(config.context.include.len() >= original_includes.len());
        assert!(config.context.include.len() >= security_template.include_patterns.len());

        // Should include security-specific patterns
        assert!(config.context.include.contains(&"**/auth*/**".to_string()));
    }

    #[test]
    fn test_get_specific_template() {
        let template = ContextTemplateLibrary::get_template("testing");
        assert!(template.is_some());

        let testing_template = template.unwrap();
        assert_eq!(testing_template.name, "Testing & Quality Assurance");
        assert!(testing_template
            .focus_areas
            .contains(&"Unit testing".to_string()));
    }

    #[test]
    fn test_list_templates() {
        let template_list = ContextTemplateLibrary::list_templates();
        assert!(!template_list.is_empty());

        // Should contain expected templates
        let names: Vec<String> = template_list.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"security".to_string()));
        assert!(names.contains(&"refactoring".to_string()));
        assert!(names.contains(&"testing".to_string()));
    }

    #[test]
    fn test_template_structure() {
        let template = ContextTemplateLibrary::api_template();

        // Should have all required fields
        assert!(!template.name.is_empty());
        assert!(!template.description.is_empty());
        assert!(!template.include_patterns.is_empty());
        assert!(!template.exclude_patterns.is_empty());
        assert!(!template.priority_patterns.is_empty());
        assert!(template.max_tokens.is_some());
        assert!(!template.focus_areas.is_empty());
        assert!(!template.suggested_queries.is_empty());
    }

    #[test]
    fn test_performance_template_specifics() {
        let template = ContextTemplateLibrary::performance_template();

        // Should focus on performance-related patterns
        assert!(template.include_patterns.contains(&"**/*perf*".to_string()));
        assert!(template
            .include_patterns
            .contains(&"**/*optimize*".to_string()));
        assert!(template.priority_patterns.contains(&"*perf*".to_string()));
        assert!(template
            .focus_areas
            .contains(&"Algorithm optimization".to_string()));

        // Should have performance-related queries
        assert!(template
            .suggested_queries
            .iter()
            .any(|q| q.contains("performance") || q.contains("optimize")));
    }
}
