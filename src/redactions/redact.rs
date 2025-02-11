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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test when there are no keys in the redactions map.
    #[test]
    fn test_redact_returns_unmodified_content_when_no_redactions() {
        // arrange
        let content = "There is no sensitive info here.";
        let redactions = HashMap::new();

        // act
        let result = redact(content, &redactions);

        // assert
        assert_eq!(result, content);
    }

    // Test when the input content is an empty string.
    #[test]
    fn test_redact_returns_empty_string_when_input_is_empty() {
        // arrange
        let content = "";
        let mut redactions = HashMap::new();
        redactions.insert("dummy".to_string(), "[REDACTED]".to_string());

        // act
        let result = redact(content, &redactions);

        // assert
        assert_eq!(result, "");
    }

    // Test that a single occurrence of a keyword is correctly replaced.
    #[test]
    fn test_redact_replaces_single_occurrence() {
        // arrange
        let content = "This password is secret.";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());

        // act
        let result = redact(content, &redactions);

        // assert
        let expected = "This password is [REDACTED].";
        assert_eq!(result, expected);
    }

    // Test that multiple occurrences of a redaction occur are all replaced.
    #[test]
    fn test_redact_replaces_multiple_occurrences() {
        // arrange
        let content = "secret secret, always keep your secret safe";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());

        // act
        let result = redact(content, &redactions);

        // assert
        let expected = "[REDACTED] [REDACTED], always keep your [REDACTED] safe";
        assert_eq!(result, expected);
    }

    // Test that multiple different redactions are replaced appropriately.
    #[test]
    fn test_redact_replaces_multiple_different_redactions() {
        // arrange
        let content = "The secret and classified info are protected.";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());
        redactions.insert("classified".to_string(), "[REDACTED]".to_string());

        // act
        let result = redact(content, &redactions);

        // assert
        let expected = "The [REDACTED] and [REDACTED] info are protected.";
        assert_eq!(result, expected);
    }

    // Test that redaction keys containing regex-special characters are correctly escaped.
    #[test]
    fn test_redact_handles_regex_special_characters_in_redaction_key() {
        // arrange
        let content = "Replace a+b*? in this text: a+b*? needs to be censored.";
        let mut redactions = HashMap::new();
        redactions.insert("a+b*?".to_string(), "[SPECIAL]".to_string());

        // act
        let result = redact(content, &redactions);

        // assert
        let expected = "Replace [SPECIAL] in this text: [SPECIAL] needs to be censored.";
        assert_eq!(result, expected);
    }
}
