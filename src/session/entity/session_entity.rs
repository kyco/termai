use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct SessionEntity {
    pub id: String,
    pub name: String,
    pub expires_at: NaiveDateTime,
    pub current: i32,
}

impl SessionEntity {
    pub fn new(id: String, name: String, expires_at: NaiveDateTime, current: i32) -> Self {
        Self {
            id,
            name,
            expires_at,
            current,
        }
    }
}
