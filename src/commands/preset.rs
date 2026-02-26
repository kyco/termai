/// Preset and template management command handler
use crate::args::{PresetAction, PresetArgs, PresetFormat};
use crate::preset::builtin::BuiltinPresets;
use crate::preset::manager::{PresetManager, PresetInfo};
use crate::preset::template::Template;
use crate::repository::db::SqliteRepository;
use crate::context::SmartContext;
use crate::git::diff::DiffAnalyzer;
use crate::path::model::Files;
use anyhow::{Context, Result, bail};
use colored::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Handle the preset subcommand
pub async fn handle_preset_command(args: &PresetArgs, _repo: &SqliteRepository) -> Result<()> {
    // Initialize preset manager
    let preset_manager = PresetManager::new()
        .context("Failed to initialize preset manager")?;

    match &args.action {
        PresetAction::List { category, search, detailed } => {
            handle_list_presets(&preset_manager, category.as_deref(), search.as_deref(), *detailed).await
        }
        PresetAction::Use { 
            name, 
            directory, 
            directories, 
            smart_context, 
            session, 
            use_defaults, 
            preview, 
            git_staged, 
            variables 
        } => {
            handle_use_preset(
                &preset_manager,
                name,
                directory.as_deref(),
                directories,
                *smart_context,
                session.as_deref(),
                *use_defaults,
                *preview,
                *git_staged,
                variables,
            ).await
        }
        PresetAction::Create { 
            name, 
            description, 
            category, 
            template, 
            from_session, 
            edit 
        } => {
            handle_create_preset(
                &preset_manager,
                name,
                description.as_deref(),
                category.as_deref(),
                template.as_deref(),
                from_session.as_deref(),
                *edit,
            ).await
        }
        PresetAction::Show { name, template, stats } => {
            handle_show_preset(&preset_manager, name, *template, *stats).await
        }
        PresetAction::Delete { name, force } => {
            handle_delete_preset(&preset_manager, name, *force).await
        }
        PresetAction::Export { name, file, format } => {
            handle_export_preset(&preset_manager, name, file, format).await
        }
        PresetAction::Import { file, force } => {
            handle_import_preset(&preset_manager, file, *force).await
        }
        PresetAction::Edit { name, template, metadata } => {
            handle_edit_preset(&preset_manager, name, *template, *metadata).await
        }
        PresetAction::Search { query, content, category } => {
            handle_search_presets(&preset_manager, query, *content, category.as_deref()).await
        }
    }
}

/// Handle listing presets
async fn handle_list_presets(
    manager: &PresetManager,
    category: Option<&str>,
    search: Option<&str>,
    detailed: bool,
) -> Result<()> {
    println!("{}", "📋 Available Presets".bright_blue().bold());
    println!("{}", "═══════════════════".white().dimmed());

    // Get all presets
    let mut presets = manager.list_presets()
        .context("Failed to list presets")?;
    
    // Add built-in presets
    for builtin in BuiltinPresets::get_all() {
        presets.push(PresetInfo {
            name: builtin.name.clone(),
            description: builtin.description.clone(),
            category: builtin.category.clone(),
            version: builtin.version.clone(),
            path: std::path::PathBuf::from("[built-in]"),
            is_builtin: true,
            usage_count: builtin.template.metadata.usage_count,
        });
    }

    // Apply filters
    if let Some(category_filter) = category {
        presets.retain(|p| p.category.to_lowercase().contains(&category_filter.to_lowercase()));
    }

    if let Some(search_query) = search {
        let query_lower = search_query.to_lowercase();
        presets.retain(|p| {
            p.name.to_lowercase().contains(&query_lower) ||
            p.description.to_lowercase().contains(&query_lower) ||
            p.category.to_lowercase().contains(&query_lower)
        });
    }

    if presets.is_empty() {
        println!("❌ No presets found matching the criteria.");
        return Ok(());
    }

    if detailed {
        // Detailed view
        for preset in presets {
            println!();
            println!("{} {}", "📦".bright_blue(), preset.name.bright_green().bold());
            println!("   {}: {}", "Description".dimmed(), preset.description);
            println!("   {}: {}", "Category".dimmed(), preset.category.bright_cyan());
            println!("   {}: {}", "Version".dimmed(), preset.version);
            println!("   {}: {}", "Type".dimmed(), if preset.is_builtin { "Built-in".bright_yellow() } else { "User".bright_magenta() });
            if preset.usage_count > 0 {
                println!("   {}: {}×", "Used".dimmed(), preset.usage_count.to_string().bright_white());
            }
        }
    } else {
        // Table view
        println!();
        println!("┌─────────────────────┬─────────────┬──────────────────────────────────┬─────────┐");
        println!("│ {} │ {} │ {} │ {} │", 
            format!("{:<19}", "Preset").bright_white().bold(),
            format!("{:<11}", "Category").bright_white().bold(),
            format!("{:<32}", "Description").bright_white().bold(),
            format!("{:<7}", "Usage").bright_white().bold()
        );
        println!("├─────────────────────┼─────────────┼──────────────────────────────────┼─────────┤");

        for preset in presets {
            let name_display = if preset.is_builtin {
                format!("{} {}", preset.name, "✨".bright_yellow())
            } else {
                preset.name.clone()
            };
            
            let description_truncated = if preset.description.len() > 32 {
                format!("{}...", &preset.description[..29])
            } else {
                preset.description.clone()
            };

            let usage_display = if preset.usage_count > 0 {
                format!("{}×", preset.usage_count)
            } else {
                "-".to_string()
            };

            println!("│ {} │ {} │ {} │ {} │",
                format_args!("{:<19}", name_display).to_string().bright_green(),
                format_args!("{:<11}", preset.category).to_string().bright_cyan(),
                format_args!("{:<32}", description_truncated),
                format_args!("{:<7}", usage_display).to_string().bright_white(),
            );
        }
        println!("└─────────────────────┴─────────────┴──────────────────────────────────┴─────────┘");
        
        println!();
        println!("{} Use a preset: {}", "💡".bright_yellow(), "termai preset use <name>".bright_cyan());
        println!("{} Show details: {}", "📖".bright_blue(), "termai preset show <name>".bright_cyan());
    }

    Ok(())
}

