use crate::openai::model::role::Role;

pub struct Message {
    pub role: Role,
    pub message: String,
}
