use super::entity::config_entity::ConfigEntity;

pub(crate) mod config_repository;

pub trait ConfigRepository {
    type Error;

    fn fetch_all_configs(&self) -> Result<Vec<ConfigEntity>, Self::Error>;
    fn fetch_by_key(&self, key: &str) -> Result<ConfigEntity, Self::Error>;
    fn add_config(&self, key: &str, value: &str) -> Result<(), Self::Error>;
    fn update_config(&self, id: i64, key: &str, value: &str) -> Result<(), Self::Error>;
}
