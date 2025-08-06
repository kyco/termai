# Task: Project Configuration System

## Overview
Implement a flexible project-specific configuration system using `.termai.toml` files to customize behavior, context rules, and preferences per project.

## Success Criteria
- [ ] 80% of active projects have custom TermAI configuration
- [ ] Context discovery accuracy improves to >95% with project config
- [ ] Team consistency increases through shared configuration
- [ ] Setup complexity reduces through intelligent defaults

## Implementation Tasks

### 1. Configuration File Structure Design
- [ ] Design `.termai.toml` schema with comprehensive options
- [ ] Create configuration hierarchy (global -> project -> user)
- [ ] Implement configuration validation and error handling
- [ ] Add configuration versioning for compatibility
- [ ] Support configuration inheritance and overrides

### 2. Context Configuration
- [ ] Implement file inclusion/exclusion pattern configuration
- [ ] Add project structure detection overrides
- [ ] Support custom context token limits per project
- [ ] Create context priority and weighting rules
- [ ] Add file type preferences and handling rules

### 3. Provider and Model Configuration
- [ ] Allow project-specific provider preferences
- [ ] Support model selection based on task type
- [ ] Add API key configuration per project (secure storage)
- [ ] Implement provider fallback chains
- [ ] Support custom model parameters per project

### 4. Workflow Integration Settings
- [ ] Configure Git integration preferences
- [ ] Set up project-specific templates and presets
- [ ] Define custom output formatting rules
- [ ] Add workflow automation settings
- [ ] Configure team collaboration preferences

### 5. Configuration Discovery and Loading
- [ ] Implement configuration file discovery algorithm
- [ ] Add configuration caching for performance
- [ ] Support configuration reloading without restart
- [ ] Create configuration validation on load
- [ ] Handle configuration conflicts and resolution

### 6. Configuration Management Commands
- [ ] Create `termai config init` for project setup
- [ ] Add `termai config show` for viewing current config
- [ ] Implement `termai config validate` for syntax checking
- [ ] Add `termai config edit` for interactive editing
- [ ] Create `termai config sync` for team synchronization

### 7. Team Configuration Features
- [ ] Support shared configuration repositories
- [ ] Add configuration templates for common project types
- [ ] Implement configuration locking for team standards
- [ ] Create configuration diffing and merging
- [ ] Add configuration review and approval workflows

### 8. Advanced Configuration Options
- [ ] Support environment-specific configurations
- [ ] Add conditional configuration based on git branch
- [ ] Implement configuration profiles for different workflows
- [ ] Support configuration includes and modularity
- [ ] Add dynamic configuration based on project state

### 9. Configuration Management
- [ ] Implement configuration validation and error handling
- [ ] Add configuration templates for common project types

**Note**: Backwards compatibility is not a concern - existing configurations will be replaced with the new format.

### 10. Security and Privacy Configuration
- [ ] Configure redaction patterns per project
- [ ] Set up secure API key management
- [ ] Add privacy level settings per project type
- [ ] Configure data retention and cleanup policies
- [ ] Implement audit logging configuration

### 11. Integration with Other Features
- [ ] Connect configuration with smart context discovery
- [ ] Integrate with template and preset systems
- [ ] Link configuration to session management
- [ ] Connect with Git workflow settings
- [ ] Integrate with output formatting preferences

### 12. Testing
- [ ] Unit tests for configuration parsing and validation
- [ ] Integration tests with different project types
- [ ] Configuration hierarchy resolution tests
- [ ] Performance tests with complex configurations
- [ ] Team workflow testing scenarios

### 13. Documentation
- [ ] Create comprehensive configuration reference guide
- [ ] Add project setup best practices
- [ ] Document configuration examples for common scenarios
- [ ] Create team configuration management guide
- [ ] Add troubleshooting guide for configuration issues

## File Changes Required

### New Files
- `src/config/project.rs` - Project-specific configuration
- `src/config/loader.rs` - Configuration discovery and loading
- `src/config/schema.rs` - Configuration schema definition
- `src/config/migration.rs` - Configuration migration utilities
- `templates/` - Configuration templates for different project types

### Modified Files
- `src/config/service/config_service.rs` - Integrate project config
- `src/main.rs` - Add config subcommands
- `src/args.rs` - Add configuration arguments
- `Cargo.toml` - Add configuration dependencies

## Dependencies to Add
```toml
[dependencies]
toml = "0.8"           # TOML parsing and generation
serde = { version = "1.0", features = ["derive"] }
config = "0.14"        # Layered configuration management
jsonschema = "0.18"    # Configuration validation
dirs = "5.0"           # Directory discovery
```

## Configuration Schema Examples

