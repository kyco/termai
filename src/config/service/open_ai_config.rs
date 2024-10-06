use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use anyhow::Result;
use crate::args::Args;

pub fn write_open_ai_key<R: ConfigRepository>(repo: &R, chat_gpt_api_key: &Args) -> Result<()> {
    if let Some(ref chat_gpt_api_key) = chat_gpt_api_key.chat_gpt_api_key {
        config_service::write_config(
            repo,
            &ConfigKeys::ChatGptApiKey.to_key(),
            &chat_gpt_api_key,
        )
    } else {
        Ok(())
    }
}