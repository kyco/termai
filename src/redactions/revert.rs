use std::collections::HashMap;
use regex::Regex;

pub fn unredact(mapped_redactions: &HashMap<String, String>, content: &str) -> String {
    mapped_redactions
        .iter()
        .fold(content.to_string(), |acc, (redaction, id)| {
            let re = Regex::new(&regex::escape(id)).unwrap();
            re.replace_all(&acc, redaction).to_string()
        })
}
