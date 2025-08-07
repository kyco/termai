use crate::llm::common::model::role::Role;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub message: String,
}
