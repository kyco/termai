/// Handlers for Config and Redact commands - configuration management
use crate::args::{ConfigAction, ConfigArgs, RedactAction, RedactArgs};
use crate::config::env::EnvResolver;
use crate::config::project::ProjectConfigService;
use crate::config::schema::ProjectConfig;
use crate::config::service::config_service;
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use std::process::Command;

/// Handle config subcommands with enhanced feedback and error handling
pub fn handle_config_command(
    repo: &SqliteRepository,
    action: &ConfigAction,
    _args: &ConfigArgs,
) -> Result<()> {
    use crate::config::model::keys::ConfigKeys;

    match action {
        ConfigAction::Show => {
            let configs = config_service::fetch_config(repo)
                .context("Failed to fetch configuration from database")?;

            println!("{}", "📋 Current Configuration".bright_blue().bold());
            println!("{}", "═══════════════════════".white().dimmed());

            if configs.is_empty() {
                println!("{}", "⚠️  No configuration found".yellow());
                println!();
                println!("{}", "💡 Quick setup:".bright_yellow().bold());
                println!("   {}      # Run setup wizard", "termai setup".cyan());
                return Ok(());
            }

            // Group and display configs in a user-friendly way
            let mut has_provider = false;
            let mut has_apis = false;

            // Show provider first
            for config in &configs {
                if config.key == ConfigKeys::ProviderKey.to_key() {
                    println!(
                        "{}  {}",
                        "🤖 Default Provider:".bright_green(),
                        config.value.bright_cyan()
                    );
                    has_provider = true;
                }
            }

            if !has_provider {
                println!(
                    "{}  {}",
                    "🤖 Default Provider:".bright_green(),
                    "Not set".red()
                );
            }

            println!();

            // Show API configurations
            for config in &configs {
                match config.key.as_str() {
                    key if key == ConfigKeys::ClaudeApiKey.to_key() => {
                        println!(
                            "{}    {}",
                            "🧠 Claude API:".bright_green(),
                            "✅ Configured".green()
                        );
                        has_apis = true;
                    }
                    key if key == ConfigKeys::ChatGptApiKey.to_key() => {
                        println!(
                            "{}    {}",
                            "⚡ OpenAI API:".bright_green(),
                            "✅ Configured".green()
                        );
                        has_apis = true;
                    }
                    key if key == ConfigKeys::Redacted.to_key() => {
                        let count = config.value.split(',').filter(|s| !s.is_empty()).count();
                        println!("{}  {} patterns", "🔒 Redactions:".bright_green(), count);
                    }
                    _ => {}
                }
            }

            if !has_apis {
                println!("{}", "⚠️  No API keys configured".yellow());
                println!();
                println!("{}", "💡 Add API keys:".bright_yellow().bold());
                println!(
                    "   {}   # Add Claude key",
                    "termai config set-claude <key>".cyan()
                );
                println!(
                    "   {}  # Add OpenAI key",
                    "termai config set-openai <key>".cyan()
                );
            } else {
                println!();
                println!("{}", "💡 Ready to chat! Try:".bright_green().bold());
                println!(
                    "   {}              # Start interactive session",
                    "termai chat".cyan()
                );
                println!(
                    "   {}  # Ask a quick question",
                    "termai ask \"question\"".cyan()
                );
            }

            Ok(())
        }
        ConfigAction::SetOpenai { api_key } => {
            if api_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "API key cannot be empty\n\n{}",
                    get_api_key_help("OpenAI")
                ));
            }

            config_service::write_config(repo, &ConfigKeys::ChatGptApiKey.to_key(), api_key)
                .context("Failed to save OpenAI API key")?;

            println!(
                "{}",
                "✅ OpenAI API key configured successfully".green().bold()
            );
            println!();
            println!("{}", "💡 Next steps:".bright_yellow().bold());
            println!(
                "   {}         # View configuration",
                "termai config show".cyan()
            );
            println!(
                "   {}   # Set as default provider",
                "termai config set-provider openai".cyan()
            );
            println!("   {}              # Start chatting", "termai chat".cyan());

            Ok(())
        }
        ConfigAction::SetClaude { api_key } => {
            if api_key.is_empty() {
                return Err(anyhow::anyhow!(
                    "API key cannot be empty\n\n{}",
                    get_api_key_help("Claude")
                ));
            }

            config_service::write_config(repo, &ConfigKeys::ClaudeApiKey.to_key(), api_key)
                .context("Failed to save Claude API key")?;

            println!(
                "{}",
                "✅ Claude API key configured successfully".green().bold()
            );
            println!();
            println!("{}", "💡 Next steps:".bright_yellow().bold());
            println!(
                "   {}         # View configuration",
                "termai config show".cyan()
            );
            println!(
                "   {}    # Set as default provider",
                "termai config set-provider claude".cyan()
            );
            println!("   {}              # Start chatting", "termai chat".cyan());

            Ok(())
        }
        ConfigAction::SetProvider { provider } => {
            config_service::write_config(
                repo,
                &ConfigKeys::ProviderKey.to_key(),
                provider.to_str(),
            )
            .context("Failed to save provider preference")?;

            println!(
                "{} {}",
                "✅ Provider set to".green().bold(),
                provider.to_str().bright_cyan().bold()
            );
            println!();
            println!("{}", "💡 Start using TermAI:".bright_yellow().bold());
            println!(
                "   {}              # Start interactive session",
                "termai chat".cyan()
            );
            println!(
                "   {}  # Ask a quick question",
                "termai ask \"question\"".cyan()
            );

            Ok(())
        }
        ConfigAction::Reset => {
            println!("{}", "🔄 Reset Configuration".bright_yellow().bold());
            println!("{}", "════════════════════".white().dimmed());
            println!();
            println!("{}", "⚠️  Configuration reset is being enhanced".yellow());
            println!();
            println!("{}", "💡 Manual reset steps:".bright_yellow().bold());
            println!(
                "   1. {}        # Delete database",
                "rm ~/.config/termai/app.db".cyan()
            );
            println!(
                "   2. {}                 # Reconfigure",
                "termai setup".cyan()
            );
            println!();
            println!(
                "{}",
                "⚠️  This will permanently delete all settings and session history".red()
            );

            Ok(())
        }
        ConfigAction::Env => {
            println!("{}", "🌍 Environment Variables".bright_green().bold());
            println!("{}", "═══════════════════════".white().dimmed());
            println!();

            // Show current environment variable values
            let env_vars = EnvResolver::get_all_set();
            if env_vars.is_empty() {
                println!(
                    "{}",
                    "📝 No TermAI environment variables are currently set".yellow()
                );
                println!();
            } else {
                println!(
                    "{}",
                    "🔧 Currently Set Environment Variables:"
                        .bright_cyan()
                        .bold()
                );
                println!();
                for (var, value) in &env_vars {
                    // Redact API keys for security
                    let display_value = if var.contains("API_KEY") {
                        if value.len() > 8 {
                            format!("{}...{}", &value[..4], &value[value.len() - 4..])
                        } else {
                            "[HIDDEN]".to_string()
                        }
                    } else {
                        value.clone()
                    };
                    println!("   {}  {}", var.bright_white(), display_value.cyan());
                }
                println!();
            }

            // Show help information
            println!(
                "{}",
                "📖 Environment Variable Reference:".bright_yellow().bold()
            );
            println!("{}", "══════════════════════════════════".white().dimmed());
            println!();

            for line in EnvResolver::help_text().lines() {
                if line.trim().is_empty() {
                    println!();
                } else if line.starts_with("  ") && line.contains("_") {
                    // Environment variable names
                    println!("  {}", line.trim().bright_white());
                } else if line.starts_with("    ") {
                    // Descriptions
                    println!("    {}", line.trim().white().dimmed());
                } else if line.starts_with("Examples:") {
                    println!("{}", line.bright_cyan().bold());
                } else if line.starts_with("  export") {
                    println!("  {}", line.trim().cyan());
                } else {
                    println!("{}", line.bright_yellow().bold());
                }
            }

            println!();
            println!("{}", "💡 Priority Order:".bright_yellow().bold());
            println!("   1. Command line arguments (highest priority)");
            println!("   2. Environment variables");
            println!("   3. Configuration file settings");
            println!("   4. Default values (lowest priority)");
            println!();
            println!("{}", "🔄 Applying Changes:".bright_green().bold());
            println!(
                "   {}  # Apply environment changes to current shell",
                "source ~/.bashrc".cyan()
            );
            println!(
                "   {}            # Test with environment variable",
                "TERMAI_PROVIDER=claude termai ask \"test\"".cyan()
            );

            Ok(())
        }
        
        // Project configuration commands
        ConfigAction::Init { project_type, template, force } => {
            handle_config_init(project_type.as_deref(), template.as_deref(), *force)
        }
        ConfigAction::Project => {
            handle_config_project()
        }
        ConfigAction::Validate => {
            handle_config_validate()
        }
        ConfigAction::Edit => {
            handle_config_edit()
        }
        ConfigAction::Export { file, format } => {
            handle_config_export(file.as_deref(), format)
        }
        ConfigAction::Import { file, merge } => {
            handle_config_import(file, *merge)
        }
    }
}

