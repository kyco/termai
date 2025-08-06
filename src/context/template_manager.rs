use crate::context::config::ContextConfig;
use crate::context::templates::{ContextTemplate, ContextTemplateLibrary};
use crate::context::SmartContext;
use anyhow::Result;
use serde_json;
use std::fs;
use std::path::Path;
use walkdir;

/// Manager for context templates that provides integration between
/// templates and smart context discovery system
#[allow(dead_code)]
pub struct TemplateManager;

impl TemplateManager {
    /// Apply a template to smart context discovery
    pub fn apply_template(
        template_name: &str,
        base_config: ContextConfig,
    ) -> Result<(ContextConfig, ContextTemplate)> {
        let template = ContextTemplateLibrary::get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

        let mut config = base_config;
        template.apply_to_config(&mut config);

        Ok((config, template))
    }

    /// Get all available templates with their descriptions
    pub fn list_available_templates() -> Vec<(String, String)> {
        ContextTemplateLibrary::list_templates()
    }

    /// Save a custom template to a project's .termai directory
    pub fn save_custom_template(
        project_path: &Path,
        template_name: &str,
        template: &ContextTemplate,
    ) -> Result<()> {
        let termai_dir = project_path.join(".termai");
        fs::create_dir_all(&termai_dir)?;

        let template_file = termai_dir.join(format!("{}.template.json", template_name));
        let template_json = serde_json::to_string_pretty(template)?;
        fs::write(template_file, template_json)?;

        Ok(())
    }

    /// Load a custom template from a project's .termai directory
    pub fn load_custom_template(
        project_path: &Path,
        template_name: &str,
    ) -> Result<Option<ContextTemplate>> {
        let template_file = project_path.join(".termai").join(format!("{}.template.json", template_name));
        
        if !template_file.exists() {
            return Ok(None);
        }

        let template_json = fs::read_to_string(template_file)?;
        let template: ContextTemplate = serde_json::from_str(&template_json)?;
        Ok(Some(template))
    }

