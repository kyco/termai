/// Handlers for Config and Redact commands - configuration management
use crate::args::{ConfigAction, ConfigArgs, Provider, RedactAction, RedactArgs};
use crate::config::settings::{
    migrate_legacy_db_config, ProjectConfig, ProjectContextSettings, ProjectPrivacySettings,
    ResolvedSettings, SettingsOverrides, SettingsProvider, UserConfig,
};
use crate::config::service::config_service;
use crate::llm::openai::model::models_api::{infer_provider_from_model_id, ModelObject};
use crate::repository::db::SqliteRepository;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Handle config subcommands with enhanced feedback and error handling
pub async fn handle_config_command(
    repo: &SqliteRepository,
    action: &ConfigAction,
    _args: &ConfigArgs,
) -> Result<()> {
    match action {
        ConfigAction::Show => handle_config_show(repo),
        ConfigAction::SetOpenai { api_key } => handle_set_api_key(
            repo,
            "OpenAI",
            &crate::config::model::keys::ConfigKeys::ChatGptApiKey.to_key(),
            api_key,
            "openai",
        ),
        ConfigAction::SetClaude { api_key } => handle_set_api_key(
            repo,
            "Claude",
            &crate::config::model::keys::ConfigKeys::ClaudeApiKey.to_key(),
            api_key,
            "claude",
        ),
        ConfigAction::SetProvider { provider } => handle_set_provider(*provider),
        ConfigAction::Reset => handle_removed_config_command(
            "reset",
            "Delete ~/.config/termai/config.toml for defaults, then use 'termai auth logout <provider>' if you also want to clear credentials.",
        ),
        ConfigAction::Env => handle_supported_env_command(),
        
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
        ConfigAction::Migrate => handle_config_migrate(repo),
        ConfigAction::Export { file, format } => {
            handle_config_export(file.as_deref(), format)
        }
        ConfigAction::Import { file, merge } => {
            handle_config_import(file, *merge)
        }
        ConfigAction::LoginCodex => {
            crate::commands::codex_auth::handle_login_codex(repo).await
        }
        ConfigAction::LogoutCodex => {
            crate::commands::codex_auth::handle_logout_codex(repo)
        }
        ConfigAction::CodexStatus => {
            crate::commands::codex_auth::handle_codex_status(repo)
        }
        ConfigAction::SetModel { model } => {
            handle_set_model(repo, model.as_deref()).await
        }
        ConfigAction::ListModels { provider, refresh } => {
            handle_list_models(repo, provider.as_ref(), *refresh).await
        }
    }
}