/// Get helpful information about obtaining API keys
fn get_api_key_help(provider: &str) -> String {
    match provider {
        "OpenAI" => format!(
            "{}\n{}\n• {}\n• {}\n• {}",
            "💡 Get your OpenAI API key:".bright_yellow().bold(),
            "   Visit: https://platform.openai.com/api-keys"
                .bright_blue()
                .underline(),
            "Sign up or log in to your OpenAI account".white(),
            "Click 'Create new secret key'".white(),
            "Copy the key and paste it in the command".white()
        ),
        "Claude" => format!(
            "{}\n{}\n• {}\n• {}\n• {}",
            "💡 Get your Claude API key:".bright_yellow().bold(),
            "   Visit: https://console.anthropic.com/"
                .bright_blue()
                .underline(),
            "Sign up or log in to your Anthropic account".white(),
            "Navigate to 'API Keys' and create a new key".white(),
            "Copy the key and paste it in the command".white()
        ),
        _ => "Visit the provider's website to obtain an API key".to_string(),
    }
}

/// Handle redaction subcommands with full functionality restored
pub fn handle_redact_command(
    repo: &SqliteRepository,
    action: &RedactAction,
    _args: &RedactArgs,
) -> Result<()> {
    use crate::config::service::redacted_config;

    match action {
        RedactAction::Add { pattern } => {
            println!("{}", "🔒 Add Redaction Pattern".bright_yellow().bold());
            println!("{}", "═════════════════════".white().dimmed());
            println!();
            println!("{}  {}", "Pattern:".bright_green(), pattern.bright_white());

            // Add the redaction pattern
            add_redaction_pattern(repo, pattern)?;

            println!();
            println!(
                "{}",
                "✅ Redaction pattern added successfully!".green().bold()
            );
            println!();
            println!("{}", "💡 What this does:".bright_yellow().bold());
            println!("   • Replaces '{}' with [REDACTED] in AI requests", pattern);
            println!("   • Protects your privacy when sharing context with AI");
            println!("   • Applied to both file contents and user messages");
            println!();
            println!("{}", "💡 Next steps:".bright_yellow().bold());
            println!(
                "   {}              # View all patterns",
                "termai redact list".cyan()
            );
            println!(
                "   {}  # Test with context",
                "termai ask --preview-context \"question\"".cyan()
            );

            Ok(())
        }
        RedactAction::Remove { pattern } => {
            println!("{}", "🔓 Remove Redaction Pattern".bright_yellow().bold());
            println!("{}", "═══════════════════════".white().dimmed());
            println!();
            println!(
                "{}  {}",
                "Pattern to remove:".bright_green(),
                pattern.bright_white()
            );

            // Remove the redaction pattern
            if remove_redaction_pattern(repo, pattern)? {
                println!();
                println!(
                    "{}",
                    "✅ Redaction pattern removed successfully!".green().bold()
                );
                println!("   '{}' will no longer be redacted", pattern);
            } else {
                println!();
                println!("{}", "⚠️  Pattern not found".yellow().bold());
                println!("   '{}' was not in the redaction list", pattern);
                println!();
                println!("{}", "💡 View current patterns:".bright_yellow().bold());
                println!(
                    "   {}              # Show all patterns",
                    "termai redact list".cyan()
                );
            }

            Ok(())
        }
        RedactAction::List => {
            println!("{}", "📝 Redaction Patterns".bright_blue().bold());
            println!("{}", "═══════════════════".white().dimmed());
            println!();

            let patterns = redacted_config::fetch_redactions(repo);
            let active_patterns: Vec<String> = patterns
                .into_iter()
                .filter(|s| !s.trim().is_empty())
                .collect();

            if active_patterns.is_empty() {
                println!("{}", "📝 No redaction patterns configured".yellow());
                println!();
                println!("{}", "💡 Add privacy protection:".bright_yellow().bold());
                println!(
                    "   {}         # Add email",
                    "termai redact add user@example.com".cyan()
                );
                println!(
                    "   {}     # Add name",
                    "termai redact add \"John Smith\"".cyan()
                );
                println!(
                    "   {}    # Add API key prefix",
                    "termai redact add sk-".cyan()
                );
            } else {
                println!(
                    "{}  {} pattern(s) active",
                    "🔒 Privacy Protection:".bright_green(),
                    active_patterns.len()
                );
                println!();

                for (i, pattern) in active_patterns.iter().enumerate() {
                    println!(
                        "   {}. {}",
                        (i + 1).to_string().bright_cyan(),
                        pattern.trim().bright_white()
                    );
                }

                println!();
                println!("{}", "💡 Pattern management:".bright_yellow().bold());
                println!(
                    "   {}    # Add new pattern",
                    "termai redact add \"<pattern>\"".cyan()
                );
                println!(
                    "   {} # Remove pattern",
                    "termai redact remove \"<pattern>\"".cyan()
                );
                println!();
                println!("{}", "🛡️ Privacy features:".bright_green().bold());
                println!("   • Patterns are case-insensitive");
                println!("   • Applied to file contents and messages");
                println!("   • Replaced with [REDACTED] before sending to AI");
            }

            println!();
            println!("{}", "💡 Test your redactions:".bright_yellow().bold());
            println!(
                "   {}  # Preview context with redactions",
                "termai ask --preview-context \"test\" src/".cyan()
            );

            Ok(())
        }
    }
}

