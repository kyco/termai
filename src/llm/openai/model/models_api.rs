use serde::{Deserialize, Serialize};

/// Response from GET /v1/models
#[derive(Debug, Deserialize)]
pub struct ModelsListResponse {
    #[allow(dead_code)]
    pub object: String,
    pub data: Vec<ModelObject>,
}

/// Individual model from the OpenAI Models API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelObject {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub owned_by: String,
}

impl ModelObject {
    /// Check if this is a chat-capable model (not embedding, whisper, tts, etc.)
    pub fn is_chat_model(&self) -> bool {
        let id = &self.id;

        // Include GPT models, O-series reasoning models
        let is_chat_family = id.starts_with("gpt")
            || id.starts_with("o1")
            || id.starts_with("o3")
            || id.starts_with("o4")
            || id.starts_with("chatgpt");

        // Exclude non-chat models
        let is_excluded = id.contains("embedding")
            || id.contains("whisper")
            || id.contains("tts")
            || id.contains("dall-e")
            || id.contains("davinci")
            || id.contains("babbage")
            || id.contains("curie")
            || id.contains("ada")
            || id.contains("moderation")
            || id.contains("realtime")
            || id.contains("transcription")
            || id.contains("audio");

        is_chat_family && !is_excluded
    }
}

/// Filter a list of models to only chat-capable models
pub fn filter_chat_models(models: &[ModelObject]) -> Vec<ModelObject> {
    models.iter()
        .filter(|m| m.is_chat_model())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_models_response() {
        let json = r#"{
            "object": "list",
            "data": [
                {"id": "gpt-5.2", "object": "model", "created": 1686935002, "owned_by": "openai"},
                {"id": "gpt-4o", "object": "model", "created": 1686935002, "owned_by": "openai"}
            ]
        }"#;
        let response: ModelsListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.data[0].id, "gpt-5.2");
        assert_eq!(response.data[1].id, "gpt-4o");
        assert_eq!(response.object, "list");
    }

    #[test]
    fn test_filter_chat_models() {
        let models = vec![
            ModelObject {
                id: "gpt-5.2".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "text-embedding-3-small".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "whisper-1".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "gpt-4o".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "dall-e-3".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "tts-1".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "o1".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
            ModelObject {
                id: "o3".into(),
                object: "model".into(),
                created: 1686935002,
                owned_by: "openai".into(),
            },
        ];

        let chat_models = filter_chat_models(&models);
        assert_eq!(chat_models.len(), 4);

        let chat_model_ids: Vec<&str> = chat_models.iter().map(|m| m.id.as_str()).collect();
        assert!(chat_model_ids.contains(&"gpt-5.2"));
        assert!(chat_model_ids.contains(&"gpt-4o"));
        assert!(chat_model_ids.contains(&"o1"));
        assert!(chat_model_ids.contains(&"o3"));
        assert!(!chat_model_ids.contains(&"text-embedding-3-small"));
        assert!(!chat_model_ids.contains(&"whisper-1"));
        assert!(!chat_model_ids.contains(&"dall-e-3"));
        assert!(!chat_model_ids.contains(&"tts-1"));
    }

    #[test]
    fn test_is_chat_model() {
        // Chat models
        assert!(ModelObject { id: "gpt-5.2".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "gpt-4o".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "gpt-4o-mini".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "o1".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "o1-mini".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "o3".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "o4-mini".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(ModelObject { id: "chatgpt-4o-latest".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());

        // Non-chat models
        assert!(!ModelObject { id: "text-embedding-3-small".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "text-embedding-ada-002".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "whisper-1".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "tts-1".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "tts-1-hd".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "dall-e-3".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "dall-e-2".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "text-davinci-003".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "text-moderation-latest".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "gpt-4o-realtime-preview".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "gpt-4o-audio-preview".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
        assert!(!ModelObject { id: "gpt-4o-transcription".into(), object: "model".into(), created: 0, owned_by: "openai".into() }.is_chat_model());
    }

    #[test]
    fn test_model_object_serialization() {
        let model = ModelObject {
            id: "gpt-5.2".into(),
            object: "model".into(),
            created: 1686935002,
            owned_by: "openai".into(),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("gpt-5.2"));
        assert!(json.contains("openai"));

        let deserialized: ModelObject = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, model.id);
        assert_eq!(deserialized.owned_by, model.owned_by);
    }
}