fn handle_config_show(repo: &SqliteRepository) -> Result<()> {
    use crate::config::model::keys::ConfigKeys;

    let settings = ResolvedSettings::load_for_current_dir_with_repo(repo, SettingsOverrides::default())
        .context("Failed to resolve effective settings")?;

    println!("{}", "📋 Effective Configuration".bright_blue().bold());
    println!("{}", "═════════════════════════".white().dimmed());
    println!();
    println!(
        "{} {}",
        "Default provider:".bright_green(),
        settings.default_provider.as_str().bright_cyan()
    );
    println!(
        "{} {}",
        "Default model:".bright_green(),
        settings.selected_model().bright_cyan()
    );
    println!(
        "{} {}",
        "Smart context:".bright_green(),
        if settings.smart_context {
            "enabled".green()
        } else {
            "disabled".yellow()
        }
    );
    println!(
        "{} {}",
        "Token budget:".bright_green(),
        settings.token_budget.to_string().bright_white()
    );
    println!(
        "{} {}",
        "Streaming:".bright_green(),
        if settings.streaming {
            "enabled".green()
        } else {
            "disabled".yellow()
        }
    );
    println!(
        "{} {}",
        "Theme:".bright_green(),
        settings.theme.bright_white()
    );
    println!();
    println!("{}", "📁 Files".bright_yellow().bold());
    println!(
        "   {} {}",
        "User config:".bright_green(),
        settings.user_config_path.display().to_string().cyan()
    );
    match &settings.project_config_path {
        Some(path) => println!(
            "   {} {}",
            "Project config:".bright_green(),
            path.display().to_string().cyan()
        ),
        None => println!(
            "   {} {}",
            "Project config:".bright_green(),
            "none in current project".dimmed()
        ),
    }
    if let Some(project_type) = &settings.project.project_type {
        println!(
            "   {} {}",
            "Project type:".bright_green(),
            project_type.bright_white()
        );
    }
    if !settings.project.context.include.is_empty() {
        println!(
            "   {} {}",
            "Context include:".bright_green(),
            settings.project.context.include.join(", ").bright_white()
        );
    }
    if !settings.project.context.exclude.is_empty() {
        println!(
            "   {} {}",
            "Context exclude:".bright_green(),
            settings.project.context.exclude.join(", ").bright_white()
        );
    }
    if !settings.project.privacy.redact.is_empty() {
        println!(
            "   {} {}",
            "Project redact:".bright_green(),
            settings.project.privacy.redact.join(", ").bright_white()
        );
    }
    println!();
    println!("{}", "🔐 Auth".bright_yellow().bold());
    println!(
        "   {} {}",
        "Claude:".bright_green(),
        auth_status(repo, &ConfigKeys::ClaudeApiKey.to_key(), false)
    );
    println!(
        "   {} {}",
        "OpenAI:".bright_green(),
        auth_status(repo, &ConfigKeys::ChatGptApiKey.to_key(), false)
    );
    println!(
        "   {} {}",
        "Codex:".bright_green(),
        auth_status(repo, &ConfigKeys::CodexAccessToken.to_key(), true)
    );
    println!();
    println!("{}", "💡 Primary commands".bright_yellow().bold());
    println!("   {}", "termai config edit      # Edit user or active project config".cyan());
    println!("   {}", "termai config validate  # Validate the active project config".cyan());
    println!("   {}", "termai config migrate   # Import legacy DB defaults into config.toml".cyan());
    println!("   {}", "termai auth status codex".cyan());
    println!("   {}", "termai setup".cyan());

    Ok(())
}

fn handle_set_api_key(
    repo: &SqliteRepository,
    provider_name: &str,
    key: &str,
    api_key: &str,
    suggested_provider: &str,
) -> Result<()> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "API key cannot be empty\n\n{}",
            get_api_key_help(provider_name)
        ));
    }

    config_service::write_config(repo, key, api_key)
        .with_context(|| format!("Failed to save {} API key", provider_name))?;

    println!(
        "{}",
        format!("✅ {} API credential configured successfully", provider_name)
            .green()
            .bold()
    );
    println!();
    println!("{}", "💡 Next steps".bright_yellow().bold());
    println!("   {}", "termai config show".cyan());
    println!("   {}", format!("termai config set-provider {}", suggested_provider).cyan());
    println!("   {}", format!("termai auth status {}", suggested_provider).cyan());

    Ok(())
}

fn handle_set_provider(provider: Provider) -> Result<()> {
    let mut user_config = UserConfig::load()?;
    user_config.default.provider = settings_provider_from_cli(provider);

    if user_config
        .default
        .model
        .as_deref()
        .is_some_and(|model| infer_provider_from_model(model) != user_config.default.provider.as_str())
    {
        user_config.default.model = None;
    }

    user_config.save()?;

    println!(
        "{} {}",
        "✅ Default provider set to".green().bold(),
        user_config.default.provider.as_str().bright_cyan().bold()
    );
    println!("   {}", "Saved in ~/.config/termai/config.toml".dimmed());
    Ok(())
}

fn handle_removed_config_command(command: &str, guidance: &str) -> Result<()> {
    println!(
        "{} {}",
        "⚠️  Deprecated config command:".yellow().bold(),
        command.bright_white()
    );
    println!();
    println!("{}", guidance.white());
    println!();
    println!("{}", "Use 'termai config show' to inspect the current simplified layout.".cyan());
    Ok(())
}

