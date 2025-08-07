/// Template parsing and rendering system
use anyhow::{Context, Result, bail};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Core template struct for storing reusable prompt templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template name identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Template content with variable placeholders
    pub template: String,
    /// Variable definitions and metadata
    pub variables: HashMap<String, TemplateVariable>,
    /// Optional template metadata
    pub metadata: TemplateMetadata,
}

/// Variable definition for template parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    /// Variable data type
    #[serde(rename = "type")]
    pub var_type: VariableType,
    /// Whether this variable is required
    #[serde(default)]
    pub required: bool,
    /// Default value if not provided
    pub default: Option<Value>,
    /// Human-readable description
    pub description: String,
    /// Optional validation pattern for strings
    pub pattern: Option<String>,
}

/// Supported variable data types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    String,
    Boolean,
    Number,
    Array,
    Object,
    File,
    Directory,
    DateTime,
    GitInfo,
    Environment,
}

/// Template metadata for organization and discovery
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateMetadata {
    /// Template version
    pub version: Option<String>,
    /// Author information
    pub author: Option<String>,
    /// Template category/tags
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Last modification timestamp
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Usage statistics
    pub usage_count: u32,
}

/// Template renderer with Handlebars backend
pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    /// Create new template renderer
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();
        
        // Configure Handlebars settings
        handlebars.set_strict_mode(true);
        
        // Register custom helpers
        Self::register_helpers(&mut handlebars)?;
        
        Ok(Self { handlebars })
    }
    
    /// Register custom Handlebars helpers for advanced functionality
    fn register_helpers(handlebars: &mut Handlebars) -> Result<()> {
        // Helper for current date/time formatting
        handlebars.register_helper("now", Box::new(helpers::now_helper));
        
        // Helper for file content inclusion
        handlebars.register_helper("file", Box::new(helpers::file_helper));
        
        // Helper for environment variable expansion
        handlebars.register_helper("env", Box::new(helpers::env_helper));
        
        // Helper for Git repository information
        handlebars.register_helper("git", Box::new(helpers::git_helper));
        
        Ok(())
    }
    
    /// Render template with provided variables
    pub fn render(&self, template: &Template, variables: &HashMap<String, Value>) -> Result<String> {
        // Validate variables before rendering
        self.validate_variables(template, variables)?;
        
        // Merge provided variables with defaults
        let context = self.build_context(template, variables)?;
        
        // Render the template
        self.handlebars
            .render_template(&template.template, &context)
            .context("Failed to render template")
    }
    
    /// Validate that all required variables are provided and types are correct
    fn validate_variables(
        &self, 
        template: &Template, 
        variables: &HashMap<String, Value>
    ) -> Result<()> {
        for (name, var_def) in &template.variables {
            if var_def.required && !variables.contains_key(name) {
                bail!("Required variable '{}' not provided", name);
            }
            
            if let Some(value) = variables.get(name) {
                self.validate_variable_type(name, var_def, value)?;
            }
        }
        Ok(())
    }
    
    /// Validate individual variable type
    fn validate_variable_type(
        &self,
        name: &str,
        var_def: &TemplateVariable,
        value: &Value
    ) -> Result<()> {
        let matches = match (&var_def.var_type, value) {
            (VariableType::String, Value::String(_)) => true,
            (VariableType::Boolean, Value::Bool(_)) => true,
            (VariableType::Number, Value::Number(_)) => true,
            (VariableType::Array, Value::Array(_)) => true,
            (VariableType::Object, Value::Object(_)) => true,
            // Special types are handled as strings
            (VariableType::File | VariableType::Directory | VariableType::DateTime | 
             VariableType::GitInfo | VariableType::Environment, Value::String(_)) => true,
            _ => false,
        };
        
        if !matches {
            bail!(
                "Variable '{}' has incorrect type. Expected {:?}, got {:?}", 
                name, var_def.var_type, value
            );
        }
        
        Ok(())
    }
    
    /// Build template context by merging provided variables with defaults
    fn build_context(
        &self, 
        template: &Template, 
        variables: &HashMap<String, Value>
    ) -> Result<Value> {
        let mut context = serde_json::Map::new();
        
        // Add default values for undefined variables
        for (name, var_def) in &template.variables {
            if let Some(value) = variables.get(name) {
                context.insert(name.clone(), value.clone());
            } else if let Some(default) = &var_def.default {
                context.insert(name.clone(), default.clone());
            }
        }
        
        Ok(Value::Object(context))
    }
}

impl Template {
    /// Create new template with basic validation
    pub fn new(
        name: String,
        description: String,
        template: String,
        variables: HashMap<String, TemplateVariable>,
    ) -> Result<Self> {
        Self::validate_template_syntax(&template)?;
        
        Ok(Self {
            name,
            description,
            template,
            variables,
            metadata: TemplateMetadata::default(),
        })
    }
    
