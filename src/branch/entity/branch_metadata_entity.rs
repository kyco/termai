#![allow(dead_code)]
/// Database entity for storing arbitrary branch metadata
#[derive(Debug, Clone)]
pub struct BranchMetadataEntity {
    pub branch_id: String,
    pub key: String,
    pub value: String,
}

impl BranchMetadataEntity {
    pub fn new(branch_id: String, key: String, value: String) -> Self {
        Self {
            branch_id,
            key,
            value,
        }
    }
}