fn handle_supported_env_command() -> Result<()> {
    let supported = [
        ("OPENAI_API_KEY", std::env::var("OPENAI_API_KEY").ok()),
        ("CLAUDE_API_KEY", std::env::var("CLAUDE_API_KEY").ok()),
        ("ANTHROPIC_API_KEY", std::env::var("ANTHROPIC_API_KEY").ok()),
    ];

    println!("{}", "🌍 Supported Environment Variables".bright_green().bold());
    println!("{}", "══════════════════════════════════".white().dimmed());
    println!();
    println!("{}", "Only auth-related environment variables remain supported in the simplified model.".white());
    println!("{}", "Behavior defaults now live in TOML config files, not TERMAI_* overrides.".white());
    println!();

    for (name, value) in supported {
        let display_value = value
            .map(|value| redact_env_value(&value))
            .unwrap_or_else(|| "not set".to_string());
        println!("   {}  {}", name.bright_white(), display_value.cyan());
    }

    Ok(())
}

fn handle_config_migrate(repo: &SqliteRepository) -> Result<()> {
    let user_config_path = UserConfig::default_path()?;
    let migrated = migrate_legacy_db_config(repo, &user_config_path)?;

    println!("{}", "🔄 Legacy Config Migration".bright_blue().bold());
    println!("{}", "══════════════════════════".white().dimmed());
    println!();

    if migrated {
        println!(
            "{} {}",
            "✅ Wrote simplified user config to".green().bold(),
            user_config_path.display().to_string().cyan()
        );
    } else if user_config_path.exists() {
        println!("{}", "User config already exists. No migration was needed.".yellow());
    } else {
        println!("{}", "No legacy DB defaults were found to migrate.".yellow());
    }

    if let Some(project_config_path) = active_project_config_path() {
        let dropped = dropped_project_sections(&project_config_path)?;
        if !dropped.is_empty() {
            println!();
            println!("{}", "⚠️  Dropped project sections detected".yellow().bold());
            for section in dropped {
                println!("   • {}", section.bright_yellow());
            }
        }
    }

    Ok(())
}

fn auth_status(repo: &SqliteRepository, key: &str, is_oauth: bool) -> ColoredString {
    if config_service::has_config(repo, key) {
        if is_oauth {
            "authenticated".green()
        } else {
            "configured".green()
        }
    } else {
        "not configured".red()
    }
}

fn settings_provider_from_cli(provider: Provider) -> SettingsProvider {
    match provider {
        Provider::Claude => SettingsProvider::Claude,
        Provider::Openai => SettingsProvider::Openai,
        Provider::OpenaiCodex => SettingsProvider::Codex,
    }
}

fn redact_env_value(value: &str) -> String {
    if value.len() > 8 {
        format!("{}...{}", &value[..4], &value[value.len() - 4..])
    } else if value.is_empty() {
        "[empty]".to_string()
    } else {
        "[set]".to_string()
    }
}

fn active_project_config_path() -> Option<PathBuf> {
    ResolvedSettings::load_for_current_dir(SettingsOverrides::default())
        .ok()
        .and_then(|settings| settings.project_config_path)
        .filter(|path| path.exists())
}

fn dropped_project_sections(path: &Path) -> Result<Vec<String>> {
    const DROPPED: &[&str] = &["providers", "git", "output", "templates", "team", "quality", "env"];

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read project config: {}", path.display()))?;
    let parsed: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse project config: {}", path.display()))?;

    let Some(table) = parsed.as_table() else {
        return Ok(Vec::new());
    };

    Ok(DROPPED
        .iter()
        .filter(|section| table.contains_key(**section))
        .map(|section| section.to_string())
        .collect())
}

fn detect_project_type() -> Option<String> {
    let current_dir = std::env::current_dir().ok()?;
    let candidates = [
        ("Cargo.toml", "rust"),
        ("package.json", "javascript"),
        ("pyproject.toml", "python"),
        ("go.mod", "go"),
    ];

    candidates
        .iter()
        .find(|(marker, _)| current_dir.join(marker).exists())
        .map(|(_, project_type)| project_type.to_string())
}

fn default_include_patterns(project_type: Option<&str>) -> Vec<String> {
    match project_type {
        Some("rust") => vec!["src/**/*.rs".to_string(), "tests/**/*.rs".to_string()],
        Some("javascript") => vec![
            "src/**/*.{js,ts,tsx}".to_string(),
            "test/**/*.{js,ts,tsx}".to_string(),
        ],
        Some("python") => vec!["**/*.py".to_string(), "tests/**/*.py".to_string()],
        Some("go") => vec!["**/*.go".to_string()],
        _ => vec!["src/**".to_string()],
    }
}