    /// List custom templates available in a project
    pub fn list_custom_templates(project_path: &Path) -> Result<Vec<String>> {
        let termai_dir = project_path.join(".termai");
        
        if !termai_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();
        
        for entry in fs::read_dir(termai_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".template.json") {
                    let template_name = file_name
                        .strip_suffix(".template.json")
                        .unwrap_or(file_name);
                    templates.push(template_name.to_string());
                }
            }
        }

        Ok(templates)
    }

    /// Create a SmartContext with template-specific configuration
    pub fn create_smart_context_with_template(
        template_name: &str,
        project_path: &Path,
    ) -> Result<(SmartContext, ContextTemplate)> {
        // Try custom template first, then built-in templates
        let template = if let Some(custom_template) = Self::load_custom_template(project_path, template_name)? {
            custom_template
        } else {
            ContextTemplateLibrary::get_template(template_name)
                .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?
        };

        // Start with project-specific config if available
        let base_config = ContextConfig::discover_config(project_path)?;
        
        // Apply template
        let mut config = base_config;
        template.apply_to_config(&mut config);

        let smart_context = SmartContext::with_config(config);
        Ok((smart_context, template))
    }

    /// Generate a template recommendation based on project structure
    pub fn recommend_template(project_path: &Path) -> Result<Vec<(String, f32)>> {
        let mut recommendations = Vec::new();
        
        // Analyze project structure to recommend templates
        let project_files = Self::scan_project_structure(project_path)?;
        
        // Score templates based on relevance to project structure
        for (name, template) in ContextTemplateLibrary::get_all_templates() {
            let score = Self::calculate_template_relevance(&project_files, &template);
            if score > 0.3 { // Only recommend templates with decent relevance
                recommendations.push((name, score));
            }
        }

        // Sort by relevance score (highest first)
        recommendations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(recommendations)
    }

    /// Interactive template selection with descriptions
    pub fn display_template_menu() -> String {
        let templates = Self::list_available_templates();
        let mut menu = String::new();
        
        menu.push_str("📋 Available Context Templates:\n");
        menu.push_str("══════════════════════════════\n\n");
        
        for (i, (name, description)) in templates.iter().enumerate() {
            menu.push_str(&format!(
                "{:2}. 🎯 {}\n    💡 {}\n\n",
                i + 1,
                name,
                description
            ));
        }
        
        menu.push_str("Select a template by name or number, or press Enter for default smart context discovery.\n");
        menu
    }

    /// Generate example .termai.toml with template integration
    pub fn generate_template_config_example(template_name: &str) -> Result<String> {
        let template = ContextTemplateLibrary::get_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template '{}' not found", template_name))?;

        let mut config_example = String::new();
        config_example.push_str(&format!("# TermAI Configuration with {} Template\n", template.name));
        config_example.push_str("# Generated automatically - customize as needed\n\n");
        
        config_example.push_str("[context]\n");
        
        if let Some(max_tokens) = template.max_tokens {
            config_example.push_str(&format!("max_tokens = {}\n", max_tokens));
        }
        
        config_example.push_str(&format!("include = [\n"));
        for pattern in &template.include_patterns {
            config_example.push_str(&format!("  \"{}\",\n", pattern));
        }
        config_example.push_str("]\n\n");
        
        config_example.push_str(&format!("exclude = [\n"));
        for pattern in &template.exclude_patterns {
            config_example.push_str(&format!("  \"{}\",\n", pattern));
        }
        config_example.push_str("]\n\n");
        
        config_example.push_str(&format!("priority_patterns = [\n"));
        for pattern in &template.priority_patterns {
            config_example.push_str(&format!("  \"{}\",\n", pattern));
        }
        config_example.push_str("]\n\n");
        
        config_example.push_str("[project]\n");
        config_example.push_str("# Override project type detection if needed\n");
        config_example.push_str("# type = \"rust\"  # rust, javascript, python, go, java, kotlin\n\n");
        
        config_example.push_str(&format!("# Template: {}\n", template.name));
        config_example.push_str(&format!("# Description: {}\n", template.description));
        config_example.push_str(&format!("# Focus Areas: {}\n", template.focus_areas.join(", ")));
        
        Ok(config_example)
    }

    /// Scan project structure to determine file types and patterns
    fn scan_project_structure(project_path: &Path) -> Result<ProjectStructure> {
        let mut structure = ProjectStructure::new();
        
        // Walk through project files
        for entry in walkdir::WalkDir::new(project_path)
            .max_depth(3) // Don't go too deep for performance
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    structure.add_file(file_name);
                }
            }
        }
        
        Ok(structure)
    }

    /// Calculate how relevant a template is to a project structure
    fn calculate_template_relevance(structure: &ProjectStructure, template: &ContextTemplate) -> f32 {
        let mut score = 0.0;
        let mut max_possible = 0.0;
        
        // Check focus areas against project characteristics
        for focus_area in &template.focus_areas {
            max_possible += 1.0;
            
            let area_score = match focus_area.to_lowercase().as_str() {
                "authentication" | "security" => {
                    if structure.has_auth_files() || structure.has_security_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                "testing" => {
                    if structure.has_test_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                "api development" | "rest api design" => {
                    if structure.has_api_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                "database" | "data modeling" => {
                    if structure.has_database_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                "performance optimization" => {
                    if structure.has_performance_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                "documentation" => {
                    if structure.has_documentation_files() {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => 0.5, // Default relevance for other focus areas
            };
            
            score += area_score;
        }
        
        if max_possible > 0.0 {
            score / max_possible
        } else {
            0.0
        }
    }
}

/// Structure representing analyzed project characteristics
#[derive(Debug)]
struct ProjectStructure {
    files: Vec<String>,
}

impl ProjectStructure {
    fn new() -> Self {
        Self { files: Vec::new() }
    }
    
    fn add_file(&mut self, filename: &str) {
        self.files.push(filename.to_lowercase());
    }
    
    fn has_auth_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("auth") || f.contains("login") || f.contains("security"))
    }
    
    fn has_security_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("security") || f.contains("crypto") || f.contains("token"))
    }
    
    fn has_test_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("test") || f.contains("spec") || f == "cargo.toml" && self.has_rust_files())
    }
    
    fn has_api_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("api") || f.contains("route") || f.contains("handler") || f.contains("controller"))
    }
    
    fn has_database_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("model") || f.contains("entity") || f.contains("repository") || f.contains("migration") || f.contains("schema"))
    }
    
    fn has_performance_files(&self) -> bool {
        self.files.iter().any(|f| f.contains("perf") || f.contains("optimize") || f.contains("cache") || f.contains("benchmark"))
    }
    
    fn has_documentation_files(&self) -> bool {
        self.files.iter().any(|f| f.ends_with(".md") || f == "readme.md" || f.starts_with("doc"))
    }
    
    fn has_rust_files(&self) -> bool {
        self.files.iter().any(|f| f.ends_with(".rs"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_apply_template() {
        let base_config = ContextConfig::default();
        let result = TemplateManager::apply_template("security", base_config);
        
        assert!(result.is_ok());
        let (config, template) = result.unwrap();
        
        // Should have security-specific patterns
        assert!(config.context.include.contains(&"**/auth*/**".to_string()));
        assert_eq!(template.name, "Security Analysis");
    }

    #[test]
    fn test_list_templates() {
        let templates = TemplateManager::list_available_templates();
        assert!(!templates.is_empty());
        
        let names: Vec<String> = templates.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"security".to_string()));
        assert!(names.contains(&"refactoring".to_string()));
    }

    #[test]
    fn test_custom_template_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create a custom template
        let custom_template = ContextTemplate::new(
            "Custom Test".to_string(),
            "A custom template for testing".to_string(),
            vec!["*.test".to_string()],
            vec!["*.ignore".to_string()],
            vec!["test*".to_string()],
            Some(1000),
            vec!["Testing".to_string()],
            vec!["Run tests".to_string()],
        );

        // Save custom template
        let save_result = TemplateManager::save_custom_template(
            project_path,
            "custom_test",
            &custom_template,
        );
        assert!(save_result.is_ok());

        // Load custom template
        let loaded_template = TemplateManager::load_custom_template(project_path, "custom_test").unwrap();
        assert!(loaded_template.is_some());
        
        let loaded = loaded_template.unwrap();
        assert_eq!(loaded.name, "Custom Test");
        assert_eq!(loaded.max_tokens, Some(1000));

        // List custom templates
        let custom_templates = TemplateManager::list_custom_templates(project_path).unwrap();
        assert_eq!(custom_templates, vec!["custom_test"]);
    }

    #[test]
    fn test_generate_config_example() {
        let config_example = TemplateManager::generate_template_config_example("security").unwrap();
        
        assert!(config_example.contains("[context]"));
        assert!(config_example.contains("Security Analysis"));
        assert!(config_example.contains("**/auth*/**"));
        assert!(config_example.contains("max_tokens"));
    }

    #[test]
    fn test_project_structure_analysis() {
        let mut structure = ProjectStructure::new();
        structure.add_file("auth.rs");
        structure.add_file("test_main.rs");
        structure.add_file("README.md");
        
        assert!(structure.has_auth_files());
        assert!(structure.has_test_files());
        assert!(structure.has_documentation_files());
        assert!(!structure.has_api_files());
    }

    #[test]
    fn test_template_recommendation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create some files to analyze
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/auth.rs"), "// auth code").unwrap();
        std::fs::write(project_path.join("tests/auth_test.rs"), "// test code").unwrap();
        std::fs::write(project_path.join("README.md"), "# Project").unwrap();
        
        let recommendations = TemplateManager::recommend_template(project_path).unwrap();
        
        // Should have some recommendations
        assert!(!recommendations.is_empty());
        
        // Should include security template due to auth.rs file
        let template_names: Vec<String> = recommendations.iter().map(|(name, _)| name.clone()).collect();
        assert!(template_names.contains(&"security".to_string()));
    }

    #[test]
    fn test_display_menu() {
        let menu = TemplateManager::display_template_menu();
        
        assert!(menu.contains("Available Context Templates"));
        assert!(menu.contains("Security Analysis"));
        assert!(menu.contains("Code Refactoring"));
        assert!(menu.contains("Select a template"));
    }

    #[test]
    fn test_invalid_template() {
        let base_config = ContextConfig::default();
        let result = TemplateManager::apply_template("nonexistent_template", base_config);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}