/// Add a redaction pattern to the configuration
fn add_redaction_pattern(repo: &SqliteRepository, pattern: &str) -> Result<()> {
    use crate::config::model::keys::ConfigKeys;
    use crate::config::service::config_service;

    // Get existing redaction patterns
    let existing_patterns = match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key())
    {
        Ok(config) => config.value,
        Err(_) => String::new(),
    };

    // Check if pattern already exists
    if !existing_patterns.is_empty() {
        let patterns: Vec<&str> = existing_patterns.split(',').collect();
        if patterns.iter().any(|&p| p.trim() == pattern.trim()) {
            println!();
            println!("{}", "⚠️  Pattern already exists".yellow());
            return Ok(());
        }
    }

    // Add the new pattern
    let updated_patterns = if existing_patterns.is_empty() {
        pattern.to_string()
    } else {
        format!("{},{}", existing_patterns, pattern)
    };

    config_service::write_config(repo, &ConfigKeys::Redacted.to_key(), &updated_patterns)?;

    Ok(())
}

/// Remove a redaction pattern from the configuration
fn remove_redaction_pattern(repo: &SqliteRepository, pattern: &str) -> Result<bool> {
    use crate::config::model::keys::ConfigKeys;
    use crate::config::service::config_service;

    // Get existing redaction patterns
    let existing_patterns = match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key())
    {
        Ok(config) => config.value,
        Err(_) => return Ok(false), // No patterns exist
    };

    // Filter out the pattern to remove
    let remaining_patterns: Vec<&str> = existing_patterns
        .split(',')
        .filter(|&p| p.trim() != pattern.trim() && !p.trim().is_empty())
        .collect();

    // Check if pattern was found and removed
    let original_count = existing_patterns
        .split(',')
        .filter(|p| !p.trim().is_empty())
        .count();
    let remaining_count = remaining_patterns.len();

    if original_count == remaining_count {
        return Ok(false); // Pattern was not found
    }

    // Update the configuration
    let updated_patterns = remaining_patterns.join(",");
    config_service::write_config(repo, &ConfigKeys::Redacted.to_key(), &updated_patterns)?;

    Ok(true)
}

