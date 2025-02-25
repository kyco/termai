use crate::args::Args;
use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use anyhow::Result;

pub fn write_provider_key<R: ConfigRepository>(repo: &R, provider: &Args) -> Result<()> {
    if let Some(ref provider) = provider.provider {
        config_service::write_config(repo, &ConfigKeys::ProviderKey.to_key(), provider.to_str())
    } else {
        Ok(())
    }
}
