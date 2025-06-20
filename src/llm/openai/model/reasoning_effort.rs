use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    #[allow(dead_code)]
    Low,
    #[allow(dead_code)]
    Medium,
    High,
}