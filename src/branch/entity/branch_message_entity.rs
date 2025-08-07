#![allow(dead_code)]
/// Database entity linking branches to messages in sequence
#[derive(Debug, Clone)]
pub struct BranchMessageEntity {
    pub id: String,
    pub branch_id: String,
    pub message_id: String,
    pub sequence_number: i32,
}

impl BranchMessageEntity {
    pub fn new(
        id: String,
        branch_id: String,
        message_id: String,
        sequence_number: i32,
    ) -> Self {
        Self {
            id,
            branch_id,
            message_id,
            sequence_number,
        }
    }
}