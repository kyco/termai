/// Preset management operations and storage
use crate::preset::template::{Template, TemplateRenderer};
use crate::preset::variables::VariableCollector;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Complete preset combining template with configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Preset identification
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    
    /// Core template
    pub template: Template,
    
    /// Preset configuration
    pub config: PresetConfig,
}

/// Configuration options for preset execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct PresetConfig {
    /// Preferred AI provider
    pub provider: Option<String>,
    /// Token limits
    pub max_tokens: Option<usize>,
    /// Temperature setting
    pub temperature: Option<f32>,
    /// Context discovery settings
    pub smart_context: Option<bool>,
    /// Session settings
    pub session: Option<String>,
}


/// Manager for preset operations
pub struct PresetManager {
    /// User preset directory
    user_presets_dir: PathBuf,
    /// Built-in preset directory (from application)
    builtin_presets_dir: Option<PathBuf>,
    /// Template renderer
    #[allow(dead_code)]
    renderer: TemplateRenderer,
    /// Variable collector
    #[allow(dead_code)]
    collector: VariableCollector,
}

impl PresetManager {
    /// Create new preset manager
    pub fn new() -> Result<Self> {
        let user_presets_dir = Self::get_user_presets_dir()?;
        
        // Ensure user preset directory exists
        fs::create_dir_all(&user_presets_dir)
            .context("Failed to create user presets directory")?;
        
        Ok(Self {
            user_presets_dir,
            builtin_presets_dir: None,
            renderer: TemplateRenderer::new()?,
            collector: VariableCollector::new(),
        })
    }
    
    /// Set built-in presets directory (for application-provided presets)
    #[allow(dead_code)]
    pub fn with_builtin_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.builtin_presets_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// Get user presets directory
    fn get_user_presets_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get user configuration directory")?;
        
