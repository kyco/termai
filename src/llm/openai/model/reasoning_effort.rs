use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}