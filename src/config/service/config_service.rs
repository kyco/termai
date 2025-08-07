use crate::config::entity::config_entity::ConfigEntity;
use crate::config::env::EnvResolver;
use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use anyhow::{anyhow, Result};

pub fn write_config<R: ConfigRepository>(repo: &R, key: &str, value: &str) -> Result<()> {
    let config = match repo.fetch_by_key(key) {
        Ok(config) => Some(config),
        Err(_) => None,
    };

    let _ = match config {
        Some(c) => repo.update_config(c.id.unwrap_or_default(), key, value),
        None => repo.add_config(key, value),
    };

    Ok(())
}

pub fn fetch_by_key<R: ConfigRepository>(repo: &R, key: &str) -> Result<ConfigEntity> {
    match repo.fetch_by_key(key) {
        Ok(config) => Ok(config),
        Err(_) => Err(anyhow!("failed to fetch configs")),
    }
}

pub fn fetch_config<R: ConfigRepository>(repo: &R) -> Result<Vec<ConfigEntity>> {
    match repo.fetch_all_configs() {
        Ok(configs) => Ok(configs),
        Err(_) => Err(anyhow!("failed to fetch configs")),
    }
}

/// Fetch configuration value with environment variable fallback
pub fn fetch_with_env_fallback<R: ConfigRepository>(repo: &R, key: &str) -> Result<ConfigEntity> {
    // First try to get from database
    if let Ok(config) = repo.fetch_by_key(key) {
        return Ok(config);
    }

    // Fall back to environment variables
    let env_value = match key {
        key if key == ConfigKeys::ClaudeApiKey.to_key() => EnvResolver::claude_api_key(),
        key if key == ConfigKeys::ChatGptApiKey.to_key() => EnvResolver::openai_api_key(),
        key if key == ConfigKeys::ProviderKey.to_key() => EnvResolver::provider(),
        _ => None,
    };

    if let Some(value) = env_value {
        // Create a temporary config entity (not persisted)
        Ok(ConfigEntity::new(key, &value))
    } else {
        Err(anyhow!(
            "Configuration key '{}' not found in database or environment",
            key
        ))
    }
}

/// Check if a configuration key exists in database or environment
#[allow(dead_code)]
pub fn has_config<R: ConfigRepository>(repo: &R, key: &str) -> bool {
    // Check database first
    if repo.fetch_by_key(key).is_ok() {
        return true;
    }

    // Check environment variables
    match key {
        key if key == ConfigKeys::ClaudeApiKey.to_key() => EnvResolver::claude_api_key().is_some(),
        key if key == ConfigKeys::ChatGptApiKey.to_key() => EnvResolver::openai_api_key().is_some(),
        key if key == ConfigKeys::ProviderKey.to_key() => EnvResolver::provider().is_some(),
        _ => false,
    }
}