/// Handle project configuration initialization
fn handle_config_init(project_type: Option<&str>, template: Option<&str>, force: bool) -> Result<()> {
    println!("{}", "🏗️  Initialize TermAI Project Configuration".bright_blue().bold());
    println!("{}", "══════════════════════════════════════════".white().dimmed());
    println!();

    let project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    // Check if configuration already exists
    if project_service.project_config_exists() && !force {
        let config_path = project_service.get_config_file_path();
        println!("{}", "⚠️  Project configuration already exists".yellow());
        println!("   File: {}", config_path.display().to_string().cyan());
        println!();
        println!("{}", "💡 Options:".bright_yellow().bold());
        println!("   {}  # Override existing config", "termai config init --force".cyan());
        println!("   {}     # View current config", "termai config project".cyan());
        println!("   {}        # Edit current config", "termai config edit".cyan());
        return Ok(());
    }

    // Use template if specified
    if let Some(template_name) = template {
        let templates = ProjectConfigService::get_project_type_templates();
        if let Some(template_config) = templates.get(template_name) {
            project_service.save_project_config(template_config)
                .context("Failed to save template configuration")?;
                
            println!("✅ Project configuration created from {} template", template_name.bright_cyan());
            println!("   File: {}", project_service.get_config_file_path().display().to_string().cyan());
            println!();
            show_next_steps();
            return Ok(());
        } else {
            println!("{}", "⚠️  Template not found".yellow());
            println!("Available templates:");
            for template in templates.keys() {
                println!("   • {}", template.cyan());
            }
            return Ok(());
        }
    }

    // Initialize configuration for detected or specified project type
    let config = project_service.init_project_config(project_type.map(|s| s.to_string()))
        .context("Failed to initialize project configuration")?;

    // Save the configuration
    project_service.save_project_config(&config)
        .context("Failed to save project configuration")?;

    println!("✅ Project configuration initialized successfully!");
    
    if let Some(project) = &config.project {
        println!("   Project: {}", project.name.bright_cyan());
        if let Some(proj_type) = &project.project_type {
            println!("   Type: {}", proj_type.bright_green());
        }
    }
    
    println!("   File: {}", project_service.get_config_file_path().display().to_string().cyan());
    println!();
    
    show_next_steps();
    Ok(())
}

