use regex::Regex;
use std::collections::HashMap;

pub fn redact(content: &str, mapped_redactions: &HashMap<String, String>) -> String {
    mapped_redactions
        .iter()
        .fold(content.to_string(), |acc, (redaction, id)| {
            let pattern = regex::escape(redaction);
            let re = Regex::new(&pattern).unwrap();
            re.replace_all(&acc, id).to_string()
        })
}
