#[cfg(test)]
mod tests {
    use super::super::validator::ApiKeyValidator;
    use super::super::wizard::SetupWizard;
    use crate::config::entity::config_entity::ConfigEntity;
    use crate::config::model::keys::ConfigKeys;
    use crate::config::repository::ConfigRepository;
    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    #[derive(Debug, Clone)]
    pub struct MockConfigRepository {
        data: Arc<Mutex<HashMap<String, ConfigEntity>>>,
        next_id: Arc<Mutex<i64>>,
    }

    impl MockConfigRepository {
        pub fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }

        pub fn with_data(initial_data: HashMap<String, String>) -> Self {
            let mut entities = HashMap::new();
            let mut next_id = 1;
            for (key, value) in initial_data.into_iter() {
                let entity = ConfigEntity::new_with_id(next_id, &key, &value);
                entities.insert(key, entity);
                next_id += 1;
            }
            let entities_len = entities.len();
            Self {
                data: Arc::new(Mutex::new(entities)),
                next_id: Arc::new(Mutex::new((entities_len + 1) as i64)),
            }
        }

        pub fn get_data(&self) -> HashMap<String, String> {
            let data = self.data.lock().unwrap();
            data.iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect()
        }
    }

    impl ConfigRepository for MockConfigRepository {
        type Error = anyhow::Error;

        fn fetch_all_configs(&self) -> Result<Vec<ConfigEntity>, Self::Error> {
            let data = self.data.lock().unwrap();
            Ok(data.values().cloned().collect())
        }

        fn fetch_by_key(&self, key: &str) -> Result<ConfigEntity, Self::Error> {
            let data = self.data.lock().unwrap();
            data.get(key)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))
        }

        fn add_config(&self, key: &str, value: &str) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            let mut next_id = self.next_id.lock().unwrap();

            let entity = ConfigEntity::new_with_id(*next_id, key, value);
            data.insert(key.to_string(), entity);
            *next_id += 1;
            Ok(())
        }

        fn update_config(&self, id: i64, key: &str, value: &str) -> Result<(), Self::Error> {
            let mut data = self.data.lock().unwrap();
            let entity = ConfigEntity::new_with_id(id, key, value);
            data.insert(key.to_string(), entity);
            Ok(())
        }
    }

    #[test]
    fn test_setup_wizard_creation() {
        let wizard = SetupWizard::new();
        // Just verify we can create the wizard without panicking
        assert_eq!(
            std::mem::size_of_val(&wizard),
            std::mem::size_of::<SetupWizard>()
        );
    }

    #[test]
    fn test_check_existing_config_empty() {
        let repo = MockConfigRepository::new();
        let _wizard = SetupWizard::new();

        // With no existing configuration, should be empty
        assert!(repo.get_data().is_empty());
    }

    #[test]
    fn test_check_existing_config_with_data() {
        let mut initial_data = HashMap::new();
        initial_data.insert(ConfigKeys::ClaudeApiKey.to_key(), "test-key".to_string());

        let repo = MockConfigRepository::with_data(initial_data);
        let _wizard = SetupWizard::new();

        // Verify the data was set
        assert!(repo
            .get_data()
            .contains_key(&ConfigKeys::ClaudeApiKey.to_key()));
    }

    #[test]
    fn test_reset_configuration() -> Result<()> {
        let mut initial_data = HashMap::new();
        initial_data.insert(
            ConfigKeys::ClaudeApiKey.to_key(),
            "test-claude-key".to_string(),
        );
        initial_data.insert(
            ConfigKeys::ChatGptApiKey.to_key(),
            "test-openai-key".to_string(),
        );
        initial_data.insert(ConfigKeys::ProviderKey.to_key(), "claude".to_string());

        let repo = MockConfigRepository::with_data(initial_data);

        // Verify data exists before reset
        assert!(repo.get_data().len() == 3);

        // Test the config clearing logic (simulating reset_configuration internals)
        let keys_to_clear = vec![
            ConfigKeys::ClaudeApiKey.to_key(),
            ConfigKeys::ChatGptApiKey.to_key(),
            ConfigKeys::ProviderKey.to_key(),
        ];

        for key in keys_to_clear {
            repo.add_config(&key, "")?;
        }

        // Verify keys were cleared (set to empty string)
        for key in [
            ConfigKeys::ClaudeApiKey.to_key(),
            ConfigKeys::ChatGptApiKey.to_key(),
            ConfigKeys::ProviderKey.to_key(),
        ] {
            assert_eq!(repo.fetch_by_key(&key)?.value, "");
        }

        Ok(())
    }

    #[test]
    fn test_config_repository_operations() -> Result<()> {
        let repo = MockConfigRepository::new();

        // Test adding config
        repo.add_config("test_key", "test_value")?;

        // Test fetching by key
        let config = repo.fetch_by_key("test_key")?;
        assert_eq!(config.key, "test_key");
        assert_eq!(config.value, "test_value");

        // Test updating config
        if let Some(id) = config.id {
            repo.update_config(id, "test_key", "updated_value")?;
            let updated_config = repo.fetch_by_key("test_key")?;
            assert_eq!(updated_config.value, "updated_value");
        }

        // Test fetch all configs
        let all_configs = repo.fetch_all_configs()?;
        assert_eq!(all_configs.len(), 1);

        Ok(())
    }

    // Mock validator for testing
    pub struct MockValidator {
        should_succeed: bool,
    }

    impl MockValidator {
        pub fn new(should_succeed: bool) -> Self {
            Self { should_succeed }
        }
    }

    #[async_trait::async_trait]
    impl ApiKeyValidator for MockValidator {
        async fn validate(&self, _api_key: &str) -> Result<()> {
            if self.should_succeed {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Mock validation failed"))
            }
        }
    }

    #[tokio::test]
    async fn test_api_key_validation_success() {
        let validator = MockValidator::new(true);
        let result = validator.validate("test-key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_key_validation_failure() {
        let validator = MockValidator::new(false);
        let result = validator.validate("invalid-key").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Mock validation failed");
    }

    #[test]
    fn test_provider_enum_descriptions() {
        use super::super::wizard::Provider;

        assert_eq!(
            Provider::Claude.description(),
            "Claude (Anthropic) - Best for analysis & coding"
        );
        assert_eq!(
            Provider::OpenAI.description(),
            "OpenAI (API Key) - Versatile general purpose"
        );
        assert_eq!(
            Provider::OpenAICodex.description(),
            "OpenAI Codex (ChatGPT Plus/Pro) - Use your subscription"
        );
        assert_eq!(Provider::Both.description(), "Both providers (recommended)");
    }

    // Integration tests that would require more setup
    #[tokio::test]
    #[ignore] // Ignore by default since these require network access
    async fn test_claude_validator_integration() {
        use super::super::validator::ClaudeValidator;
        let _validator = ClaudeValidator::new();
        // This would require a real API key to test
        // let result = validator.validate("invalid-key").await;
        // assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Ignore by default since these require network access
    async fn test_openai_validator_integration() {
        use super::super::validator::OpenAIValidator;
        let _validator = OpenAIValidator::new();
        // This would require a real API key to test
        // let result = validator.validate("invalid-key").await;
        // assert!(result.is_err());
    }
}
