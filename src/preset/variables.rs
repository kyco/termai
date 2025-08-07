/// Variable system for template parameter handling
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Variable resolver for expanding different variable types
pub struct VariableResolver {
    /// Current working directory for relative path resolution
    base_path: std::path::PathBuf,
}

impl VariableResolver {
    /// Create new variable resolver
    pub fn new() -> Self {
        Self {
            base_path: std::env::current_dir().unwrap_or_default(),
        }
    }
    
    /// Set base path for relative path resolution
    #[allow(dead_code)]
    pub fn with_base_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.base_path = path.as_ref().to_path_buf();
        self
    }
    
    /// Resolve a variable value based on its type and content
    pub fn resolve_variable(
        &self,
        var_type: &crate::preset::template::VariableType,
        value: &Value,
    ) -> Result<Value> {
        use crate::preset::template::VariableType;
        
        match var_type {
            VariableType::String => Ok(value.clone()),
            VariableType::Boolean => Ok(value.clone()),
            VariableType::Number => Ok(value.clone()),
            VariableType::Array => Ok(value.clone()),
            VariableType::Object => Ok(value.clone()),
            
            VariableType::File => {
                if let Some(path_str) = value.as_str() {
                    self.resolve_file_content(path_str)
                } else {
                    bail!("File variable must be a string path")
                }
            }
            
            VariableType::Directory => {
                if let Some(path_str) = value.as_str() {
                    self.resolve_directory_listing(path_str)
                } else {
                    bail!("Directory variable must be a string path")
                }
            }
            
            VariableType::DateTime => {
                if let Some(format_str) = value.as_str() {
                    self.resolve_datetime(format_str)
                } else {
                    // Default to ISO format
                    self.resolve_datetime("%Y-%m-%d %H:%M:%S UTC")
                }
            }
            
            VariableType::GitInfo => {
                if let Some(info_type) = value.as_str() {
                    self.resolve_git_info(info_type)
                } else {
                    bail!("GitInfo variable must specify info type")
                }
            }
            
            VariableType::Environment => {
                if let Some(var_name) = value.as_str() {
                    self.resolve_environment_var(var_name)
                } else {
                    bail!("Environment variable must specify variable name")
                }
            }
        }
    }
    
    /// Resolve file content variable
    fn resolve_file_content(&self, path_str: &str) -> Result<Value> {
        let path = if Path::new(path_str).is_relative() {
            self.base_path.join(path_str)
        } else {
            Path::new(path_str).to_path_buf()
        };
        
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
            
        Ok(Value::String(content))
    }
    
    /// Resolve directory listing variable
    fn resolve_directory_listing(&self, path_str: &str) -> Result<Value> {
        let path = if Path::new(path_str).is_relative() {
            self.base_path.join(path_str)
        } else {
            Path::new(path_str).to_path_buf()
        };
        
        let entries = fs::read_dir(&path)
            .with_context(|| format!("Failed to read directory: {}", path.display()))?;
            
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            files.push(Value::String(name));
        }
        
        Ok(Value::Array(files))
    }
    
    /// Resolve datetime variable with formatting
    fn resolve_datetime(&self, format: &str) -> Result<Value> {
        let now = chrono::Utc::now();
        let formatted = now.format(format).to_string();
        Ok(Value::String(formatted))
    }
    
    /// Resolve Git repository information
    fn resolve_git_info(&self, info_type: &str) -> Result<Value> {
        let value = match info_type {
            "branch" => self.get_git_branch(),
            "commit" => self.get_git_commit(),
            "commit_short" => self.get_git_commit_short(),
            "author" => self.get_git_author(),
            "email" => self.get_git_email(),
            "status" => self.get_git_status(),
            "remote" => self.get_git_remote(),
            _ => return Ok(Value::String(format!("Unknown git info type: {}", info_type))),
        };
        
        Ok(Value::String(value))
    }
    
    /// Resolve environment variable
    fn resolve_environment_var(&self, var_name: &str) -> Result<Value> {
        let value = std::env::var(var_name)
            .unwrap_or_else(|_| format!("[Environment variable {} not set]", var_name));
        Ok(Value::String(value))
    }
    
    // Git helper methods
    fn get_git_branch(&self) -> String {
        self.run_git_command(&["branch", "--show-current"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_commit(&self) -> String {
        self.run_git_command(&["rev-parse", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_commit_short(&self) -> String {
        self.run_git_command(&["rev-parse", "--short", "HEAD"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_author(&self) -> String {
        self.run_git_command(&["config", "user.name"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_email(&self) -> String {
        self.run_git_command(&["config", "user.email"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_status(&self) -> String {
        self.run_git_command(&["status", "--porcelain"])
            .map(|output| {
                if output.is_empty() {
                    "clean".to_string()
                } else {
                    "dirty".to_string()
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    fn get_git_remote(&self) -> String {
        self.run_git_command(&["remote", "get-url", "origin"])
            .unwrap_or_else(|| "unknown".to_string())
    }
    
    /// Run Git command and return stdout as string
    fn run_git_command(&self, args: &[&str]) -> Option<String> {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&self.base_path)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout).ok()
                } else {
                    None
                }
            })
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

/// Interactive variable collector for prompting users
pub struct VariableCollector {
    resolver: VariableResolver,
}

impl VariableCollector {
    /// Create new variable collector
    pub fn new() -> Self {
        Self {
            resolver: VariableResolver::new(),
        }
    }
    
    /// Collect variables interactively from user
    pub fn collect_variables(
        &self,
        template: &crate::preset::template::Template,
    ) -> Result<HashMap<String, Value>> {
        use dialoguer::{Confirm, Input};
        
        let mut variables = HashMap::new();
        
        println!("📝 {} - {}", template.name, template.description);
        println!();
        
        for (name, var_def) in &template.variables {
            let value = match &var_def.var_type {
                crate::preset::template::VariableType::Boolean => {
                    let default = var_def.default
                        .as_ref()
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                        
                    let result = Confirm::new()
                        .with_prompt(&var_def.description)
                        .default(default)
                        .interact()?;
                        
                    Value::Bool(result)
                }
                
                _ => {
                    let default = var_def.default
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    let mut input = Input::<String>::new()
                        .with_prompt(&var_def.description);
                    
                    if !default.is_empty() {
                        input = input.default(default);
                    }
                    
                    if !var_def.required {
                        input = input.allow_empty(true);
                    }
                    
                    let result = input.interact_text()?;
                    
                    // Resolve special variable types
                    self.resolver.resolve_variable(&var_def.var_type, &Value::String(result))?
                }
            };
            
            variables.insert(name.clone(), value);
        }
        
        Ok(variables)
    }
    
    /// Collect variables non-interactively from provided values
    pub fn collect_from_values(
        &self,
        template: &crate::preset::template::Template,
        provided: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>> {
        let mut variables = HashMap::new();
        
        for (name, var_def) in &template.variables {
            if let Some(value) = provided.get(name) {
                // Resolve the provided value
                let resolved = self.resolver.resolve_variable(&var_def.var_type, value)?;
                variables.insert(name.clone(), resolved);
            } else if let Some(default) = &var_def.default {
                // Use default value
                let resolved = self.resolver.resolve_variable(&var_def.var_type, default)?;
                variables.insert(name.clone(), resolved);
            } else if var_def.required {
                bail!("Required variable '{}' not provided", name);
            }
        }
        
        Ok(variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::template::VariableType;
    
    #[test]
    fn test_variable_resolver_string() {
        let resolver = VariableResolver::new();
        let result = resolver.resolve_variable(
            &VariableType::String,
            &Value::String("test".to_string()),
        ).unwrap();
        
        assert_eq!(result, Value::String("test".to_string()));
    }
    
    #[test]
    fn test_variable_resolver_datetime() {
        let resolver = VariableResolver::new();
        let result = resolver.resolve_variable(
            &VariableType::DateTime,
            &Value::String("%Y".to_string()),
        ).unwrap();
        
        // Should return current year as string
        if let Value::String(year) = result {
            assert_eq!(year.len(), 4);
            assert!(year.parse::<i32>().is_ok());
        } else {
            panic!("Expected string value");
        }
    }
    
    #[test]
    fn test_variable_resolver_environment() {
        let resolver = VariableResolver::new();
        
        // Set a test environment variable
        std::env::set_var("TEST_VAR", "test_value");
        
        let result = resolver.resolve_variable(
            &VariableType::Environment,
            &Value::String("TEST_VAR".to_string()),
        ).unwrap();
        
        assert_eq!(result, Value::String("test_value".to_string()));
    }
}