    /// Validate template syntax without rendering
    pub fn validate_template_syntax(template_str: &str) -> Result<()> {
        let handlebars = Handlebars::new();
        handlebars
            .render_template(template_str, &serde_json::json!({}))
            .context("Invalid template syntax")?;
        Ok(())
    }
    
    /// Get list of variables required by this template
    #[allow(dead_code)]
    pub fn required_variables(&self) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, var)| var.required)
            .map(|(name, _)| name.as_str())
            .collect()
    }
    
    /// Get list of optional variables with defaults
    #[allow(dead_code)]
    pub fn optional_variables(&self) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, var)| !var.required)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

impl TemplateVariable {
    /// Create new template variable
    pub fn new(
        var_type: VariableType,
        description: String,
        required: bool,
        default: Option<Value>,
    ) -> Self {
        Self {
            var_type,
            required,
            default,
            description,
            pattern: None,
        }
    }
    
    /// Create required string variable
    pub fn required_string(description: String) -> Self {
        Self::new(VariableType::String, description, true, None)
    }
    
    /// Create optional string variable with default
    pub fn optional_string(description: String, default: String) -> Self {
        Self::new(
            VariableType::String, 
            description, 
            false, 
            Some(Value::String(default))
        )
    }
    
    /// Create boolean variable with default
    pub fn boolean(description: String, default: bool) -> Self {
        Self::new(
            VariableType::Boolean, 
            description, 
            false, 
            Some(Value::Bool(default))
        )
    }
}

/// Custom Handlebars helpers for advanced template functionality
mod helpers {
    use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
    use std::fs;
    
    /// Helper for current date/time formatting
    pub fn now_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let format = h.param(0)
            .map(|v| v.value().as_str().unwrap_or("%Y-%m-%d %H:%M:%S"))
            .unwrap_or("%Y-%m-%d %H:%M:%S");
            
        let now = chrono::Utc::now();
        let formatted = now.format(format).to_string();
        out.write(&formatted)?;
        Ok(())
    }
    
    /// Helper for file content inclusion
    pub fn file_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(path) = h.param(0) {
            if let Some(path_str) = path.value().as_str() {
                match fs::read_to_string(path_str) {
                    Ok(content) => out.write(&content)?,
                    Err(_) => out.write(&format!("[Error: Could not read file {}]", path_str))?,
                }
            }
        }
        Ok(())
    }
    
    /// Helper for environment variable expansion
    pub fn env_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(var_name) = h.param(0) {
            if let Some(name) = var_name.value().as_str() {
                let value = std::env::var(name).unwrap_or_default();
                out.write(&value)?;
            }
        }
        Ok(())
    }
    
    /// Helper for Git repository information
    pub fn git_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        if let Some(info_type) = h.param(0) {
            if let Some(info) = info_type.value().as_str() {
                let value = match info {
                    "branch" => get_git_branch(),
                    "commit" => get_git_commit(),
                    "author" => get_git_author(),
                    _ => String::new(),
                };
                out.write(&value)?;
            }
        }
        Ok(())
    }
    
    fn get_git_branch() -> String {
        // Simplified Git branch detection
        std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
    
    fn get_git_commit() -> String {
        // Simplified Git commit detection
        std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
    
    fn get_git_author() -> String {
        // Simplified Git author detection
        std::process::Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_creation() {
        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable::required_string("User name".to_string()),
        );
        
        let template = Template::new(
            "test".to_string(),
            "Test template".to_string(),
            "Hello {{name}}!".to_string(),
            variables,
        ).unwrap();
        
        assert_eq!(template.name, "test");
        assert_eq!(template.required_variables(), vec!["name"]);
    }
    
    #[test]
    fn test_template_rendering() {
        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable::required_string("User name".to_string()),
        );
        
        let template = Template::new(
            "test".to_string(),
            "Test template".to_string(),
            "Hello {{name}}!".to_string(),
            variables,
        ).unwrap();
        
        let renderer = TemplateRenderer::new().unwrap();
        let mut context = HashMap::new();
        context.insert("name".to_string(), serde_json::json!("World"));
        
        let result = renderer.render(&template, &context).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_variable_validation() {
        let mut variables = HashMap::new();
        variables.insert(
            "required_var".to_string(),
            TemplateVariable::required_string("Required variable".to_string()),
        );
        
        let template = Template::new(
            "test".to_string(),
            "Test template".to_string(),
            "{{required_var}}".to_string(),
            variables,
        ).unwrap();
        
        let renderer = TemplateRenderer::new().unwrap();
        let context = HashMap::new(); // Empty context - should fail
        
        let result = renderer.render(&template, &context);
        assert!(result.is_err());
    }
}