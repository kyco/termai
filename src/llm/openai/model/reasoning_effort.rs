use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ReasoningEffort {
    None,
    Low,
    Medium,
    High,
    XHigh,
    /// Highest single-model effort (GPT-5.6 Sol only)
    Max,
    /// Multi-agent mode, ~4 subagents (GPT-5.6 Sol only)
    Ultra,
}

#[allow(dead_code)]
impl ReasoningEffort {
    /// All supported reasoning effort values, lowest to highest.
    pub fn all() -> &'static [ReasoningEffort] {
        &[
            ReasoningEffort::None,
            ReasoningEffort::Low,
            ReasoningEffort::Medium,
            ReasoningEffort::High,
            ReasoningEffort::XHigh,
            ReasoningEffort::Max,
            ReasoningEffort::Ultra,
        ]
    }

    /// Whether this effort level is only supported by GPT-5.6 Sol.
    pub fn requires_sol(&self) -> bool {
        matches!(self, ReasoningEffort::Max | ReasoningEffort::Ultra)
    }
}

/// Model ids that support the Sol-only `max`/`ultra` reasoning efforts.
fn model_supports_sol_effort(model: &str) -> bool {
    matches!(model, "gpt-5.6-sol" | "gpt-5.6")
}

/// Returns a warning when a Sol-only effort (`max`/`ultra`) is requested for
/// a model other than `gpt-5.6-sol` (or the bare `gpt-5.6` alias, which
/// routes to Sol). The combination is allowed, not blocked; the API itself
/// may reject it.
pub fn sol_only_effort_warning(model: &str, effort: &ReasoningEffort) -> Option<String> {
    if effort.requires_sol() && !model_supports_sol_effort(model) {
        Some(format!(
            "{} effort requires gpt-5.6-sol; the API may reject this for model '{}'",
            effort, model
        ))
    } else {
        None
    }
}

impl std::fmt::Display for ReasoningEffort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasoningEffort::None => write!(f, "none"),
            ReasoningEffort::Low => write!(f, "low"),
            ReasoningEffort::Medium => write!(f, "medium"),
            ReasoningEffort::High => write!(f, "high"),
            ReasoningEffort::XHigh => write!(f, "xhigh"),
            ReasoningEffort::Max => write!(f, "max"),
            ReasoningEffort::Ultra => write!(f, "ultra"),
        }
    }
}

impl std::str::FromStr for ReasoningEffort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(ReasoningEffort::None),
            "low" => Ok(ReasoningEffort::Low),
            "medium" => Ok(ReasoningEffort::Medium),
            "high" => Ok(ReasoningEffort::High),
            "xhigh" => Ok(ReasoningEffort::XHigh),
            "max" => Ok(ReasoningEffort::Max),
            "ultra" => Ok(ReasoningEffort::Ultra),
            other => Err(format!(
                "invalid reasoning effort '{}' (expected one of: none, low, medium, high, xhigh, max, ultra)",
                other
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_serde_round_trip_all_variants() {
        for effort in ReasoningEffort::all() {
            let json = serde_json::to_string(effort).unwrap();
            let parsed: ReasoningEffort = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, effort);
        }
    }

    #[test]
    fn test_max_and_ultra_serialize_lowercase() {
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::Max).unwrap(),
            "\"max\""
        );
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::Ultra).unwrap(),
            "\"ultra\""
        );
    }

    #[test]
    fn test_from_str_parses_every_display_value() {
        for effort in ReasoningEffort::all() {
            let parsed = ReasoningEffort::from_str(&effort.to_string()).unwrap();
            assert_eq!(&parsed, effort);
        }
    }

    #[test]
    fn test_from_str_rejects_unknown_value() {
        assert!(ReasoningEffort::from_str("mega").is_err());
    }

    #[test]
    fn test_max_and_ultra_are_sol_only() {
        assert!(ReasoningEffort::Max.requires_sol());
        assert!(ReasoningEffort::Ultra.requires_sol());
        assert!(!ReasoningEffort::XHigh.requires_sol());
        assert!(!ReasoningEffort::High.requires_sol());
    }

    #[test]
    fn test_sol_only_effort_warning_for_non_sol_model() {
        let warning = sol_only_effort_warning("gpt-5.6-terra", &ReasoningEffort::Ultra);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("requires gpt-5.6-sol"));

        let warning = sol_only_effort_warning("gpt-5.4", &ReasoningEffort::Max);
        assert!(warning.is_some());
    }

    #[test]
    fn test_sol_only_effort_warning_absent_for_sol_and_alias() {
        assert!(sol_only_effort_warning("gpt-5.6-sol", &ReasoningEffort::Ultra).is_none());
        assert!(sol_only_effort_warning("gpt-5.6", &ReasoningEffort::Max).is_none());
        assert!(sol_only_effort_warning("gpt-5.6-luna", &ReasoningEffort::High).is_none());
    }
}