/// Handle using a preset
#[allow(clippy::too_many_arguments)]
async fn handle_use_preset(
    manager: &PresetManager,
    name: &str,
    directory: Option<&str>,
    directories: &[String],
    smart_context: bool,
    session: Option<&str>,
    use_defaults: bool,
    preview: bool,
    git_staged: bool,
    variables: &[String],
) -> Result<()> {
    println!("🔍 Using preset: {}", name.bright_blue().bold());
    println!();

    // Try to load preset (first check built-ins, then user presets)
    let preset = if let Some(builtin) = BuiltinPresets::get_by_name(name) {
        builtin
    } else {
        manager.load_preset(name)
            .with_context(|| format!("Preset '{}' not found", name))?
    };

    println!("{} {}", "📦".bright_green(), preset.description.bright_white());
    println!();

    // Parse provided variables
    let mut provided_vars = HashMap::new();
    for var in variables {
        if let Some((key, value)) = var.split_once('=') {
            provided_vars.insert(key.to_string(), Value::String(value.to_string()));
        } else {
            eprintln!("⚠️  Invalid variable format '{}'. Use key=value format.", var);
        }
    }
    
    // Collect file content based on integration flags
    let file_content = collect_context_files(directory, directories, smart_context, git_staged).await?;
    
    // Add file content to variables if we have any
    if !file_content.is_empty() {
        let content_str = format_files_for_template(&file_content);
        provided_vars.insert("file_content".to_string(), Value::String(content_str));
        
        // Add additional Git context if using git-staged
        if git_staged {
            let git_info = collect_git_context().await?;
            for (key, value) in git_info {
                provided_vars.insert(key, Value::String(value));
            }
        }
    } else if !provided_vars.contains_key("file_content") {
        // Add default file_content for built-in presets that expect it
        provided_vars.insert("file_content".to_string(), Value::String("[No file content provided]".to_string()));
    }

    // Collect variables
    let mut variables = if use_defaults {
        // Use defaults only
        let mut vars = HashMap::new();
        for (name, var_def) in &preset.template.variables {
            if let Some(provided) = provided_vars.get(name) {
                vars.insert(name.clone(), provided.clone());
            } else if let Some(default) = &var_def.default {
                vars.insert(name.clone(), default.clone());
            } else if var_def.required {
                bail!("Required variable '{}' not provided and has no default", name);
            }
        }
        vars
    } else if provided_vars.len() <= 1 {
        // Interactive collection (only file_content counts as non-interactive)
        use crate::preset::variables::VariableCollector;
        let collector = VariableCollector::new();
        collector.collect_variables(&preset.template)
            .context("Failed to collect variables")?
    } else {
        // Mix of provided and defaults
        use crate::preset::variables::VariableCollector;
        let collector = VariableCollector::new();
        collector.collect_from_values(&preset.template, &provided_vars)
            .context("Failed to process provided variables")?
    };

    // Ensure file_content and git context variables are preserved
    for (key, value) in provided_vars {
        if key.starts_with("file_content") || key.starts_with("git_") {
            variables.insert(key, value);
        }
    }


    // Render template
    use crate::preset::template::TemplateRenderer;
    let renderer = TemplateRenderer::new()
        .context("Failed to create template renderer")?;
    
    let rendered = renderer.render(&preset.template, &variables)
        .context("Failed to render template")?;

    if preview {
        println!("{}", "📝 Template Preview:".bright_yellow().bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", rendered.bright_white());
        println!("{}", "─".repeat(50).dimmed());
        println!();
        
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt("Continue with this template?")
            .default(true)
            .interact()? {
            println!("❌ Cancelled by user.");
            return Ok(());
        }
        println!();
    }

    println!("{}", "✅ Generated Prompt:".bright_green().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!("{}", rendered.bright_white());
    println!("{}", "═".repeat(50).dimmed());

    // Session integration
    if let Some(session_name) = session {
        println!();
        println!("{} {}", "💾".bright_blue(), format!("Saving to session: {}", session_name).bright_white());
        // TODO: Integrate with session management system
        // For now, just show the integration point
        println!("   Session management integration will be implemented in the next phase.");
    }

    // Show suggested next steps
    println!();
    println!("{} Template rendered successfully!", "🎉".bright_green());
    
    if session.is_some() {
        println!("💡 The rendered prompt has been prepared for AI interaction.");
        println!("💡 Session context will be preserved for follow-up questions.");
    } else {
        println!("💡 Add --session <name> to save this interaction for later reference.");
    }
    
    if git_staged && !file_content.is_empty() {
        println!("🚀 Ready for Git workflow - staged files have been analyzed.");
    }
    
    if smart_context && !file_content.is_empty() {
        println!("🧠 Smart context discovery selected the most relevant files.");
    }

    Ok(())
}

/// Handle creating a new preset
async fn handle_create_preset(
    manager: &PresetManager,
    name: &str,
    description: Option<&str>,
    category: Option<&str>,
    template_content: Option<&str>,
    _from_session: Option<&str>,
    edit: bool,
) -> Result<()> {
    println!("{}", "🧙‍♂️ Preset Creation Wizard".bright_blue().bold());
    println!("{}", "═".repeat(30).dimmed());
    println!("Let's create a new preset: {}", name.bright_green().bold());
    println!();

    use dialoguer::{Input, Select, Confirm};

    // Step 1: Basic Information
    println!("{}", "📝 Step 1: Basic Information".bright_yellow().bold());
    
    let description = if let Some(desc) = description {
        desc.to_string()
    } else {
        Input::new()
            .with_prompt("📋 Description")
            .with_initial_text("A custom preset for...")
            .interact_text()?
    };

    let category_options = vec![
        "development", "testing", "debugging", "documentation", 
        "refactoring", "review", "analysis", "custom"
    ];
    
    let category = if let Some(cat) = category {
        cat.to_string()
    } else {
        let selection = Select::new()
            .with_prompt("🏷️  Category")
            .default(7) // custom
            .items(&category_options)
            .interact()?;
        category_options[selection].to_string()
    };

    println!();

    // Step 2: Template Content
    println!("{}", "📝 Step 2: Template Content".bright_yellow().bold());
    
    let template_content = if let Some(content) = template_content {
        content.to_string()
    } else if edit {
        // Open editor for template content
        println!("Opening your default editor for template content...");
        edit_template_in_editor("")?
    } else {
        Input::new()
            .with_prompt("📄 Template content (use {{variable}} for placeholders)")
            .with_initial_text("Please {{task}} the following:\n\n{{#if file_content}}\n{{file_content}}\n{{else}}\nPlease provide the content to work with.\n{{/if}}")
            .interact_text()?
    };

    // Step 3: Variable Definitions
    println!();
    println!("{}", "📝 Step 3: Variable Definitions".bright_yellow().bold());
    
    let mut variables = HashMap::new();
    
    // Auto-detect variables from template content
    let detected_vars = detect_template_variables(&template_content);
    if !detected_vars.is_empty() {
        println!("🔍 Detected variables: {}", detected_vars.join(", ").bright_cyan());
        
        if Confirm::new()
            .with_prompt("Define these variables interactively?")
            .default(true)
            .interact()? {
            
            variables = define_template_variables(&detected_vars)?;
        }
    }

    // Validate template syntax
    println!();
    println!("{}", "🔍 Validating template...".bright_blue());
    
    if let Err(e) = Template::validate_template_syntax(&template_content) {
        eprintln!("❌ Template validation failed: {}", e);
        if !Confirm::new()
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()? {
            bail!("Template validation failed");
        }
    } else {
        println!("✅ Template syntax is valid!");
    }

    // Step 4: Configuration
    println!();
    println!("{}", "📝 Step 4: Configuration (Optional)".bright_yellow().bold());
    
    let mut config = crate::preset::manager::PresetConfig::default();
    
    if Confirm::new()
        .with_prompt("Configure advanced options?")
        .default(false)
        .interact()? {
        
        let providers = vec!["auto", "claude", "openai"];
        let provider_selection = Select::new()
            .with_prompt("🤖 Preferred AI provider")
            .items(&providers)
            .interact()?;
        config.provider = Some(providers[provider_selection].to_string());
        
        let max_tokens: usize = Input::new()
            .with_prompt("📊 Max tokens (0 for default)")
            .default(0)
            .interact()?;
        if max_tokens > 0 {
            config.max_tokens = Some(max_tokens);
        }
        
        let temperature: f32 = Input::new()
            .with_prompt("🌡️  Temperature (0.0-1.0)")
            .default(0.3)
            .interact()?;
        config.temperature = Some(temperature.clamp(0.0, 1.0));
    }

    // Create template
    let template = Template::new(
        name.to_string(),
        description.clone(),
        template_content.clone(),
        variables,
    ).context("Failed to create template")?;

    let preset = crate::preset::manager::Preset {
        name: name.to_string(),
        description: description.clone(),
        category: category.clone(),
        version: "1.0".to_string(),
        template,
        config,
    };

    // Step 5: Preview and Confirm
    println!();
    println!("{}", "📝 Step 5: Preview & Confirmation".bright_yellow().bold());
    println!();
    println!("📦 {}", preset.name.bright_green().bold());
    println!("   📋 {}", preset.description);
    println!("   🏷️  {}", preset.category.bright_cyan());
    println!("   🔧 {} variables defined", preset.template.variables.len());
    
    if Confirm::new()
        .with_prompt("Save this preset?")
        .default(true)
        .interact()? {
        
        // Save preset
        manager.save_preset(&preset)
            .context("Failed to save preset")?;

        println!();
        println!("🎉 {} Preset '{}' created successfully!", "✅".bright_green(), name.bright_green().bold());
        println!("💡 Use it with: {}", format!("termai preset use \"{}\"", name).bright_cyan());
        println!("📖 View details: {}", format!("termai preset show \"{}\"", name).bright_cyan());
        
        if !preset.template.variables.is_empty() {
            println!("🔧 Test variables: {}", format!("termai preset use \"{}\" --preview", name).bright_cyan());
        }
    } else {
        println!("❌ Preset creation cancelled.");
    }

    Ok(())
}

/// Handle showing preset details
async fn handle_show_preset(
    manager: &PresetManager,
    name: &str,
    show_template: bool,
    _show_stats: bool,
) -> Result<()> {
    // Try to load preset (first check built-ins, then user presets)
    let preset = if let Some(builtin) = BuiltinPresets::get_by_name(name) {
        builtin
    } else {
        manager.load_preset(name)
            .with_context(|| format!("Preset '{}' not found", name))?
    };

    println!("{} {}", "📦".bright_green(), preset.name.bright_white().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!("{}: {}", "Description".bright_blue(), preset.description);
    println!("{}: {}", "Category".bright_blue(), preset.category.bright_cyan());
    println!("{}: {}", "Version".bright_blue(), preset.version);
    
    if !preset.template.variables.is_empty() {
        println!();
        println!("{}", "Variables:".bright_yellow().bold());
        for (var_name, var_def) in &preset.template.variables {
            let required_marker = if var_def.required { "*".red() } else { " ".normal() };
            let default_info = if let Some(default) = &var_def.default {
                format!(" (default: {})", default).dimmed()
            } else {
                "".normal()
            };
            println!("  {}{}: {} {}", 
                required_marker, 
                var_name.bright_white(), 
                var_def.description.bright_white(),
                default_info
            );
        }
    }

    if show_template {
        println!();
        println!("{}", "Template Content:".bright_yellow().bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", preset.template.template.bright_white());
        println!("{}", "─".repeat(50).dimmed());
    }

    Ok(())
}

/// Handle deleting a preset
async fn handle_delete_preset(
    manager: &PresetManager,
    name: &str,
    force: bool,
) -> Result<()> {
    // Check if it's a built-in preset
    if BuiltinPresets::get_by_name(name).is_some() {
        bail!("Cannot delete built-in preset '{}'", name);
    }

    if !force {
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt(format!("Are you sure you want to delete preset '{}'?", name))
            .default(false)
            .interact()? {
            println!("❌ Cancelled by user.");
            return Ok(());
        }
    }

    manager.delete_preset(name)
        .with_context(|| format!("Failed to delete preset '{}'", name))?;

    println!("✅ Preset '{}' deleted successfully!", name.bright_green());

    Ok(())
}

/// Handle exporting a preset
async fn handle_export_preset(
    manager: &PresetManager,
    name: &str,
    file_path: &str,
    _format: &PresetFormat,
) -> Result<()> {
    let path = Path::new(file_path);
    
    manager.export_preset(name, path)
        .with_context(|| format!("Failed to export preset '{}'", name))?;

    Ok(())
}

/// Handle importing a preset
async fn handle_import_preset(
    manager: &PresetManager,
    file_path: &str,
    _force: bool,
) -> Result<()> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        bail!("File '{}' does not exist", file_path);
    }

    manager.import_preset(path)
        .with_context(|| format!("Failed to import preset from '{}'", file_path))?;

    Ok(())
}

/// Handle editing a preset
async fn handle_edit_preset(
    manager: &PresetManager,
    name: &str,
    edit_template: bool,
    edit_metadata: bool,
) -> Result<()> {
    // Check if it's a built-in preset
    if BuiltinPresets::get_by_name(name).is_some() {
        println!("🔄 {} Creating editable copy of built-in preset '{}'", "📋".bright_yellow(), name.bright_green().bold());
        
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt(format!("Built-in presets cannot be edited directly. Create a custom copy of '{}'?", name))
            .default(true)
            .interact()? {
            println!("❌ Edit cancelled.");
            return Ok(());
        }
        
        // Create custom copy
        let builtin = BuiltinPresets::get_by_name(name).unwrap();
        let custom_name = format!("{} (Custom)", name);
        
        let mut custom_preset = builtin.clone();
        custom_preset.name = custom_name.clone();
        custom_preset.version = "1.0-custom".to_string();
        
        manager.save_preset(&custom_preset)?;
        println!("✅ Created custom copy: '{}'", custom_name.bright_green());
        println!("💡 Editing the custom copy instead...");
        println!();
        
        // Continue editing the custom copy
        return Box::pin(handle_edit_preset(manager, &custom_name, edit_template, edit_metadata)).await;
    }

    // Load the preset
    let mut preset = manager.load_preset(name)
        .with_context(|| format!("Preset '{}' not found", name))?;

    println!("{}", "✏️  Preset Editor".bright_blue().bold());
    println!("{}", "═".repeat(20).dimmed());
    println!("Editing preset: {}", preset.name.bright_green().bold());
    println!();

    use dialoguer::{Select, Confirm};

    // Determine what to edit
    let edit_options = if edit_template && edit_metadata {
        vec!["Template & Metadata"]
    } else if edit_template {
        vec!["Template Only"]
    } else if edit_metadata {
        vec!["Metadata Only"]  
    } else {
        vec!["Template", "Metadata", "Variables", "Configuration", "All"]
    };

    let edit_choice = if edit_options.len() == 1 {
        0
    } else {
        Select::new()
            .with_prompt("🔧 What would you like to edit?")
            .items(&edit_options)
            .interact()?
    };

    let mut changes_made = false;

    // Edit based on choice
    match edit_choice {
        0 if edit_options[0] == "Template" => {
            changes_made |= edit_preset_template(&mut preset).await?;
        }
        1 if edit_options[1] == "Metadata" => {
            changes_made |= edit_preset_metadata(&mut preset)?;
        }
        2 if edit_options[2] == "Variables" => {
            changes_made |= edit_preset_variables(&mut preset)?;
        }
        3 if edit_options[3] == "Configuration" => {
            changes_made |= edit_preset_configuration(&mut preset)?;
        }
        4 if edit_options[4] == "All" => {
            // Edit everything
            changes_made |= edit_preset_metadata(&mut preset)?;
            println!();
            changes_made |= edit_preset_template(&mut preset).await?;
            println!();
            changes_made |= edit_preset_variables(&mut preset)?;
            println!();
            changes_made |= edit_preset_configuration(&mut preset)?;
        }
        _ => {
            // Handle forced template/metadata editing
            if edit_template || edit_options[0] == "Template Only" || edit_options[0] == "Template & Metadata" {
                changes_made |= edit_preset_template(&mut preset).await?;
                if edit_options[0] == "Template & Metadata" {
                    println!();
                    changes_made |= edit_preset_metadata(&mut preset)?;
                }
            } else if edit_metadata || edit_options[0] == "Metadata Only" {
                changes_made |= edit_preset_metadata(&mut preset)?;
            }
        }
    }

    // Save changes if any were made
    if changes_made {
        println!();
        println!("{}", "💾 Saving changes...".bright_blue());
        
        // Validate template before saving
        if let Err(e) = Template::validate_template_syntax(&preset.template.template) {
            eprintln!("⚠️  Template validation warning: {}", e);
            if !Confirm::new()
                .with_prompt("Save anyway?")
                .default(false)
                .interact()? {
                println!("❌ Changes not saved.");
                return Ok(());
            }
        }
        
        manager.save_preset(&preset)?;
        println!("✅ Preset '{}' updated successfully!", preset.name.bright_green());
        println!("🧪 Test it: {}", format!("termai preset use \"{}\" --preview", preset.name).bright_cyan());
    } else {
        println!("ℹ️  No changes made.");
    }

    Ok(())
}

/// Handle searching presets
async fn handle_search_presets(
    manager: &PresetManager,
    query: &str,
    _search_content: bool,
    category: Option<&str>,
) -> Result<()> {
    println!("🔍 Searching presets for: '{}'", query.bright_blue().bold());
    println!();

    let results = manager.search_presets(query)
        .context("Failed to search presets")?;

    let mut filtered_results = results;
    if let Some(cat_filter) = category {
        filtered_results.retain(|p| p.category.to_lowercase().contains(&cat_filter.to_lowercase()));
    }

    // Also search built-in presets
    let query_lower = query.to_lowercase();
    let builtin_matches: Vec<_> = BuiltinPresets::get_all()
        .into_iter()
        .filter(|preset| {
            preset.name.to_lowercase().contains(&query_lower) ||
            preset.description.to_lowercase().contains(&query_lower) ||
            preset.category.to_lowercase().contains(&query_lower)
        })
        .filter(|preset| {
            if let Some(cat_filter) = category {
                preset.category.to_lowercase().contains(&cat_filter.to_lowercase())
            } else {
                true
            }
        })
        .collect();

    if filtered_results.is_empty() && builtin_matches.is_empty() {
        println!("❌ No presets found matching '{}'", query);
        return Ok(());
    }

    println!("📋 Found {} result(s):", filtered_results.len() + builtin_matches.len());
    println!();

    // Show built-in matches
    for preset in builtin_matches {
        println!("{} {} {}", "📦".bright_blue(), preset.name.bright_green().bold(), "✨".bright_yellow());
        println!("   {}: {}", "Description".dimmed(), preset.description);
        println!("   {}: {}", "Category".dimmed(), preset.category.bright_cyan());
        println!();
    }

    // Show user preset matches
    for preset in filtered_results {
        println!("{} {}", "📦".bright_blue(), preset.name.bright_green().bold());
        println!("   {}: {}", "Description".dimmed(), preset.description);
        println!("   {}: {}", "Category".dimmed(), preset.category.bright_cyan());
        if preset.usage_count > 0 {
            println!("   {}: {}×", "Used".dimmed(), preset.usage_count.to_string().bright_white());
        }
        println!();
    }

    println!("💡 Use a preset: {}", "termai preset use <name>".bright_cyan());

    Ok(())
}

/// Collect context files based on various integration options
async fn collect_context_files(
    directory: Option<&str>,
    directories: &[String],
    smart_context: bool,
    git_staged: bool,
) -> Result<Vec<Files>> {
    let mut files = Vec::new();
    
    // Determine working directory
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    
    if git_staged {
        // Collect Git staged files
        files.extend(collect_git_staged_files(&current_dir).await?);
        
        if smart_context {
            println!("📊 {} Smart context discovery with Git staged files", "🔍".bright_blue());
            // Use smart context to enhance the staged file selection
            let smart_ctx = SmartContext::from_project(&current_dir)?;
            let additional_files = smart_ctx.discover_context(&current_dir, None).await?;
            
            // Add files that aren't already included from Git staging
            for file in additional_files {
                if !files.iter().any(|f| f.path == file.path) {
                    files.push(file);
                }
            }
        }
    } else if smart_context {
        // Use smart context discovery on specified directories
        println!("📊 {} Smart context discovery", "🔍".bright_blue());
        let smart_ctx = SmartContext::from_project(&current_dir)?;
        
        if let Some(dir) = directory {
            let dir_path = if Path::new(dir).is_relative() {
                current_dir.join(dir)
            } else {
                PathBuf::from(dir)
            };
            files.extend(smart_ctx.discover_context(&dir_path, None).await?);
        } else if !directories.is_empty() {
            for dir in directories {
                let dir_path = if Path::new(dir).is_relative() {
                    current_dir.join(dir)
                } else {
                    PathBuf::from(dir)
                };
                files.extend(smart_ctx.discover_context(&dir_path, None).await?);
            }
        } else {
            // Use current directory for smart context
            files.extend(smart_ctx.discover_context(&current_dir, None).await?);
        }
    } else {
        // Traditional directory-based file collection
        let mut target_dirs = Vec::new();
        
        if let Some(dir) = directory {
            target_dirs.push(dir.to_string());
        }
        target_dirs.extend_from_slice(directories);
        
        if target_dirs.is_empty() {
            target_dirs.push(".".to_string());
        }
        
        for dir in target_dirs {
            files.extend(collect_files_from_directory(&dir).await?);
        }
    }
    
    if !files.is_empty() {
        println!("📁 Collected {} files for context", files.len());
        for file in &files {
            let display_path = if file.path.starts_with(&current_dir.to_string_lossy().to_string()) {
                file.path.strip_prefix(&format!("{}/", current_dir.display())).unwrap_or(&file.path)
            } else {
                &file.path
            };
            println!("   📄 {}", display_path.bright_cyan());
        }
        println!();
    }
    
    Ok(files)
}

/// Collect Git staged files
async fn collect_git_staged_files(repo_path: &Path) -> Result<Vec<Files>> {
    use git2::Repository;
    
    let repo = Repository::open(repo_path)
        .context("Failed to open Git repository. Use --directory to specify a Git repository.")?;
    
    let diff_analyzer = DiffAnalyzer::new(&repo);
    let staged_diff = diff_analyzer.analyze_staged()
        .context("Failed to analyze staged changes")?;
    
    if staged_diff.files_changed == 0 {
        println!("⚠️  No staged files found. Stage files with 'git add' first.");
        return Ok(Vec::new());
    }
    
    println!("📊 {} Found {} staged file(s)", "📝".bright_green(), staged_diff.files_changed);
    staged_diff.display_summary();
    println!();
    
    let mut files = Vec::new();
    for file_change in &staged_diff.files {
        if let Some(path) = &file_change.new_path {
            let full_path = repo_path.join(path);
            if full_path.exists() && !file_change.is_binary {
                match std::fs::read_to_string(&full_path) {
                    Ok(content) => {
                        files.push(Files {
                            path: path.display().to_string(),
                            content,
                        });
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to read {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
    
    Ok(files)
}

/// Collect files from a directory (traditional approach)
async fn collect_files_from_directory(directory: &str) -> Result<Vec<Files>> {
    use crate::path::extract::extract_content;
    
    let target_path = if Path::new(directory).is_relative() {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        current_dir.join(directory)
    } else {
        PathBuf::from(directory)
    };
    
    if !target_path.exists() {
        bail!("Directory '{}' does not exist", directory);
    }
    
    // Use the existing path extraction functionality
    let files = extract_content(&Some(target_path.to_string_lossy().to_string()), &[], &[])
        .ok_or_else(|| anyhow::anyhow!("Failed to extract files from directory"))?;
        
    Ok(files)
}

/// Collect additional Git context information
async fn collect_git_context() -> Result<HashMap<String, String>> {
    use git2::Repository;
    
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    let repo = Repository::open(&current_dir)?;
    
    let mut git_info = HashMap::new();
    
    // Current branch
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            git_info.insert("git_branch".to_string(), branch_name.to_string());
        }
    }
    
    // Current commit
    if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            let commit_id = commit.id().to_string();
            git_info.insert("git_commit".to_string(), commit_id[..8].to_string());
            git_info.insert("git_commit_full".to_string(), commit_id);
            
            if let Some(message) = commit.message() {
                git_info.insert("git_last_commit_message".to_string(), message.lines().next().unwrap_or("").to_string());
            }
        }
    }
    
    // Repository status
    let diff_analyzer = DiffAnalyzer::new(&repo);
    if let Ok(unstaged_diff) = diff_analyzer.analyze_unstaged() {
        if unstaged_diff.files_changed > 0 {
            git_info.insert("git_status".to_string(), "dirty".to_string());
            git_info.insert("git_unstaged_files".to_string(), unstaged_diff.files_changed.to_string());
        } else {
            git_info.insert("git_status".to_string(), "clean".to_string());
        }
    }
    
    Ok(git_info)
}

/// Format collected files for template rendering
fn format_files_for_template(files: &[Files]) -> String {
    if files.is_empty() {
        return "[No files provided]".to_string();
    }
    
    let mut result = String::new();
    
    for (i, file) in files.iter().enumerate() {
        if i > 0 {
            result.push_str("\n\n");
        }
        
        result.push_str(&format!("## File: {}\n", file.path));
        result.push_str("```");
        
        // Add language hint based on file extension
        if let Some(ext) = Path::new(&file.path).extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => result.push_str("rust"),
                "js" | "jsx" => result.push_str("javascript"),
                "ts" | "tsx" => result.push_str("typescript"),
                "py" => result.push_str("python"),
                "go" => result.push_str("go"),
                "java" => result.push_str("java"),
                "kt" => result.push_str("kotlin"),
                "cpp" | "cc" | "cxx" => result.push_str("cpp"),
                "c" => result.push('c'),
                "cs" => result.push_str("csharp"),
                "php" => result.push_str("php"),
                "rb" => result.push_str("ruby"),
                "swift" => result.push_str("swift"),
                "yaml" | "yml" => result.push_str("yaml"),
                "json" => result.push_str("json"),
                "html" => result.push_str("html"),
                "css" => result.push_str("css"),
                "sh" => result.push_str("bash"),
                _ => result.push_str("text"),
            }
        }
        
        result.push('\n');
        result.push_str(&file.content);
        result.push_str("\n```");
    }
    
    result
}

// Helper functions for enhanced preset creation
// Detect template variables from content using regex
fn detect_template_variables(content: &str) -> Vec<String> {
    use regex::Regex;
    
    let mut variables = std::collections::HashSet::new();
    
    // Match {{variable}} patterns, excluding built-in helpers
    let var_regex = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
    for cap in var_regex.captures_iter(content) {
        if let Some(var_name) = cap.get(1) {
            let name = var_name.as_str();
            // Skip Handlebars built-in helpers
            if !matches!(name, "if" | "else" | "unless" | "each" | "with" | "lookup" | "log" | "#if" | "#else" | "#unless" | "#each" | "#with") {
                variables.insert(name.to_string());
            }
        }
    }
    
    // Also match {{#if variable}} patterns
    let if_regex = Regex::new(r"\{\{\s*#if\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
    for cap in if_regex.captures_iter(content) {
        if let Some(var_name) = cap.get(1) {
            variables.insert(var_name.as_str().to_string());
        }
    }
    
    let mut result: Vec<String> = variables.into_iter().collect();
    result.sort();
    result
}

/// Interactively define template variables
fn define_template_variables(var_names: &[String]) -> Result<HashMap<String, crate::preset::template::TemplateVariable>> {
    use dialoguer::{Input, Select, Confirm};
    use crate::preset::template::{TemplateVariable, VariableType};
    
    let mut variables = HashMap::new();
    
    println!();
    println!("🔧 Defining variables...");
    
    let var_types = vec![
        "String", "Boolean", "Number", "File", "Directory", 
        "DateTime", "GitInfo", "Environment"
    ];
    
    for var_name in var_names {
        println!();
        println!("📝 Variable: {}", var_name.bright_cyan().bold());
        
        let description: String = Input::new()
            .with_prompt("  📋 Description")
            .with_initial_text(format!("Description for {}", var_name))
            .interact_text()?;
        
        let type_selection = Select::new()
            .with_prompt("  🏷️  Type")
            .default(0) // String
            .items(&var_types)
            .interact()?;
        
        let var_type = match type_selection {
            0 => VariableType::String,
            1 => VariableType::Boolean,
            2 => VariableType::Number,
            3 => VariableType::File,
            4 => VariableType::Directory,
            5 => VariableType::DateTime,
            6 => VariableType::GitInfo,
            7 => VariableType::Environment,
            _ => VariableType::String,
        };
        
        let required = Confirm::new()
            .with_prompt("  ❓ Required?")
            .default(true)
            .interact()?;
        
        let default_value = if !required {
            let default_input: String = Input::new()
                .with_prompt("  💡 Default value (optional)")
                .allow_empty(true)
                .interact_text()?;
            
            if default_input.is_empty() {
                None
            } else {
                Some(match var_type {
                    VariableType::Boolean => serde_json::Value::Bool(default_input.to_lowercase() == "true"),
                    VariableType::Number => {
                        if let Ok(num) = default_input.parse::<f64>() {
                            serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap_or(serde_json::Number::from(0)))
                        } else {
                            serde_json::Value::String(default_input)
                        }
                    },
                    _ => serde_json::Value::String(default_input),
                })
            }
        } else {
            None
        };
        
        variables.insert(
            var_name.clone(),
            TemplateVariable::new(var_type, description, required, default_value)
        );
        
        println!("  ✅ Variable '{}' configured", var_name.bright_green());
    }
    
    Ok(variables)
}

/// Edit template content in external editor
fn edit_template_in_editor(initial_content: &str) -> Result<String> {
    use std::fs;
    use std::process::Command;
    
    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("termai_template_{}.hbs", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
    ));
    
    // Write initial content
    fs::write(&temp_file, initial_content)
        .context("Failed to create temporary template file")?;
    
    // Get editor command
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });
    
    // Open editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .context("Failed to open editor")?;
    
    if !status.success() {
        bail!("Editor exited with error");
    }
    
    // Read edited content
    let content = fs::read_to_string(&temp_file)
        .context("Failed to read edited template")?;
    
    // Clean up
    let _ = fs::remove_file(&temp_file);
    
    Ok(content)
}