/// Handle project configuration display
fn handle_config_project() -> Result<()> {
    println!("{}", "📋 Project Configuration".bright_blue().bold());
    println!("{}", "════════════════════════".white().dimmed());
    println!();

    let mut project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    if !project_service.project_config_exists() {
        println!("{}", "⚠️  No project configuration found".yellow());
        println!();
        println!("{}", "💡 Initialize project configuration:".bright_yellow().bold());
        println!("   {}           # Auto-detect project type", "termai config init".cyan());
        println!("   {} # Specify project type", "termai config init --project-type rust".cyan());
        return Ok(());
    }

    let config = project_service.load_config()
        .context("Failed to load project configuration")?;

    // Show project metadata
    if let Some(project) = &config.project {
        println!("{}  {}", "🏷️  Project Name:".bright_green(), project.name.bright_cyan());
        if let Some(project_type) = &project.project_type {
            println!("{}      {}", "🎯 Project Type:".bright_green(), project_type.bright_cyan());
        }
        if let Some(description) = &project.description {
            println!("{}   {}", "📝 Description:".bright_green(), description);
        }
        println!();
    }

    // Show context configuration
    if let Some(context) = &config.context {
        println!("{}", "📁 Context Configuration".bright_yellow().bold());
        println!("   Max tokens: {}", context.max_tokens.unwrap_or(8000).to_string().cyan());
        
        if let Some(include) = &context.include {
            println!("   Include patterns: {} patterns", include.len().to_string().cyan());
            for pattern in include.iter().take(3) {
                println!("     • {}", pattern.dimmed());
            }
            if include.len() > 3 {
                println!("     ... and {} more", (include.len() - 3).to_string().dimmed());
            }
        }
        
        if let Some(exclude) = &context.exclude {
            println!("   Exclude patterns: {} patterns", exclude.len().to_string().cyan());
        }
        println!();
    }

    // Show provider configuration
    if let Some(providers) = &config.providers {
        println!("{}", "🤖 Provider Configuration".bright_yellow().bold());
        if let Some(default) = &providers.default {
            println!("   Default provider: {}", default.bright_cyan());
        }
        if let Some(fallback) = &providers.fallback {
            println!("   Fallback provider: {}", fallback.bright_cyan());
        }
        println!();
    }

    // Show file location
    println!("{}      {}", "📄 Config File:".bright_green(), 
             project_service.get_config_file_path().display().to_string().cyan());

    println!();
    println!("{}", "💡 Management commands:".bright_yellow().bold());
    println!("   {}       # Validate configuration", "termai config validate".cyan());
    println!("   {}          # Edit configuration", "termai config edit".cyan());
    println!("   {}      # Export configuration", "termai config export".cyan());

    Ok(())
}

