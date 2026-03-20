use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SettingsProvider {
    Claude,
    Openai,
    Codex,
}

impl Default for SettingsProvider {
    fn default() -> Self {
        Self::Claude
    }
}

impl SettingsProvider {
    pub fn from_alias(value: &str) -> Option<Self> {
        match value {
            "claude" => Some(Self::Claude),
            "openai" => Some(Self::Openai),
            "codex" | "openai-codex" | "openai_codex" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Openai => "openai",
            Self::Codex => "codex",
        }
    }

    pub fn recommended_model(&self) -> &'static str {
        match self {
            Self::Claude => "claude-sonnet-4-20250514",
            Self::Openai => "gpt-5.2",
            Self::Codex => "gpt-5.3-codex",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub default: DefaultSettings,
    #[serde(default)]
    pub context: UserContextSettings,
    #[serde(default)]
    pub ui: UiSettings,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            default: Default::default(),
            context: Default::default(),
            ui: Default::default(),
        }
    }
}

impl UserConfig {
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to resolve configuration directory"))?;
        Ok(config_dir.join("termai").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::default_path()?)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read user config: {}", path.display()))?;
        let config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse user config: {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to_path(&Self::default_path()?)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create user config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize user config")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write user config: {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultSettings {
    #[serde(default)]
    pub provider: SettingsProvider,
    pub model: Option<String>,
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self {
            provider: SettingsProvider::Claude,
            model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserContextSettings {
    #[serde(default = "default_true")]
    pub smart: bool,
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
}

impl Default for UserContextSettings {
    fn default() -> Self {
        Self {
            smart: default_true(),
            token_budget: default_token_budget(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_true")]
    pub streaming: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            streaming: default_true(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    pub project_type: Option<String>,
    pub context: ProjectContextSettings,
    pub privacy: ProjectPrivacySettings,
    pub path: Option<PathBuf>,
}

impl ProjectConfig {
    pub fn file_path(root: &Path) -> PathBuf {
        root.join(".termai.toml")
    }

    pub fn load_from_root(root: &Path) -> Result<Self> {
        let path = Self::file_path(root);
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read project config: {}", path.display()))?;
        let parsed: ProjectConfigFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse project config: {}", path.display()))?;

        Ok(Self {
            project_type: parsed.project.and_then(|project| project.project_type),
            context: parsed.context.unwrap_or_default(),
            privacy: parsed.privacy.unwrap_or_default(),
            path: Some(path),
        })
    }

    pub fn save_to_root(&self, root: &Path) -> Result<()> {
        let path = Self::file_path(root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create project config directory: {}", parent.display())
            })?;
        }

        let file = ProjectConfigFile {
            version: default_version(),
            project: self.project_type.as_ref().map(|project_type| ProjectSection {
                project_type: Some(project_type.clone()),
            }),
            context: Some(self.context.clone()),
            privacy: Some(self.privacy.clone()),
        };

        let content =
            toml::to_string_pretty(&file).context("Failed to serialize project config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write project config: {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectContextSettings {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub entry_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectPrivacySettings {
    #[serde(default)]
    pub redact: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectConfigFile {
    #[serde(default = "default_version")]
    version: u32,
    project: Option<ProjectSection>,
    context: Option<ProjectContextSettings>,
    privacy: Option<ProjectPrivacySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectSection {
    #[serde(rename = "type")]
    project_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSettingsPaths {
    pub user_config_path: PathBuf,
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct SettingsOverrides {
    pub provider: Option<SettingsProvider>,
    pub model: Option<String>,
    pub smart_context: Option<bool>,
    pub token_budget: Option<usize>,
    pub theme: Option<String>,
    pub streaming: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSettings {
    pub default_provider: SettingsProvider,
    pub default_model: Option<String>,
    pub smart_context: bool,
    pub token_budget: usize,
    pub theme: String,
    pub streaming: bool,
    pub project: ProjectConfig,
    pub user_config_path: PathBuf,
    pub project_config_path: Option<PathBuf>,
}

impl ResolvedSettings {
    pub fn load_for_current_dir_with_repo<R: ConfigRepository>(
        repo: &R,
        overrides: SettingsOverrides,
    ) -> Result<Self> {
        let user_config_path = UserConfig::default_path()?;
        let _ = migrate_legacy_db_config(repo, &user_config_path);

        let current_dir =
            std::env::current_dir().context("Failed to determine current working directory")?;
        Self::load(
            ResolvedSettingsPaths {
                user_config_path,
                project_root: find_project_root(&current_dir),
            },
            overrides,
        )
    }

    pub fn load_for_current_dir(overrides: SettingsOverrides) -> Result<Self> {
        let current_dir =
            std::env::current_dir().context("Failed to determine current working directory")?;
        Self::load(
            ResolvedSettingsPaths {
                user_config_path: UserConfig::default_path()?,
                project_root: find_project_root(&current_dir),
            },
            overrides,
        )
    }

    pub fn load(paths: ResolvedSettingsPaths, overrides: SettingsOverrides) -> Result<Self> {
        let user = UserConfig::load_from_path(&paths.user_config_path)?;
        let project = match &paths.project_root {
            Some(project_root) => ProjectConfig::load_from_root(project_root)?,
            None => ProjectConfig::default(),
        };

        let default_provider = overrides
            .provider
            .clone()
            .unwrap_or_else(|| user.default.provider.clone());

        let mut default_model = overrides
            .model
            .clone()
            .or_else(|| user.default.model.clone());

        if default_model
            .as_deref()
            .is_some_and(|model| !model_matches_provider(model, &default_provider))
        {
            default_model = None;
        }

        let smart_context = overrides.smart_context.unwrap_or(user.context.smart);
        let token_budget = overrides.token_budget.unwrap_or(user.context.token_budget);
        let theme = overrides.theme.unwrap_or_else(|| user.ui.theme.clone());
        let streaming = overrides.streaming.unwrap_or(user.ui.streaming);
        let project_config_path = project.path.clone();

        Ok(Self {
            default_provider,
            default_model,
            smart_context,
            token_budget,
            theme,
            streaming,
            project,
            user_config_path: paths.user_config_path,
            project_config_path,
        })
    }

    pub fn selected_model(&self) -> String {
        self.default_model
            .clone()
            .unwrap_or_else(|| self.default_provider.recommended_model().to_string())
    }
}

pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);

    while let Some(path) = current {
        if ProjectConfig::file_path(path).exists() || path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }

    None
}

pub fn migrate_legacy_db_config<R: ConfigRepository>(
    repo: &R,
    user_config_path: &Path,
) -> Result<bool> {
    if user_config_path.exists() {
        return Ok(false);
    }

    let provider = config_service::fetch_by_key(repo, &ConfigKeys::ProviderKey.to_key())
        .ok()
        .and_then(|config| SettingsProvider::from_alias(&config.value));

    let model = load_legacy_default_model(repo, provider.as_ref());

    if provider.is_none() && model.is_none() {
        return Ok(false);
    }

    let user_config = UserConfig {
        default: DefaultSettings {
            provider: provider.unwrap_or_default(),
            model,
        },
        ..UserConfig::default()
    };
    user_config.save_to_path(user_config_path)?;
    Ok(true)
}

fn load_legacy_default_model<R: ConfigRepository>(
    repo: &R,
    provider: Option<&SettingsProvider>,
) -> Option<String> {
    let provider_specific_key = match provider {
        Some(SettingsProvider::Claude) => Some(ConfigKeys::ClaudeDefaultModel.to_key()),
        Some(SettingsProvider::Openai) => Some(ConfigKeys::OpenAIDefaultModel.to_key()),
        Some(SettingsProvider::Codex) => Some(ConfigKeys::CodexDefaultModel.to_key()),
        None => None,
    };

    if let Some(key) = provider_specific_key {
        if let Ok(config) = config_service::fetch_by_key(repo, &key) {
            return Some(config.value);
        }
    }

    [
        ConfigKeys::ClaudeDefaultModel.to_key(),
        ConfigKeys::OpenAIDefaultModel.to_key(),
        ConfigKeys::CodexDefaultModel.to_key(),
    ]
    .into_iter()
    .find_map(|key| config_service::fetch_by_key(repo, &key).ok().map(|config| config.value))
}

fn model_matches_provider(model: &str, provider: &SettingsProvider) -> bool {
    match provider {
        SettingsProvider::Claude => model.starts_with("claude"),
        SettingsProvider::Openai => {
            !model.contains("codex")
                && (model.starts_with("gpt")
                    || model.starts_with('o')
                    || model.starts_with("chatgpt")
                    || model.starts_with("computer-use"))
        }
        SettingsProvider::Codex => model.contains("codex"),
    }
}

fn default_version() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

fn default_token_budget() -> usize {
    4000
}

fn default_theme() -> String {
    "default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::keys::ConfigKeys;
    use crate::repository::db::SqliteRepository;
    use tempfile::TempDir;

    #[test]
    fn test_migrate_legacy_db_config_to_user_config() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("app.db");
        let repo = SqliteRepository::new(db_path.to_str().unwrap()).unwrap();

        crate::config::service::config_service::write_config(
            &repo,
            &ConfigKeys::ProviderKey.to_key(),
            "openai-codex",
        )
        .unwrap();
        crate::config::service::config_service::write_config(
            &repo,
            &ConfigKeys::CodexDefaultModel.to_key(),
            "gpt-5.3-codex",
        )
        .unwrap();

        let user_config_path = temp_dir.path().join("config.toml");
        let migrated = migrate_legacy_db_config(&repo, &user_config_path).unwrap();

        assert!(migrated);

        let user_config = UserConfig::load_from_path(&user_config_path).unwrap();
        assert_eq!(user_config.default.provider, SettingsProvider::Codex);
        assert_eq!(
            user_config.default.model.as_deref(),
            Some("gpt-5.3-codex")
        );
    }

    #[test]
    fn test_resolved_settings_precedence_is_flags_then_project_then_user_then_defaults() {
        let temp_dir = TempDir::new().unwrap();

        let user_config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &user_config_path,
            r#"
version = 1

[default]
provider = "claude"
model = "claude-sonnet-4-20250514"

[context]
smart = false
token_budget = 4000
"#,
        )
        .unwrap();

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(
            project_dir.join(".termai.toml"),
            r#"
version = 1

[project]
type = "rust"

[context]
include = ["src/**/*.rs"]
exclude = ["target/**"]
entry_points = ["src/main.rs"]

[privacy]
redact = ["SECRET_.*"]
"#,
        )
        .unwrap();

        let settings = ResolvedSettings::load(
            ResolvedSettingsPaths {
                user_config_path,
                project_root: Some(project_dir),
            },
            SettingsOverrides {
                provider: Some(SettingsProvider::Openai),
                model: None,
                smart_context: Some(true),
                token_budget: Some(8000),
                theme: None,
                streaming: None,
            },
        )
        .unwrap();

        assert_eq!(settings.default_provider, SettingsProvider::Openai);
        assert_eq!(settings.default_model, None);
        assert!(settings.smart_context);
        assert_eq!(settings.token_budget, 8000);
        assert_eq!(settings.project.project_type.as_deref(), Some("rust"));
        assert_eq!(settings.project.privacy.redact, vec!["SECRET_.*".to_string()]);
    }

    #[test]
    fn test_project_config_loader_only_uses_dot_termai_toml() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(project_dir.join(".termai")).unwrap();

        std::fs::write(
            project_dir.join(".termai").join("config.toml"),
            r#"
version = 1

[project]
type = "python"
"#,
        )
        .unwrap();

        let config = ProjectConfig::load_from_root(&project_dir).unwrap();

        assert_eq!(config.project_type.as_deref(), None);
        assert!(config.context.include.is_empty());
    }
}
