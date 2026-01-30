use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use crate::llm::openai::adapter::models_adapter::ModelsAdapter;
use crate::llm::openai::model::models_api::ModelObject;

const CACHE_TTL_HOURS: i64 = 24;

pub struct ModelsService;

impl ModelsService {
    /// Get models - from cache if fresh, otherwise fetch from API
    pub async fn get_models<R: ConfigRepository>(
        repo: &R,
        api_key: &str,
    ) -> Result<Vec<ModelObject>> {
        // Check cache first
        if let Some(cached) = Self::load_from_cache(repo)? {
            return Ok(cached);
        }

        // Fetch from API
        let models = ModelsAdapter::list_chat_models(api_key).await?;

        // Cache the results
        Self::save_to_cache(repo, &models)?;

        Ok(models)
    }

    /// Force refresh from API (ignores cache)
    pub async fn refresh_models<R: ConfigRepository>(
        repo: &R,
        api_key: &str,
    ) -> Result<Vec<ModelObject>> {
        let models = ModelsAdapter::list_chat_models(api_key).await?;
        Self::save_to_cache(repo, &models)?;
        Ok(models)
    }

    /// Load models from cache if not expired
    fn load_from_cache<R: ConfigRepository>(repo: &R) -> Result<Option<Vec<ModelObject>>> {
        // Load timestamp and check if expired
        let timestamp_result =
            config_service::fetch_by_key(repo, &ConfigKeys::OpenAIModelsCacheTimestamp.to_key());

        let timestamp = match timestamp_result {
            Ok(config) => config.value,
            Err(_) => return Ok(None), // No cache timestamp, cache is invalid
        };

        // Parse timestamp
        let cached_at = match DateTime::parse_from_rfc3339(&timestamp) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return Ok(None), // Invalid timestamp, cache is invalid
        };

        // Check if cache has expired
        let now = Utc::now();
        let expiry = cached_at + Duration::hours(CACHE_TTL_HOURS);
        if now > expiry {
            return Ok(None); // Cache expired
        }

        // Load cached models JSON
        let cache_result =
            config_service::fetch_by_key(repo, &ConfigKeys::OpenAIModelsCache.to_key());

        let cache_json = match cache_result {
            Ok(config) => config.value,
            Err(_) => return Ok(None), // No cache data
        };

