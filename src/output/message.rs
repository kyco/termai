use crate::openai::model::role::Role;

pub struct Message {
    pub role: Role,
    pub message: String,
}

impl Message {
    pub fn copy_with_message(&self, message: String) -> Self {
        Self {
            role: self.role.clone(),
            message,
        }
    }
}