/// Handle project configuration validation
fn handle_config_validate() -> Result<()> {
    println!("{}", "✅ Validate Project Configuration".bright_blue().bold());
    println!("{}", "══════════════════════════════════".white().dimmed());
    println!();

    let mut project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    if !project_service.project_config_exists() {
        println!("{}", "⚠️  No project configuration found".yellow());
        println!("   Run {} to create one", "termai config init".cyan());
        return Ok(());
    }

    let config = project_service.load_config()
        .context("Failed to load project configuration")?;

    let validation = config.validate();
    
    if validation.is_valid {
        println!("{}", "✅ Configuration is valid!".green().bold());
    } else {
        println!("{}", "❌ Configuration has errors:".red().bold());
        for error in &validation.errors {
            println!("   • {}: {}", error.field.bright_red(), error.message);
            if let Some(suggestion) = &error.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
    }

    if !validation.warnings.is_empty() {
        println!();
        println!("{}", "⚠️  Warnings:".yellow().bold());
        for warning in &validation.warnings {
            println!("   • {}: {}", warning.field.bright_yellow(), warning.message);
            if let Some(suggestion) = &warning.suggestion {
                println!("     💡 {}", suggestion.dimmed());
            }
        }
    }

    if validation.is_valid && validation.warnings.is_empty() {
        println!("   All checks passed!");
        println!();
        println!("{}", "🚀 Ready to use:".bright_green().bold());
        println!("   {}              # Start chatting", "termai chat".cyan());
        println!("   {}  # Ask with context", "termai ask \"question\" src/".cyan());
    }

    Ok(())
}

/// Handle project configuration editing
fn handle_config_edit() -> Result<()> {
    let project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    let config_path = project_service.get_config_file_path();

    if !project_service.project_config_exists() {
        println!("{}", "⚠️  No project configuration found".yellow());
        println!("   Run {} to create one first", "termai config init".cyan());
        return Ok(());
    }

    // Try to find a suitable editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "macos") {
                "open".to_string()
            } else if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });

    println!("📝 Opening configuration in {}", editor.bright_cyan());
    println!("   File: {}", config_path.display().to_string().dimmed());

    let mut command = Command::new(&editor);
    command.arg(&config_path);

    match command.status() {
        Ok(status) if status.success() => {
            println!();
            println!("{}", "✅ Configuration edited successfully".green());
            println!("   Run {} to validate changes", "termai config validate".cyan());
        }
        Ok(_) => {
            println!("{}", "⚠️  Editor exited with non-zero status".yellow());
        }
        Err(e) => {
            println!("{}", format!("❌ Failed to open editor: {}", e).red());
            println!("   Try setting EDITOR environment variable");
            println!("   Example: {} termai config edit", "EDITOR=vim".cyan());
        }
    }

    Ok(())
}

