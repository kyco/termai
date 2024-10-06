use crate::config::entity::config_entity::ConfigEntity;
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
