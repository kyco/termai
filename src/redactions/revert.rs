use regex::Regex;
use std::collections::HashMap;

pub fn unredact(mapped_redactions: &HashMap<String, String>, content: &str) -> String {
    mapped_redactions
        .iter()
        .fold(content.to_string(), |acc, (redaction, id)| {
            let re = Regex::new(&regex::escape(id)).unwrap();
            re.replace_all(&acc, redaction).to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Test that when there are no redaction mappings the content remains unchanged.
    #[test]
    fn test_unredact_returns_unmodified_content_when_no_redactions() {
        // Arrange
        let content = "This content is safe and unmodified.";
        let redactions = HashMap::new();

        // Act
        let result = unredact(&redactions, content);

        // Assert
        assert_eq!(result, content);
    }

    // Test that an empty content returns an empty string even if a mapping is provided.
    #[test]
    fn test_unredact_returns_empty_string_when_input_is_empty() {
        // Arrange
        let content = "";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());

        // Act
        let result = unredact(&redactions, content);

        // Assert
        assert_eq!(result, "");
    }

    // Test that a single occurrence of a redaction id is correctly reverted.
    #[test]
    fn test_unredact_replaces_single_occurrence() {
        // Arrange
        let content = "My [REDACTED] is secure.";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());

        // Act
        let result = unredact(&redactions, content);

        // Assert
        let expected = "My secret is secure.";
        assert_eq!(result, expected);
    }

    // Test that multiple occurrences of a given redaction id are all reverted.
    #[test]
    fn test_unredact_replaces_multiple_occurrences() {
        // Arrange
        let content = "[REDACTED] was mentioned, then [REDACTED] appeared again.";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[REDACTED]".to_string());

        // Act
        let result = unredact(&redactions, content);

        // Assert
        let expected = "secret was mentioned, then secret appeared again.";
        assert_eq!(result, expected);
    }

    // Test that multiple different redactions are correctly reverted.
    #[test]
    fn test_unredact_replaces_multiple_different_redactions() {
        // Arrange
        let content = "Access [SECRET] and [CLASSIFIED] data.";
        let mut redactions = HashMap::new();
        redactions.insert("secret".to_string(), "[SECRET]".to_string());
        redactions.insert("classified".to_string(), "[CLASSIFIED]".to_string());

        // Act
        let result = unredact(&redactions, content);

        // Assert
        let expected = "Access secret and classified data.";
        assert_eq!(result, expected);
    }

    // Test that redaction ids containing regex-special characters are correctly escaped and reverted.
    #[test]
    fn test_unredact_handles_regex_special_characters_in_id() {
        // Arrange
        // The redaction key contains regex special characters.
        let content = "Special pattern: [SPECI(AL)] is found and must be reverted.";
        let mut redactions = HashMap::new();
        redactions.insert("a+b*?".to_string(), "[SPECI(AL)]".to_string());

        // Act
        let result = unredact(&redactions, content);

        // Assert
        let expected = "Special pattern: a+b*? is found and must be reverted.";
        assert_eq!(result, expected);
    }
}
