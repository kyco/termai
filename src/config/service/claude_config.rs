use crate::args::Args;
use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use anyhow::Result;

pub fn write_claude_key<R: ConfigRepository>(repo: &R, claude_api_key: &Args) -> Result<()> {
    if let Some(ref claude_api_key) = claude_api_key.claude_api_key {
        config_service::write_config(repo, &ConfigKeys::ClaudeApiKey.to_key(), claude_api_key)
    } else {
        Ok(())
    }
}
