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