        // Parse cached models
        match serde_json::from_str::<Vec<ModelObject>>(&cache_json) {
            Ok(models) => Ok(Some(models)),
            Err(_) => Ok(None), // Invalid cache data
        }
    }

    /// Save models to cache with current timestamp
    fn save_to_cache<R: ConfigRepository>(repo: &R, models: &[ModelObject]) -> Result<()> {
        // Serialize models to JSON
        let models_json = serde_json::to_string(models)?;

        // Save models cache
        config_service::write_config(
            repo,
            &ConfigKeys::OpenAIModelsCache.to_key(),
            &models_json,
        )?;

        // Save timestamp
        let now = Utc::now().to_rfc3339();
        config_service::write_config(
            repo,
            &ConfigKeys::OpenAIModelsCacheTimestamp.to_key(),
            &now,
        )?;

        Ok(())
    }

    /// Clear the models cache
    #[allow(dead_code)]
    pub fn clear_cache<R: ConfigRepository>(repo: &R) -> Result<()> {
        // Clear by writing empty values
        let _ = config_service::write_config(repo, &ConfigKeys::OpenAIModelsCache.to_key(), "");
        let _ = config_service::write_config(
            repo,
            &ConfigKeys::OpenAIModelsCacheTimestamp.to_key(),
            "",
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::entity::config_entity::ConfigEntity;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Test repository for unit tests
    struct TestRepository {
        configs: Mutex<HashMap<String, String>>,
    }

    impl TestRepository {
        fn new() -> Self {
            Self {
                configs: Mutex::new(HashMap::new()),
            }
        }
    }

    impl ConfigRepository for TestRepository {
        type Error = anyhow::Error;

        fn add_config(&self, key: &str, value: &str) -> anyhow::Result<()> {
            self.configs
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn update_config(&self, _id: i64, key: &str, value: &str) -> anyhow::Result<()> {
            self.configs
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }

        fn fetch_by_key(&self, key: &str) -> anyhow::Result<ConfigEntity> {
            let configs = self.configs.lock().unwrap();
            match configs.get(key) {
                Some(value) => Ok(ConfigEntity {
                    id: Some(1),
                    key: key.to_string(),
                    value: value.clone(),
                }),
                None => Err(anyhow::anyhow!("Key not found")),
            }
        }

        fn fetch_all_configs(&self) -> anyhow::Result<Vec<ConfigEntity>> {
            let configs = self.configs.lock().unwrap();
            Ok(configs
                .iter()
                .enumerate()
                .map(|(i, (k, v))| ConfigEntity {
                    id: Some(i as i64),
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect())
        }
    }

    #[test]
    fn test_cache_models_to_db() {
        let repo = TestRepository::new();
        let models = vec![ModelObject {
            id: "gpt-5.2".into(),
            object: "model".into(),
            created: 1686935002,
            owned_by: "openai".into(),
        }];

        // Save to cache
        ModelsService::save_to_cache(&repo, &models).unwrap();

        // Load from cache
        let cached = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached.is_some());
        let cached_models = cached.unwrap();
        assert_eq!(cached_models.len(), 1);
        assert_eq!(cached_models[0].id, "gpt-5.2");
    }

    #[test]
    fn test_cache_empty_when_no_data() {
        let repo = TestRepository::new();

        // Load from cache with no data
        let cached = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_expiry() {
        let repo = TestRepository::new();

        // Set an expired timestamp (25 hours ago)
        let expired_time = Utc::now() - Duration::hours(25);
        let models = vec![ModelObject {
            id: "gpt-5.2".into(),
            object: "model".into(),
            created: 1686935002,
            owned_by: "openai".into(),
        }];

        // Manually set cache with expired timestamp
        let models_json = serde_json::to_string(&models).unwrap();
        repo.add_config(&ConfigKeys::OpenAIModelsCache.to_key(), &models_json)
            .unwrap();
        repo.add_config(
            &ConfigKeys::OpenAIModelsCacheTimestamp.to_key(),
            &expired_time.to_rfc3339(),
        )
        .unwrap();

        // Load from cache - should return None due to expiry
        let cached = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_valid_within_ttl() {
        let repo = TestRepository::new();

        // Set a recent timestamp (1 hour ago)
        let recent_time = Utc::now() - Duration::hours(1);
        let models = vec![ModelObject {
            id: "gpt-5.2".into(),
            object: "model".into(),
            created: 1686935002,
            owned_by: "openai".into(),
        }];

        // Manually set cache with recent timestamp
        let models_json = serde_json::to_string(&models).unwrap();
        repo.add_config(&ConfigKeys::OpenAIModelsCache.to_key(), &models_json)
            .unwrap();
        repo.add_config(
            &ConfigKeys::OpenAIModelsCacheTimestamp.to_key(),
            &recent_time.to_rfc3339(),
        )
        .unwrap();

        // Load from cache - should return cached data
        let cached = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
    }

    #[test]
    fn test_clear_cache() {
        let repo = TestRepository::new();
        let models = vec![ModelObject {
            id: "gpt-5.2".into(),
            object: "model".into(),
            created: 1686935002,
            owned_by: "openai".into(),
        }];

        // Save to cache
        ModelsService::save_to_cache(&repo, &models).unwrap();

        // Verify cache exists
        let cached = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached.is_some());

        // Clear cache
        ModelsService::clear_cache(&repo).unwrap();

        // Verify cache is cleared (empty string is treated as no cache)
        let cached_after = ModelsService::load_from_cache(&repo).unwrap();
        assert!(cached_after.is_none());
    }
}