        Ok(config_dir.join("termai").join("presets"))
    }
    
    /// List all available presets
    pub fn list_presets(&self) -> Result<Vec<PresetInfo>> {
        let mut presets = Vec::new();
        
        // Add built-in presets
        if let Some(builtin_dir) = &self.builtin_presets_dir {
            presets.extend(self.scan_preset_directory(builtin_dir, true)?);
        }
        
        // Add user presets
        presets.extend(self.scan_preset_directory(&self.user_presets_dir, false)?);
        
        // Sort by name
        presets.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(presets)
    }
    
    /// Scan directory for preset files
    fn scan_preset_directory(&self, dir: &Path, is_builtin: bool) -> Result<Vec<PresetInfo>> {
        let mut presets = Vec::new();
        
        if !dir.exists() {
            return Ok(presets);
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml" || ext == "yml") {
                match self.load_preset_info(&path) {
                    Ok(mut info) => {
                        info.is_builtin = is_builtin;
                        presets.push(info);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load preset {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(presets)
    }
    
    /// Load preset information without full template parsing
    fn load_preset_info(&self, path: &Path) -> Result<PresetInfo> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read preset file: {}", path.display()))?;
        
        let preset: Preset = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse preset file: {}", path.display()))?;
        
        Ok(PresetInfo {
            name: preset.name,
            description: preset.description,
            category: preset.category,
            version: preset.version,
            path: path.to_path_buf(),
            is_builtin: false, // Will be set by caller
            usage_count: preset.template.metadata.usage_count,
        })
    }
    
    /// Load a preset by name
    pub fn load_preset(&self, name: &str) -> Result<Preset> {
        let preset_info = self.find_preset(name)?;
        self.load_preset_from_path(&preset_info.path)
    }
    
    /// Load preset from file path
    pub fn load_preset_from_path(&self, path: &Path) -> Result<Preset> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read preset file: {}", path.display()))?;
        
        let preset: Preset = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse preset file: {}", path.display()))?;
        
        Ok(preset)
    }
    
    /// Find preset by name
    pub fn find_preset(&self, name: &str) -> Result<PresetInfo> {
        let presets = self.list_presets()?;
        
        presets
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Preset '{}' not found", name))
    }
    
    /// Use a preset interactively
    #[allow(dead_code)]
    pub fn use_preset_interactive(&self, name: &str) -> Result<String> {
        let preset = self.load_preset(name)?;
        
        // Collect variables interactively
        let variables = self.collector.collect_variables(&preset.template)?;
        
        // Render template
        self.renderer.render(&preset.template, &variables)
    }
    
    /// Use a preset with provided variables
    #[allow(dead_code)]
    pub fn use_preset_with_variables(
        &self,
        name: &str,
        variables: &HashMap<String, Value>,
    ) -> Result<String> {
        let preset = self.load_preset(name)?;
        
        // Collect/resolve variables
        let resolved_variables = self.collector.collect_from_values(&preset.template, variables)?;
        
        // Render template
        self.renderer.render(&preset.template, &resolved_variables)
    }
    
    /// Save a preset
    pub fn save_preset(&self, preset: &Preset) -> Result<()> {
        let filename = format!("{}.yaml", preset.name.replace(' ', "-").to_lowercase());
        let path = self.user_presets_dir.join(filename);
        
        let content = serde_yaml::to_string(preset)
            .context("Failed to serialize preset")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write preset to {}", path.display()))?;
        
        println!("✅ Preset '{}' saved to {}", preset.name, path.display());
        
        Ok(())
    }
    
    /// Delete a preset (user presets only)
    pub fn delete_preset(&self, name: &str) -> Result<()> {
        let preset_info = self.find_preset(name)?;
        
        if preset_info.is_builtin {
            bail!("Cannot delete built-in preset '{}'", name);
        }
        
        fs::remove_file(&preset_info.path)
            .with_context(|| format!("Failed to delete preset file: {}", preset_info.path.display()))?;
        
        println!("✅ Preset '{}' deleted", name);
        
        Ok(())
    }
    
    /// Export preset to file
    pub fn export_preset(&self, name: &str, output_path: &Path) -> Result<()> {
        let preset = self.load_preset(name)?;
        
        let content = serde_yaml::to_string(&preset)
            .context("Failed to serialize preset")?;
        
        fs::write(output_path, content)
            .with_context(|| format!("Failed to write preset to {}", output_path.display()))?;
        
        println!("✅ Preset '{}' exported to {}", name, output_path.display());
        
        Ok(())
    }
    
    /// Import preset from file
    pub fn import_preset(&self, file_path: &Path) -> Result<()> {
        let preset = self.load_preset_from_path(file_path)?;
        self.save_preset(&preset)?;
        
        println!("✅ Preset '{}' imported", preset.name);
        
        Ok(())
    }
    
    /// Search presets by name or description
    pub fn search_presets(&self, query: &str) -> Result<Vec<PresetInfo>> {
        let presets = self.list_presets()?;
        let query_lower = query.to_lowercase();
        
        let matches: Vec<PresetInfo> = presets
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.category.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(matches)
    }
}

/// Preset information for listing and discovery
#[derive(Debug, Clone)]
pub struct PresetInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub path: PathBuf,
    pub is_builtin: bool,
    pub usage_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::template::TemplateVariable;
    
    #[test]
    fn test_preset_creation() {
        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable::required_string("Test name".to_string()),
        );
        
        let template = Template::new(
            "test-template".to_string(),
            "Test template".to_string(),
            "Hello {{name}}!".to_string(),
            variables,
        ).unwrap();
        
        let preset = Preset {
            name: "test-preset".to_string(),
            description: "Test preset".to_string(),
            category: "test".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig::default(),
        };
        
        assert_eq!(preset.name, "test-preset");
        assert_eq!(preset.category, "test");
    }
    
    #[test]
    fn test_preset_serialization() {
        let mut variables = HashMap::new();
        variables.insert(
            "name".to_string(),
            TemplateVariable::required_string("Test name".to_string()),
        );
        
        let template = Template::new(
            "test-template".to_string(),
            "Test template".to_string(),
            "Hello {{name}}!".to_string(),
            variables,
        ).unwrap();
        
        let preset = Preset {
            name: "test-preset".to_string(),
            description: "Test preset".to_string(),
            category: "test".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig::default(),
        };
        
        // Test YAML serialization
        let yaml = serde_yaml::to_string(&preset).unwrap();
        let deserialized: Preset = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(preset.name, deserialized.name);
        assert_eq!(preset.template.name, deserialized.template.name);
    }
}