fn default_exclude_patterns(project_type: Option<&str>) -> Vec<String> {
    let mut patterns = vec![".git/**".to_string()];
    match project_type {
        Some("rust") => patterns.push("target/**".to_string()),
        Some("javascript") => patterns.push("node_modules/**".to_string()),
        Some("python") => patterns.push("__pycache__/**".to_string()),
        _ => {}
    }
    patterns
}

fn default_entry_points(project_type: Option<&str>) -> Vec<String> {
    match project_type {
        Some("rust") => vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
        Some("javascript") => vec!["src/index.ts".to_string(), "src/index.js".to_string()],
        Some("python") => vec!["main.py".to_string()],
        Some("go") => vec!["main.go".to_string()],
        _ => Vec::new(),
    }
}

fn open_in_editor(path: &Path) -> Result<()> {
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

    let status = Command::new(&editor).arg(path).status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(anyhow::anyhow!("Editor exited with a non-zero status")),
        Err(err) => Err(anyhow::anyhow!("Failed to launch editor '{}': {}", editor, err)),
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
    let root = std::env::current_dir().context("Failed to determine current working directory")?;
    let config_path = ProjectConfig::file_path(&root);

    if config_path.exists() && !force {
        println!("{}", "⚠️  Project configuration already exists".yellow());
        println!("   {}", config_path.display().to_string().cyan());
        println!("   {}", "Use 'termai config init --force' to overwrite it.".cyan());
        return Ok(());
    }

    if let Some(template_name) = template {
        println!(
            "{} {}",
            "⚠️  Project templates were removed in the simplified config model. Ignoring template:".yellow(),
            template_name.bright_white()
        );
        println!();
    }

    let resolved_project_type = project_type
        .map(str::to_string)
        .or_else(detect_project_type);
    let project_type_str = resolved_project_type.clone();

    let config = ProjectConfig {
        project_type: resolved_project_type,
        context: ProjectContextSettings {
            include: default_include_patterns(project_type_str.as_deref()),
            exclude: default_exclude_patterns(project_type_str.as_deref()),
            entry_points: default_entry_points(project_type_str.as_deref()),
        },
        privacy: ProjectPrivacySettings {
            redact: vec!["API_KEY_.*".to_string(), "SECRET_.*".to_string()],
        },
        path: Some(config_path.clone()),
    };

    config.save_to_root(&root)?;

    println!("{}", "✅ Project configuration initialized".green().bold());
    println!("   {}", config_path.display().to_string().cyan());
    if let Some(project_type) = &config.project_type {
        println!("   {} {}", "Detected type:".bright_green(), project_type.bright_white());
    }
    println!();
    show_next_steps();
    Ok(())
}

/// Handle project configuration display
fn handle_config_project() -> Result<()> {
    let Some(project_config_path) = active_project_config_path() else {
        println!("{}", "⚠️  No active .termai.toml found".yellow());
        println!("   {}", "Run 'termai config init' in the project root to create one.".cyan());
        return Ok(());
    };

    let root = project_config_path
        .parent()
        .context("Invalid project configuration path")?;
    let config = ProjectConfig::load_from_root(root)?;

    println!("{}", "📋 Project Configuration".bright_blue().bold());
    println!("{}", "════════════════════════".white().dimmed());
    println!();
    println!("{} {}", "File:".bright_green(), project_config_path.display().to_string().cyan());
    if let Some(project_type) = &config.project_type {
        println!("{} {}", "Project type:".bright_green(), project_type.bright_white());
    }
    println!(
        "{} {}",
        "Include:".bright_green(),
        if config.context.include.is_empty() {
            "[none]".dimmed().to_string()
        } else {
            config.context.include.join(", ")
        }
    );
    println!(
        "{} {}",
        "Exclude:".bright_green(),
        if config.context.exclude.is_empty() {
            "[none]".dimmed().to_string()
        } else {
            config.context.exclude.join(", ")
        }
    );
    println!(
        "{} {}",
        "Entry points:".bright_green(),
        if config.context.entry_points.is_empty() {
            "[none]".dimmed().to_string()
        } else {
            config.context.entry_points.join(", ")
        }
    );
    println!(
        "{} {}",
        "Redact:".bright_green(),
        if config.privacy.redact.is_empty() {
            "[none]".dimmed().to_string()
        } else {
            config.privacy.redact.join(", ")
        }
    );

    Ok(())
}

