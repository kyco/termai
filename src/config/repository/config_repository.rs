use crate::config::{entity::config_entity::ConfigEntity, repository::ConfigRepository};
use crate::repository::db::SqliteRepository;
use rusqlite::{params, Result};

impl ConfigRepository for SqliteRepository {
    type Error = rusqlite::Error;

    fn fetch_all_configs(&self) -> Result<Vec<ConfigEntity>, Self::Error> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM config")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok(ConfigEntity::new(&key, &value))
        })?;

        let mut configs = Vec::new();
        for config in rows {
            configs.push(config?);
        }
        Ok(configs)
    }

    fn fetch_by_key(&self, key: &str) -> Result<ConfigEntity, Self::Error> {
        let config = self.conn.query_row(
            "SELECT id, key, value FROM config WHERE key = ?1",
            params![key],
            |row| {
                let id: i64 = row.get(0)?;
                let key: String = row.get(1)?;
                let value: String = row.get(2)?;

                Ok(ConfigEntity::new_with_id(id, &key, &value))
            },
        )?;

        Ok(config)
    }

    fn add_config(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        self.conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn update_config(&self, id: i64, key: &str, value: &str) -> Result<(), Self::Error> {
        self.conn.execute(
            "UPDATE config SET key = ?1, value = ?2 WHERE id = ?3",
            params![key, value, id],
        )?;
        Ok(())
    }
}
