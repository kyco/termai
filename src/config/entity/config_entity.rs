#[derive(Debug, Clone)]
pub struct ConfigEntity {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
}

impl ConfigEntity {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            id: None,
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    pub fn new_with_id(id: i64, key: &str, value: &str) -> Self {
        Self {
            id: Some(id),
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}