/// Handle project configuration validation
fn handle_config_validate() -> Result<()> {
    let Some(project_config_path) = active_project_config_path() else {
        println!("{}", "⚠️  No active .termai.toml found".yellow());
        println!("   {}", "Run 'termai config init' in the project root to create one.".cyan());
        return Ok(());
    };

    let root = project_config_path
        .parent()
        .context("Invalid project configuration path")?;
    let config = ProjectConfig::load_from_root(root)
        .with_context(|| format!("Failed to load {}", project_config_path.display()))?;

    println!("{}", "✅ Validate Project Configuration".bright_blue().bold());
    println!("{}", "══════════════════════════════════".white().dimmed());
    println!();
    println!("{} {}", "File:".bright_green(), project_config_path.display().to_string().cyan());

    let dropped = dropped_project_sections(&project_config_path)?;
    let mut has_warnings = false;
    if !dropped.is_empty() {
        has_warnings = true;
        println!();
        println!("{}", "⚠️  Dropped sections".yellow().bold());
        for section in dropped {
            println!("   • {}", section.bright_yellow());
        }
    }

    if config.context.include.is_empty() {
        has_warnings = true;
        println!();
        println!("{}", "⚠️  No include patterns configured".yellow());
    }

    println!();
    println!("{}", "✅ Simplified project config parsed successfully.".green().bold());
    if !has_warnings {
        println!("{}", "No structural issues detected.".green());
    }

    Ok(())
}

/// Handle project configuration editing
fn handle_config_edit() -> Result<()> {
    let target_path = if let Some(project_config_path) = active_project_config_path() {
        project_config_path
    } else {
        let user_config = UserConfig::load()?;
        user_config.save()?;
        UserConfig::default_path()?
    };

    println!("{} {}", "📝 Opening".bright_blue().bold(), target_path.display().to_string().cyan());
    open_in_editor(&target_path)?;
    println!("   {}", "Run 'termai config validate' if you edited a project config.".dimmed());
    Ok(())
}

/// Handle project configuration export
fn handle_config_export(file: Option<&str>, format: &str) -> Result<()> {
    let _ = file;
    let _ = format;
    handle_removed_config_command(
        "export",
        "Export/import were removed. The simplified config model uses plain TOML files you can copy directly.",
    )
}

/// Handle project configuration import
fn handle_config_import(file: &str, merge: bool) -> Result<()> {
    let _ = file;
    let _ = merge;
    handle_removed_config_command(
        "import",
        "Export/import were removed. Edit ~/.config/termai/config.toml or .termai.toml directly instead.",
    )
}

/// Show next steps after configuration initialization
fn show_next_steps() {
    println!("{}", "💡 Next steps:".bright_yellow().bold());
    println!("   {}        # View effective settings", "termai config show".cyan());
    println!("   {}       # Validate configuration", "termai config validate".cyan());
    println!("   {}          # Edit configuration", "termai config edit".cyan());
    println!("   {}              # Start chatting with project context", "termai chat".cyan());
}

struct ModelCatalog {
    model_names: Vec<String>,
    live_models: Option<Vec<ModelObject>>,
    warning: Option<String>,
}

fn normalize_provider_name(provider: &str) -> &str {
    match provider {
        "openai-codex" | "openai_codex" => "codex",
        other => other,
    }
}

fn fallback_model_names(provider: &str) -> Vec<String> {
    use crate::chat::state::ChatState;

    let state = ChatState::new(
        normalize_provider_name(provider).to_string(),
        "placeholder".to_string(),
    );
    dedupe_model_names(state.available_models)
}

fn dedupe_model_names(model_names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for model_name in model_names {
        if seen.insert(model_name.clone()) {
            deduped.push(model_name);
        }
    }

    deduped
}

