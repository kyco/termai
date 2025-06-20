pub mod fixtures;
pub mod mocks;
pub mod test_db;

use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        
        Ok(TestEnvironment {
            temp_dir,
            db_path,
        })
    }
    
    pub fn db_path_str(&self) -> &str {
        self.db_path.to_str().unwrap()
    }
}