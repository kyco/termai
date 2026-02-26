use serde::{Serialize, Deserialize};

/// Controls the verbosity level for GPT-5 responses
/// Determines how many output tokens are generated
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Verbosity {
    /// Concise answers, minimal commentary
    Low,
    /// Balanced explanations (default)
    #[default]
    Medium,
    /// Thorough explanations and extensive details
    High,
}


#[allow(dead_code)]
impl Verbosity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Verbosity::Low),
            "medium" => Some(Verbosity::Medium),
            "high" => Some(Verbosity::High),
            _ => None,
        }
    }

    /// Returns a description of what this verbosity level does
    pub fn description(&self) -> &'static str {
        match self {
            Verbosity::Low => "Concise answers, minimal commentary",
            Verbosity::Medium => "Balanced explanations (default)",
            Verbosity::High => "Thorough explanations and extensive details",
        }
    }
}

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Verbosity::Low => "low",
            Verbosity::Medium => "medium",
            Verbosity::High => "high",
        };
        write!(f, "{}", s)
    }
}