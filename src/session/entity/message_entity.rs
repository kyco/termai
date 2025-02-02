pub struct MessageEntity {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
}

impl MessageEntity {
    pub fn new(id: String, session_id: String, role: String, content: String) -> Self {
        Self {
            id,
            session_id,
            role,
            content,
        }
    }
}