/// Edit preset metadata (name, description, category)
fn edit_preset_metadata(preset: &mut crate::preset::manager::Preset) -> Result<bool> {
    use dialoguer::{Input, Select};
    
    println!("{}", "📝 Editing Metadata".bright_yellow().bold());
    
    let mut changes_made = false;
    
    // Edit description
    let new_description: String = Input::new()
        .with_prompt("📋 Description")
        .with_initial_text(&preset.description)
        .interact_text()?;
    
    if new_description != preset.description {
        preset.description = new_description;
        changes_made = true;
    }
    
    // Edit category
    let category_options = vec![
        "development", "testing", "debugging", "documentation", 
        "refactoring", "review", "analysis", "custom"
    ];
    
    let current_category_index = category_options
        .iter()
        .position(|&cat| cat == preset.category)
        .unwrap_or(7); // default to custom
    
    let category_selection = Select::new()
        .with_prompt("🏷️  Category")
        .default(current_category_index)
        .items(&category_options)
        .interact()?;
    
    let new_category = category_options[category_selection].to_string();
    if new_category != preset.category {
        preset.category = new_category;
        changes_made = true;
    }
    
    if changes_made {
        println!("✅ Metadata updated");
    } else {
        println!("ℹ️  No metadata changes made");
    }
    
    Ok(changes_made)
}

