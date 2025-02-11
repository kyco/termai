use std::collections::HashMap;
use uuid::Uuid;

pub fn redaction_map(redactions: Vec<String>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for redaction in redactions {
        map.insert(redaction, generate_uuid_v4());
    }
    map
}

fn generate_uuid_v4() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // Test that an empty vector produces an empty map.
    #[test]
    fn test_redaction_map_empty() {
        // Arrange
        let input: Vec<String> = Vec::new();

        // Act
        let result = redaction_map(input);

        // Assert
        assert!(result.is_empty(), "Expected an empty map when provided an empty vector.");
    }

    // Test that each redaction key is assigned a valid UUID.
    #[test]
    fn test_redaction_map_valid_uuid_values() {
        // Arrange: vector with several redaction keys.
        let keys = vec!["ssn".to_string(), "credit_card".to_string(), "email".to_string()];

        // Act
        let result = redaction_map(keys.clone());

        // Assert:
        // 1. The map should contain one entry per unique key.
        assert_eq!(
            result.len(),
            keys.len(),
            "Expected {} entries in the map, but got {}.",
            keys.len(),
            result.len()
        );

        // 2. Each value must be parsed as a valid UUID.
        for key in keys {
            let uuid_value = result.get(&key).expect("Key should exist in the map.");
            assert!(
                uuid::Uuid::parse_str(uuid_value).is_ok(),
                "Value '{}' for key '{}' is not a valid UUID.",
                uuid_value,
                key
            );
        }
    }

    // Test that duplicate keys in the vector produce only one entry in the map.
    #[test]
    fn test_redaction_map_with_duplicates() {
        // Arrange: vector with duplicate entries.
        let input = vec![
            "duplicate".to_string(),
            "unique".to_string(),
            "duplicate".to_string(),
        ];

        // Act
        let result = redaction_map(input);

        // Assert: There should be only two unique keys.
        assert_eq!(
            result.len(),
            2,
            "Expected 2 unique keys in the map when duplicates are provided."
        );
        // Verify each stored value is a valid UUID.
        for key in ["duplicate", "unique"].iter() {
            let uuid_value = result
                .get(&key.to_string())
                .expect(&format!("Key '{}' should be present in the map.", key));
            assert!(
                uuid::Uuid::parse_str(uuid_value).is_ok(),
                "Value for key '{}' is not a valid UUID.",
                key
            );
        }
    }

    // Test that for multiple unique entries each assigned UUID is unique.
    #[test]
    fn test_redaction_map_unique_entries_have_unique_uuids() {
        // Arrange
        let input = vec!["A".to_string(), "B".to_string(), "C".to_string()];

        // Act
        let result = redaction_map(input.clone());

        // Assert:
        // Collect all generated UUIDs into a vector.
        let uuid_values: Vec<&String> = result.values().collect();
        // Convert the vector into a HashSet to remove duplicates.
        let unique_uuids: HashSet<&&String> = uuid_values.iter().collect();

        assert_eq!(
            uuid_values.len(),
            unique_uuids.len(),
            "Expected each unique key to have a unique UUID."
        );
    }
}