### Basic Project Configuration
```toml
# .termai.toml
[project]
name = "termAI"
type = "rust"
description = "Terminal AI assistant"
version = "1.0"

[context]
max_tokens = 8000
include = [
    "src/**/*.rs",
    "tests/**/*.rs", 
    "Cargo.toml",
    "README.md"
]
exclude = [
    "target/**",
    "**/*.log",
    "**/node_modules/**"
]
priority_patterns = ["main.rs", "lib.rs", "mod.rs"]

[providers]
default = "claude"
fallback = "openapi"

[providers.claude]
model = "claude-3-sonnet"
max_tokens = 4000
temperature = 0.3

[providers.openapi]
model = "gpt-4"
max_tokens = 4000
temperature = 0.3

[git]
auto_commit_messages = true
review_on_push = false
conventional_commits = true

[output]
theme = "dark"
streaming = true
syntax_highlighting = true
export_format = "markdown"

[templates]
default_review = "code-review-rust"
default_docs = "rust-documentation"

[redaction]
patterns = [
    "API_KEY_.*",
    "SECRET_.*",
    "PASSWORD.*"
]
```

### Team Configuration Template
```toml
# .termai.toml (team template)
[project]
type = "javascript"
standards = "company-js-2024"

[context]
max_tokens = 6000
include = [
    "src/**/*.{js,ts,jsx,tsx}",
    "tests/**/*.{js,ts}",
    "package.json",
    "README.md",
    "docs/**/*.md"
]
exclude = [
    "node_modules/**",
    "dist/**",
    "coverage/**",
    "**/*.min.js"
]

[providers]
default = "claude"
# Locked by team policy
locked = true

[git]
auto_commit_messages = true
conventional_commits = true
commit_template = "team-commit-template"

[quality]
auto_review = true
review_depth = "thorough"
security_scan = true

[team]
shared_presets = "https://github.com/company/termai-presets"
sync_frequency = "daily"
```

### Environment-Specific Configuration
```toml
[project]
name = "myapp"

[context]
base_include = ["src/**", "tests/**"]

# Development environment
[env.development]
context.max_tokens = 8000
providers.default = "claude"
git.auto_commit_messages = false

# Production environment  
[env.production]
context.max_tokens = 4000
providers.default = "openapi"
redaction.strict_mode = true
```

## Command Examples

### Configuration Initialization
```bash
# Initialize project configuration
termai config init
> 🏗️  Initializing TermAI configuration for current project
> 
> Project type detected: Rust
> Repository: git@github.com:user/termAI.git
> 
> [1/5] Context configuration:
>   Include patterns: src/**/*.rs, tests/**/*.rs, Cargo.toml [Y/n]: 
>   Max context tokens [4000]: 8000
> 
> [2/5] Provider preferences:
>   Default provider [claude]: 
>   Fallback provider [openapi]: 
> 
> ✅ Configuration saved to .termai.toml

# Initialize from template
termai config init --template rust-library
termai config init --template javascript-react
```

### Configuration Management
```bash
# Show current configuration
termai config show
> 📋 Current TermAI Configuration
> 
> Project: termAI (rust)
> Config sources:
>   • Global: ~/.config/termai/config.toml
>   • Project: ./.termai.toml ✓
> 
> Context:
>   Max tokens: 8,000
>   Include: 15 patterns
>   Exclude: 3 patterns
> 
> Providers:
>   Default: Claude (claude-3-sonnet)
>   Fallback: OpenAI (gpt-4)

# Validate configuration
termai config validate
> ✅ Configuration is valid
> 
> Warnings:
>   • Context may exceed token limits for large files
>   • Consider adding test files to exclusion patterns

# Edit configuration interactively
termai config edit
> Opens .termai.toml in default editor with validation
```

### Team Configuration Sync
```bash
# Sync with team configuration repository
termai config sync --team-repo https://github.com/company/termai-config
> 🔄 Syncing with team configuration...
> 
> Updates available:
>   • New preset: code-review-security
>   • Updated template: api-documentation
>   • Policy change: require conventional commits
> 
> Apply updates? [Y/n]: y
> ✅ Configuration synchronized

# Share current configuration
termai config export --file team-config.toml
termai config import team-config.toml --merge
```

## Configuration Discovery Algorithm
```rust
// Configuration loading priority order
fn discover_configuration() -> Vec<ConfigPath> {
    vec![
        // 1. Command line arguments (highest priority)
        CommandLineConfig,
        
        // 2. Environment variables
        EnvironmentConfig,
        
        // 3. Project-specific config (current directory up to git root)
        ProjectConfig(".termai.toml"),
        ProjectConfig(".termai/config.toml"),
        
        // 4. User-specific config
        UserConfig("~/.config/termai/config.toml"),
        
        // 5. System-wide config (lowest priority)
        SystemConfig("/etc/termai/config.toml"),
    ]
}
```

## Success Metrics
- Project configuration adoption: >80% of active projects
- Context accuracy improvement: >95% with project config
- Team consistency: 60% reduction in configuration variations
- Setup time reduction: 70% faster project onboarding
- Configuration errors: <5% of configurations have validation issues

## Risk Mitigation
- **Risk**: Configuration complexity overwhelming users
  - **Mitigation**: Provide sensible defaults and templates
- **Risk**: Configuration conflicts between team members
  - **Mitigation**: Clear hierarchy rules and conflict resolution
- **Risk**: Security issues with configuration sharing
  - **Mitigation**: Separate secret management from configuration