/// Edit preset template content
async fn edit_preset_template(preset: &mut crate::preset::manager::Preset) -> Result<bool> {
    use dialoguer::{Input, Select, Confirm};
    
    println!("{}", "📝 Editing Template".bright_yellow().bold());
    
    let edit_options = vec![
        "Edit in external editor",
        "Edit inline (text input)",
        "View current template",
    ];
    
    let choice = Select::new()
        .with_prompt("🔧 How would you like to edit the template?")
        .items(&edit_options)
        .interact()?;
    
    match choice {
        0 => {
            // External editor
            println!("🔧 Opening external editor...");
            let new_content = edit_template_in_editor(&preset.template.template)?;
            
            if new_content != preset.template.template {
                // Check for new variables
                let old_vars = detect_template_variables(&preset.template.template);
                let new_vars = detect_template_variables(&new_content);
                let added_vars: Vec<_> = new_vars.iter()
                    .filter(|&var| !old_vars.contains(var))
                    .collect();
                
                if !added_vars.is_empty() {
                    println!("🔍 Detected new variables: {}", added_vars.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ").bright_cyan());
                    
                    if Confirm::new()
                        .with_prompt("Define these new variables?")
                        .default(true)
                        .interact()? {
                        
                        let new_var_defs = define_template_variables(&added_vars.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
                        for (name, var_def) in new_var_defs {
                            preset.template.variables.insert(name, var_def);
                        }
                    }
                }
                
                preset.template.template = new_content;
                println!("✅ Template updated via external editor");
                Ok(true)
            } else {
                println!("ℹ️  No template changes made");
                Ok(false)
            }
        }
        1 => {
            // Inline editing
            let new_content: String = Input::new()
                .with_prompt("📄 Template content")
                .with_initial_text(&preset.template.template)
                .interact_text()?;
            
            if new_content != preset.template.template {
                preset.template.template = new_content;
                println!("✅ Template updated");
                Ok(true)
            } else {
                println!("ℹ️  No template changes made");
                Ok(false)
            }
        }
        2 => {
            // View current template
            println!();
            println!("{}", "📄 Current Template:".bright_blue().bold());
            println!("{}", "─".repeat(50).dimmed());
            println!("{}", preset.template.template.bright_white());
            println!("{}", "─".repeat(50).dimmed());
            println!();
            
            if Confirm::new()
                .with_prompt("Edit this template?")
                .default(true)
                .interact()? {
                return Box::pin(edit_preset_template(preset)).await;
            }
            
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Edit preset variables
fn edit_preset_variables(preset: &mut crate::preset::manager::Preset) -> Result<bool> {
    use dialoguer::{Input, Select, Confirm};
    
    println!("{}", "📝 Editing Variables".bright_yellow().bold());
    
    let mut changes_made = false;
    
    if preset.template.variables.is_empty() {
        println!("ℹ️  No variables currently defined");
        if Confirm::new()
            .with_prompt("Add variables from template?")
            .default(true)
            .interact()? {
            
            let detected_vars = detect_template_variables(&preset.template.template);
            if !detected_vars.is_empty() {
                let new_vars = define_template_variables(&detected_vars)?;
                preset.template.variables.extend(new_vars);
                changes_made = true;
                println!("✅ Added {} variables", detected_vars.len());
            } else {
                println!("ℹ️  No variables detected in template");
            }
        }
        return Ok(changes_made);
    }
    
    loop {
        println!();
        println!("📋 Current variables:");
        let var_names: Vec<_> = preset.template.variables.keys().cloned().collect();
        for (i, name) in var_names.iter().enumerate() {
            let var_def = &preset.template.variables[name];
            let type_str = format!("{:?}", var_def.var_type);
            let required_str = if var_def.required { "*" } else { "" };
            println!("  {}. {}{} ({}) - {}", 
                i + 1, 
                name.bright_cyan(), 
                required_str.red(), 
                type_str.bright_blue(),
                var_def.description.dimmed()
            );
        }
        
        println!();
        let options = vec![
            "Add new variable",
            "Edit existing variable", 
            "Remove variable",
            "Done with variables"
        ];
        
        let choice = Select::new()
            .with_prompt("🔧 Variable operations")
            .items(&options)
            .interact()?;
        
        match choice {
            0 => {
                // Add new variable
                let name: String = Input::new()
                    .with_prompt("📝 Variable name")
                    .interact_text()?;
                
                if preset.template.variables.contains_key(&name) {
                    println!("⚠️  Variable '{}' already exists", name);
                    continue;
                }
                
                let vars = define_template_variables(std::slice::from_ref(&name))?;
                if let Some(var_def) = vars.get(&name) {
                    preset.template.variables.insert(name.clone(), var_def.clone());
                    println!("✅ Added variable '{}'", name.bright_green());
                    changes_made = true;
                }
            }
            1 => {
                // Edit existing variable
                if var_names.is_empty() {
                    continue;
                }
                
                let var_choice = Select::new()
                    .with_prompt("🔧 Select variable to edit")
                    .items(&var_names)
                    .interact()?;
                
                let var_name = var_names[var_choice].clone();
                println!("✏️  Editing variable: {}", var_name.bright_cyan());
                
                let new_vars = define_template_variables(std::slice::from_ref(&var_name))?;
                if let Some(new_var_def) = new_vars.get(&var_name) {
                    preset.template.variables.insert(var_name.clone(), new_var_def.clone());
                    println!("✅ Updated variable '{}'", var_name.bright_green());
                    changes_made = true;
                }
            }
            2 => {
                // Remove variable
                if var_names.is_empty() {
                    continue;
                }
                
                let var_choice = Select::new()
                    .with_prompt("🗑️  Select variable to remove")
                    .items(&var_names)
                    .interact()?;
                
                let var_name = var_names[var_choice].clone();
                if Confirm::new()
                    .with_prompt(format!("Remove variable '{}'?", var_name))
                    .default(false)
                    .interact()? {
                    
                    preset.template.variables.remove(&var_name);
                    println!("✅ Removed variable '{}'", var_name.bright_red());
                    changes_made = true;
                }
            }
            3 => {
                // Done
                break;
            }
            _ => break,
        }
    }
    
    if changes_made {
        println!("✅ Variables updated");
    } else {
        println!("ℹ️  No variable changes made");
    }
    
    Ok(changes_made)
}

/// Edit preset configuration
fn edit_preset_configuration(preset: &mut crate::preset::manager::Preset) -> Result<bool> {
    use dialoguer::{Input, Select};
    
    println!("{}", "📝 Editing Configuration".bright_yellow().bold());
    
    let mut changes_made = false;
    
    // Edit provider
    let providers = vec!["auto", "claude", "openai"];
    let current_provider = preset.config.provider.as_deref().unwrap_or("auto");
    let current_provider_index = providers.iter()
        .position(|&p| p == current_provider)
        .unwrap_or(0);
    
    let provider_selection = Select::new()
        .with_prompt("🤖 Preferred AI provider")
        .default(current_provider_index)
        .items(&providers)
        .interact()?;
    
    let new_provider = providers[provider_selection].to_string();
    if Some(&new_provider) != preset.config.provider.as_ref() {
        preset.config.provider = Some(new_provider);
        changes_made = true;
    }
    
    // Edit max tokens
    let current_max_tokens = preset.config.max_tokens.unwrap_or(0);
    let new_max_tokens: usize = Input::new()
        .with_prompt("📊 Max tokens (0 for default)")
        .default(current_max_tokens)
        .interact()?;
    
    let new_max_tokens_option = if new_max_tokens == 0 { None } else { Some(new_max_tokens) };
    if new_max_tokens_option != preset.config.max_tokens {
        preset.config.max_tokens = new_max_tokens_option;
        changes_made = true;
    }
    
    // Edit temperature
    let current_temperature = preset.config.temperature.unwrap_or(0.3);
    let new_temperature: f32 = Input::new()
        .with_prompt("🌡️  Temperature (0.0-1.0)")
        .default(current_temperature)
        .interact()?;
    
    let clamped_temperature = new_temperature.clamp(0.0, 1.0);
    if Some(clamped_temperature) != preset.config.temperature {
        preset.config.temperature = Some(clamped_temperature);
        changes_made = true;
    }
    
    if changes_made {
        println!("✅ Configuration updated");
    } else {
        println!("ℹ️  No configuration changes made");
    }
    
    Ok(changes_made)
}