use termai::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use termai::llm::claude::model::chat_message::ChatMessage;
use termai::llm::claude::model::thinking::{Thinking};
use termai::llm::claude::model::thinking_type::ThinkingType;
use wiremock::matchers::{method, path, header};
use wiremock::{Mock, ResponseTemplate, MockServer};
use serde_json::json;

#[tokio::test]
async fn test_claude_chat_completion_success() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("content-type", "application/json"))
        .and(header("x-api-key", "test-api-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_test123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "Hello! This is a test response from Claude."
                }
            ],
            "model": "claude-opus-4-20250514",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 25
            }
        })))
        .mount(&mock_server)
        .await;
    
    // Create test request
    let _request = ChatCompletionRequest {
        model: "claude-opus-4-20250514".to_string(),
        max_tokens: 32000,
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, Claude!".to_string(),
            }
        ],
        system: Some("You are a helpful assistant.".to_string()),
        thinking: Some(Thinking {
            budget_tokens: 16000,
            thinking_type: ThinkingType::Enabled,
        }),
    };
    
    // Note: We would need to modify the adapter to accept a custom URL for testing
    // For now, this test shows the structure. In a real implementation, we'd need
    // dependency injection or configuration to point to the mock server.
    
    // This test demonstrates the structure but won't pass without URL injection
    // let (status, response) = claude_adapter::chat(&request, "test-api-key").await
    //     .expect("Failed to get Claude response");
    
    // assert_eq!(status, 200);
    // assert_eq!(response.model, "claude-opus-4-20250514");
    // assert_eq!(response.stop_reason, "end_turn");
    // assert!(!response.content.is_empty());
}

#[tokio::test]
async fn test_claude_chat_completion_error_handling() {
    let mock_server = MockServer::start().await;
    
    // Set up mock error response
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "type": "error",
            "error": {
                "type": "authentication_error",
                "message": "Invalid API key"
            }
        })))
        .mount(&mock_server)
        .await;
    
    let _request = ChatCompletionRequest {
        model: "claude-opus-4-20250514".to_string(),
        max_tokens: 32000,
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }
        ],
        system: None,
        thinking: None,
    };
    
    // This test demonstrates error handling structure
    // In a real implementation with URL injection, we would test actual error responses
    // let result = claude_adapter::chat(&request, "invalid-key").await;
    // assert!(result.is_err());
}

#[tokio::test]
async fn test_claude_api_timeout_handling() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response with delay to simulate timeout
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(30)) // Simulate long response
                .set_body_json(json!({
                    "id": "msg_delayed",
                    "type": "message",
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Delayed response"}],
                    "model": "claude-opus-4-20250514",
                    "stop_reason": "end_turn",
                    "usage": {"input_tokens": 10, "output_tokens": 25}
                }))
        )
        .mount(&mock_server)
        .await;
    
    // This test would verify timeout handling in the adapter
    // In practice, we'd need to configure the reqwest client with a shorter timeout
    // for testing purposes
}

#[tokio::test]
async fn test_claude_thinking_response_parsing() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response with thinking content
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_thinking123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "thinking",
                    "text": "I need to think about this carefully..."
                },
                {
                    "type": "text",
                    "text": "Based on my analysis, here's my response."
                }
            ],
            "model": "claude-opus-4-20250514",
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": 15,
                "output_tokens": 50
            }
        })))
        .mount(&mock_server)
        .await;
    
    // This test would verify that thinking content is properly handled
    // The current implementation filters out non-text content blocks
}

#[tokio::test]
async fn test_claude_refusal_handling() {
    let mock_server = MockServer::start().await;
    
    // Set up mock response with refusal stop reason
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "msg_refusal123",
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": "I can't help with that request as it may be harmful."
                }
            ],
            "model": "claude-opus-4-20250514",
            "stop_reason": "refusal",
            "usage": {
                "input_tokens": 20,
                "output_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;
    
    // This test would verify that refusal responses are properly handled
    // The chat service should detect stop_reason "refusal" and return an error
}

#[tokio::test]
async fn test_claude_adapter_request_formatting() {
    // Test that requests are properly formatted for Claude API
    let request = ChatCompletionRequest {
        model: "claude-opus-4-20250514".to_string(),
        max_tokens: 4096,
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Test message".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Previous response".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Follow-up question".to_string(),
            },
        ],
        system: Some("You are a helpful assistant that provides concise answers.".to_string()),
        thinking: Some(Thinking {
            budget_tokens: 2048,
            thinking_type: ThinkingType::Enabled,
        }),
    };
    
    // Verify request structure
    assert_eq!(request.model, "claude-opus-4-20250514");
    assert_eq!(request.max_tokens, 4096);
    assert_eq!(request.messages.len(), 3);
    assert!(request.system.is_some());
    assert!(request.thinking.is_some());
    
    // Verify message format
    assert_eq!(request.messages[0].role, "user");
    assert_eq!(request.messages[1].role, "assistant");
    assert_eq!(request.messages[2].role, "user");
    
    // Verify thinking configuration
    let thinking = request.thinking.unwrap();
    assert_eq!(thinking.budget_tokens, 2048);
    assert!(matches!(thinking.thinking_type, ThinkingType::Enabled));
}

#[tokio::test]
async fn test_claude_large_conversation_handling() {
    // Test handling of conversations with many messages
    let mut messages = Vec::new();
    
    // Create a conversation with 50 messages
    for i in 0..50 {
        let role = if i % 2 == 0 { "user" } else { "assistant" };
        messages.push(ChatMessage {
            role: role.to_string(),
            content: format!("Message {} content", i + 1),
        });
    }
    
    let request = ChatCompletionRequest {
        model: "claude-opus-4-20250514".to_string(),
        max_tokens: 32000,
        messages,
        system: Some("System message for large conversation".to_string()),
        thinking: Some(Thinking {
            budget_tokens: 16000,
            thinking_type: ThinkingType::Enabled,
        }),
    };
    
    // Verify the request can handle large conversations
    assert_eq!(request.messages.len(), 50);
    
    // In a real test with mock server, we'd verify that large requests
    // are handled properly and don't exceed token limits
}

#[tokio::test]
async fn test_claude_unicode_and_special_characters() {
    // Test handling of various character encodings and special characters
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello with émojis 🚀 and ünïcödé characters".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Special chars: @#$%^&*(){}[]|\\:;\"'<>,.?/~`".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "多言語テスト 中文测试 العربية русский".to_string(),
        },
    ];
    
    let request = ChatCompletionRequest {
        model: "claude-opus-4-20250514".to_string(),
        max_tokens: 4096,
        messages,
        system: Some("Handle unicode properly 🌍".to_string()),
        thinking: None,
    };
    
    // Verify unicode content is preserved
    assert!(request.messages[0].content.contains("émojis 🚀"));
    assert!(request.messages[1].content.contains("@#$%^&*()"));
    assert!(request.messages[2].content.contains("多言語テスト"));
    assert!(request.system.unwrap().contains("🌍"));
}