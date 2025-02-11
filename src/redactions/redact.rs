use regex::Regex;
use std::collections::HashMap;

pub fn redact(content: &str, mapped_redactions: &HashMap<String, String>) -> String {
    let input_with_redactions =
        mapped_redactions
            .iter()
            .fold(content.to_string(), |acc, (redaction, id)| {
                let re = Regex::new(&format!("(?i){}", regex::escape(redaction))).unwrap();
                re.replace_all(&acc, id).to_string()
            });

    input_with_redactions.to_string()
}