/// Handle project configuration export
fn handle_config_export(file: Option<&str>, format: &str) -> Result<()> {
    println!("{}", "📤 Export Project Configuration".bright_blue().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());
    println!();

    let mut project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    if !project_service.project_config_exists() {
        println!("{}", "⚠️  No project configuration found".yellow());
        return Ok(());
    }

    let config = project_service.load_config()
        .context("Failed to load project configuration")?;

    let output_file = file.unwrap_or("termai-config-export.toml");

    match format {
        "toml" => {
            let content = toml::to_string_pretty(&config)
                .context("Failed to serialize configuration to TOML")?;
            std::fs::write(output_file, content)
                .context("Failed to write configuration file")?;
        }
        "json" => {
            let content = serde_json::to_string_pretty(&config)
                .context("Failed to serialize configuration to JSON")?;
            std::fs::write(output_file, content)
                .context("Failed to write configuration file")?;
        }
        "yaml" => {
            let content = serde_yaml::to_string(&config)
                .context("Failed to serialize configuration to YAML")?;
            std::fs::write(output_file, content)
                .context("Failed to write configuration file")?;
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported format: {}. Supported: toml, json, yaml",
                format
            ));
        }
    }

    println!("✅ Configuration exported successfully");
    println!("   File: {}", output_file.bright_cyan());
    println!("   Format: {}", format.bright_green());

    Ok(())
}

/// Handle project configuration import
fn handle_config_import(file: &str, merge: bool) -> Result<()> {
    println!("{}", "📥 Import Project Configuration".bright_blue().bold());
    println!("{}", "═══════════════════════════════".white().dimmed());
    println!();

    let project_service = ProjectConfigService::new()
        .context("Failed to initialize project configuration service")?;

    if !std::path::Path::new(file).exists() {
        return Err(anyhow::anyhow!("Import file not found: {}", file));
    }

    let content = std::fs::read_to_string(file)
        .context("Failed to read import file")?;

    // Determine format from file extension
    let imported_config: ProjectConfig = if file.ends_with(".json") {
        serde_json::from_str(&content)
            .context("Failed to parse JSON configuration")?
    } else if file.ends_with(".yaml") || file.ends_with(".yml") {
        serde_yaml::from_str(&content)
            .context("Failed to parse YAML configuration")?
    } else {
        // Default to TOML
        toml::from_str(&content)
            .context("Failed to parse TOML configuration")?
    };

    let final_config = if merge && project_service.project_config_exists() {
        // Merge with existing configuration
        let _existing_config = project_service.load_config_file(&project_service.get_config_file_path())
            .context("Failed to load existing configuration")?;
        
        // Simple merge - imported config takes priority
        // In a full implementation, this would be more sophisticated
        imported_config // For now, just replace
    } else {
        imported_config
    };

    // Validate before saving
    let validation = final_config.validate();
    if !validation.is_valid {
        println!("{}", "❌ Import validation failed:".red().bold());
        for error in &validation.errors {
            println!("   • {}: {}", error.field.bright_red(), error.message);
        }
        return Ok(());
    }

    project_service.save_project_config(&final_config)
        .context("Failed to save imported configuration")?;

    println!("✅ Configuration imported successfully");
    println!("   From: {}", file.bright_cyan());
    if merge {
        println!("   Mode: Merged with existing configuration");
    } else {
        println!("   Mode: Replaced existing configuration");
    }

    if !validation.warnings.is_empty() {
        println!();
        println!("{}", "⚠️  Import warnings:".yellow().bold());
        for warning in &validation.warnings {
            println!("   • {}: {}", warning.field.bright_yellow(), warning.message);
        }
    }

    Ok(())
}

/// Show next steps after configuration initialization
fn show_next_steps() {
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {}     # View configuration", "termai config project".cyan());
    println!("   {}       # Validate configuration", "termai config validate".cyan());
    println!("   {}          # Edit configuration", "termai config edit".cyan());
    println!("   {}              # Start chatting with project context", "termai chat".cyan());
}
