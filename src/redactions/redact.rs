use std::collections::HashMap;
use regex::Regex;
use crate::config::repository::ConfigRepository;
use crate::config::service::redacted_config;

use super::common;

pub fn redact<R: ConfigRepository>(repo: &R, content: &str) -> (String, HashMap<String, String>) {
    let redactions = redacted_config::fetch_redactions(repo);
    let mapped_redactions = common::redaction_map(redactions);

    let input_with_redactions =
        mapped_redactions
            .iter()
            .fold(content.to_string(), |acc, (redaction, id)| {
                let re = Regex::new(&format!("(?i){}", regex::escape(redaction))).unwrap();
                re.replace_all(&acc, id).to_string()
            });

    (input_with_redactions.to_string(), mapped_redactions)
}
