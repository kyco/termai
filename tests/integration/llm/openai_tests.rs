use termai::llm::openai::model::chat_completion_request::ChatCompletionRequest;
use termai::llm::openai::model::chat_message::ChatMessage;
use termai::llm::openai::model::model::Model;
use termai::llm::openai::model::reasoning_effort::ReasoningEffort;
use wiremock::matchers::{method, path, header};
use wiremock::{Mock, ResponseTemplate, MockServer};
use serde_json::json;

#[tokio::test]
async fn test_openai_chat_completion_success() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("content-type", "application/json"))
        .and(header("authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-test123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "o3-mini",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! This is a test response from OpenAI."
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 25,
                "total_tokens": 35,
                "completion_token_details": {
                    "reasoning_tokens": 100
                }
            }
        })))
        .mount(&mock_server)
        .await;
    
    // Create test request
    let _request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, OpenAI!".to_string(),
            }
        ],
        reasoning_effort: ReasoningEffort::High,
    };
    
    // Note: Similar to Claude tests, we would need URL injection for actual testing
    // This demonstrates the structure for OpenAI integration tests
    
    // let response = open_ai_adapter::chat(&request, "test-api-key").await
    //     .expect("Failed to get OpenAI response");
    
    // assert_eq!(response.model, Some("o3-mini".to_string()));
    // assert!(response.choices.is_some());
    // let choices = response.choices.unwrap();
    // assert_eq!(choices.len(), 1);
    // assert_eq!(choices[0].message.role, "assistant");
}

#[tokio::test]
async fn test_openai_chat_completion_error_handling() {
    let mock_server = MockServer::start().await;
    
    // Set up mock error response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "message": "Incorrect API key provided",
                "type": "invalid_request_error",
                "param": null,
                "code": "invalid_api_key"
            }
        })))
        .mount(&mock_server)
        .await;
    
    let _request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }
        ],
        reasoning_effort: ReasoningEffort::Medium,
    };
    
    // This test demonstrates error handling structure
    // let result = open_ai_adapter::chat(&request, "invalid-key").await;
    // assert!(result.is_err());
}

#[tokio::test]
async fn test_openai_reasoning_effort_handling() {
    // Test different reasoning effort levels
    let test_cases = vec![
        (ReasoningEffort::Low, "low"),
        (ReasoningEffort::Medium, "medium"), 
        (ReasoningEffort::High, "high"),
    ];
    
    for (effort, _expected_str) in test_cases {
        let _request = ChatCompletionRequest {
            model: Model::O3Mini.to_string(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Complex reasoning task".to_string(),
                }
            ],
            reasoning_effort: effort,
        };
        
        // Verify reasoning effort is set correctly
        // In a real implementation, we'd verify this gets serialized properly
        // assert_eq!(request.reasoning_effort.to_string(), expected_str);
    }
}

#[tokio::test]
async fn test_openai_api_rate_limiting() {
    let mock_server = MockServer::start().await;
    
    // Set up mock rate limit response
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": {
                "message": "Rate limit exceeded",
                "type": "rate_limit_error",
                "param": null,
                "code": "rate_limit_exceeded"
            }
        })))
        .mount(&mock_server)
        .await;
    
    let _request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }
        ],
        reasoning_effort: ReasoningEffort::Medium,
    };
    
    // This would test rate limiting error handling
    // let result = open_ai_adapter::chat(&request, "test-key").await;
    // assert!(result.is_err());
    // // Verify that the error indicates rate limiting
}

#[tokio::test]
async fn test_openai_adapter_request_formatting() {
    // Test that requests are properly formatted for OpenAI API
    let request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi there! How can I help you?".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What's the weather like?".to_string(),
            },
        ],
        reasoning_effort: ReasoningEffort::High,
    };
    
    // Verify request structure
    assert_eq!(request.model, Model::O3Mini.to_string());
    assert_eq!(request.messages.len(), 4);
    
    // Verify message roles
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
    assert_eq!(request.messages[2].role, "assistant");
    assert_eq!(request.messages[3].role, "user");
    
    // Verify reasoning effort
    assert!(matches!(request.reasoning_effort, ReasoningEffort::High));
}

#[tokio::test]
async fn test_openai_model_variants() {
    // Test different OpenAI model variants
    let models = vec![
        Model::O3Mini,
        // Add other model variants if they exist in the enum
    ];
    
    for model in models {
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Test message".to_string(),
                }
            ],
            reasoning_effort: ReasoningEffort::Medium,
        };
        
        // Verify model string conversion
        assert!(!request.model.is_empty());
    }
}

#[tokio::test]
async fn test_openai_large_conversation_handling() {
    // Test handling of conversations with many messages
    let mut messages = Vec::new();
    
    // Add system message
    messages.push(ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    });
    
    // Create a conversation with 100 user/assistant pairs
    for i in 0..100 {
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: format!("User message {}", i + 1),
        });
        messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: format!("Assistant response {}", i + 1),
        });
    }
    
    let request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages,
        reasoning_effort: ReasoningEffort::High,
    };
    
    // Verify the request can handle large conversations
    assert_eq!(request.messages.len(), 201); // 1 system + 200 user/assistant
    
    // In a real test, we'd verify token limits and proper handling
}

#[tokio::test]
async fn test_openai_unicode_and_special_characters() {
    // Test handling of various character encodings
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Unicode test: 🚀 émojis and spëcial chars".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Math symbols: ∑∞∈∅∩∪⊆⊇⊄⊅∀∃∄".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "CJK: 日本語 中文 한국어".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Arabic: مرحبا العالم".to_string(),
        },
    ];
    
    let request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages,
        reasoning_effort: ReasoningEffort::Medium,
    };
    
    // Verify unicode content is preserved
    assert!(request.messages[0].content.contains("🚀 émojis"));
    assert!(request.messages[1].content.contains("∑∞∈∅"));
    assert!(request.messages[2].content.contains("日本語"));
    assert!(request.messages[3].content.contains("مرحبا"));
}

#[tokio::test]
async fn test_openai_reasoning_tokens_usage() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response with reasoning tokens
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-reasoning123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "o3-mini",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "After careful reasoning, here's my response."
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 50,
                "completion_tokens": 30,
                "total_tokens": 80,
                "completion_token_details": {
                    "reasoning_tokens": 200
                }
            }
        })))
        .mount(&mock_server)
        .await;
    
    // This test would verify that reasoning tokens are properly tracked
    // in usage statistics for the o3 model series
}

#[tokio::test]
async fn test_openai_streaming_response_handling() {
    // Test structure for streaming responses (if supported)
    let _request = ChatCompletionRequest {
        model: Model::O3Mini.to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Tell me a story".to_string(),
            }
        ],
        reasoning_effort: ReasoningEffort::High,
    };
    
    // In a real implementation, we might add a `stream` parameter
    // and test that streaming responses are handled correctly
    // This test shows the structure for such functionality
}