fn sort_live_models(models: &mut [ModelObject]) {
    models.sort_by(|left, right| {
        right
            .created
            .cmp(&left.created)
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn infer_provider_from_model(model: &str) -> &'static str {
    infer_provider_from_model_id(model).unwrap_or("openai")
}

fn current_provider(repo: &SqliteRepository) -> String {
    ResolvedSettings::load_for_current_dir_with_repo(repo, SettingsOverrides::default())
        .map(|settings| settings.default_provider.as_str().to_string())
        .unwrap_or_else(|_| SettingsProvider::default().as_str().to_string())
}

fn current_model_for_provider(repo: &SqliteRepository, provider: &str) -> Option<String> {
    ResolvedSettings::load_for_current_dir_with_repo(repo, SettingsOverrides::default())
        .ok()
        .and_then(|settings| {
            if settings.default_provider.as_str() == normalize_provider_name(provider) {
                Some(settings.selected_model())
            } else {
                None
            }
        })
}

async fn load_model_catalog(
    repo: &SqliteRepository,
    provider: &str,
    refresh: bool,
) -> Result<ModelCatalog> {
    use crate::config::model::keys::ConfigKeys;
    use crate::llm::openai::service::models_service::ModelsService;

    let provider = normalize_provider_name(provider);

    if matches!(provider, "openai" | "codex") {
        let api_key = config_service::fetch_by_key(repo, &ConfigKeys::ChatGptApiKey.to_key())
            .ok()
            .map(|config| config.value)
            .filter(|value| !value.is_empty());

        if let Some(api_key) = api_key {
            let fetched_models = if refresh {
                ModelsService::refresh_models_for_provider(repo, &api_key, provider).await
            } else {
                ModelsService::get_models_for_provider(repo, &api_key, provider).await
            };

            match fetched_models {
                Ok(mut models) if !models.is_empty() => {
                    sort_live_models(&mut models);
                    let model_names = models.iter().map(|model| model.id.clone()).collect();
                    return Ok(ModelCatalog {
                        model_names,
                        live_models: Some(models),
                        warning: None,
                    });
                }
                Ok(_) => {
                    return Ok(ModelCatalog {
                        model_names: fallback_model_names(provider),
                        live_models: None,
                        warning: Some(
                            "OpenAI returned no models for this provider. Using the built-in list."
                                .to_string(),
                        ),
                    });
                }
                Err(err) => {
                    return Ok(ModelCatalog {
                        model_names: fallback_model_names(provider),
                        live_models: None,
                        warning: Some(format!(
                            "Failed to fetch models from OpenAI: {}. Using the built-in list.",
                            err
                        )),
                    });
                }
            }
        }

        return Ok(ModelCatalog {
            model_names: fallback_model_names(provider),
            live_models: None,
            warning: Some(
                "No OpenAI API key configured for live model lookup. Using the built-in list."
                    .to_string(),
            ),
        });
    }

    Ok(ModelCatalog {
        model_names: fallback_model_names(provider),
        live_models: None,
        warning: None,
    })
}

/// Handle setting the default model
async fn handle_set_model(repo: &SqliteRepository, model: Option<&str>) -> Result<()> {
    use crate::chat::state::ChatState;

    let provider = match model {
        Some(model_name) => infer_provider_from_model(model_name).to_string(),
        None => current_provider(repo),
    };

    let catalog = load_model_catalog(repo, &provider, true).await?;

    let selected_model = match model {
        Some(model_name) => {
            if !catalog.model_names.iter().any(|candidate| candidate == model_name) {
                println!("{}", "❌ Invalid model name".red().bold());
                println!();
                println!("{}", "💡 Available models:".bright_yellow().bold());
                println!("   {}  # Open the model selector", "termai config set-model".cyan());
                println!("   {}  # List all models", "termai config list-models".cyan());
                return Ok(());
            }

            model_name.to_string()
        }
        None => {
            if let Some(warning) = &catalog.warning {
                println!("{} {}", "⚠️".yellow(), warning.dimmed());
                println!();
            }

            if catalog.model_names.is_empty() {
                println!("{}", "❌ No models available for the current provider".red().bold());
                return Ok(());
            }

            let default_selection = current_model_for_provider(repo, &provider)
                .and_then(|current_model| {
                    catalog
                        .model_names
                        .iter()
                        .position(|candidate| candidate == &current_model)
                })
                .unwrap_or(0);

            let theme = ColorfulTheme::default();
            let selection = Select::with_theme(&theme)
                .with_prompt("Select the default model")
                .items(&catalog.model_names)
                .default(default_selection)
                .interact()?;

            catalog.model_names[selection].clone()
        }
    };

    let mut user_config = UserConfig::load()?;
    user_config.default.provider = match normalize_provider_name(&provider) {
        "claude" => SettingsProvider::Claude,
        "codex" => SettingsProvider::Codex,
        _ => SettingsProvider::Openai,
    };
    user_config.default.model = Some(selected_model.clone());
    user_config.save()?;

    println!(
        "{} {}",
        "✅ Default model set to".green().bold(),
        selected_model.bright_cyan().bold()
    );

    // Show model description
    let temp_state = ChatState::new(provider.clone(), selected_model.clone());
    let description = temp_state.list_models();
    let model_line = description.lines()
        .find(|line| line.contains(&selected_model))
        .unwrap_or("");
    if !model_line.is_empty() {
        let desc_part = model_line.split(" - ").nth(1).unwrap_or("");
        if !desc_part.is_empty() {
            println!("   {}", desc_part.dimmed());
        }
    }

    println!();
    println!("{}", "💡 Start using:".bright_yellow().bold());
    println!("   {}              # Start chatting with this model", "termai chat".cyan());
    println!("   {}  # Quick question", "termai ask \"question\"".cyan());

    Ok(())
}

/// Handle listing available models
async fn handle_list_models(
    repo: &SqliteRepository,
    provider: Option<&crate::args::Provider>,
    refresh: bool,
) -> Result<()> {
    use crate::args::Provider;

    println!("{}", "🤖 Available Models".bright_blue().bold());
    println!("{}", "═══════════════════".white().dimmed());
    println!();

    // Get models for all providers or a specific one
    let providers_to_show: Vec<&str> = match provider {
        Some(Provider::Claude) => vec!["claude"],
        Some(Provider::Openai) => vec!["openai"],
        Some(Provider::OpenaiCodex) => vec!["codex"],
        None => vec!["claude", "openai", "codex"],
    };

    for provider_name in providers_to_show {
        // Print provider header
        let provider_display = match provider_name {
            "claude" => "🧠 Claude (Anthropic)",
            "openai" => "⚡ OpenAI (API Key)",
            "codex" => "🔐 OpenAI Codex (ChatGPT Plus/Pro)",
            _ => provider_name,
        };
        println!("{}", provider_display.bright_green().bold());

        let catalog = load_model_catalog(repo, provider_name, refresh).await?;

        if let Some(mut live_models) = catalog.live_models {
            sort_live_models(&mut live_models);
            for model in &live_models {
                println!(
                    "   {} {}",
                    format!("• {}", model.id).bright_cyan(),
                    format!("(owned by: {})", model.owned_by).dimmed()
                );
            }
        } else {
            if let Some(warning) = &catalog.warning {
                println!("   {} {}", "⚠️".yellow(), warning.dimmed());
            }

            for model_name in &catalog.model_names {
                println!("   {}", format!("• {}", model_name).bright_cyan());
            }
        }
        println!();
    }

    println!("{}", "💡 Set default model:".bright_yellow().bold());
    println!(
        "   {}  # Open the selector",
        "termai config set-model".cyan()
    );
    println!(
        "   {}  # Set directly by model id",
        "termai config set-model <model>".cyan()
    );
    println!();
    println!("{}", "💡 Switch model in chat:".bright_yellow().bold());
    println!(
        "   {}            # Change model during session",
        "/model <model>".cyan()
    );
    println!();
    println!("{}", "💡 Refresh models:".bright_yellow().bold());
    println!(
        "   {}  # Force refresh from API",
        "termai config list-models --refresh".cyan()
    );

    Ok(())
}
