# Task: Implement Comprehensive LLM Integration Tests

## Priority: Critical
## Estimated Effort: 5-7 days
## Dependencies: None

## Overview
Implement comprehensive integration tests for LLM providers (OpenAI and Claude) to ensure reliability of the core AI interaction functionality. This addresses the current 0% test coverage for LLM integration.

## Requirements

### Functional Requirements
1. **Mock Infrastructure**
   - HTTP request/response mocking with wiremock
   - Realistic API response simulation
   - Error scenario simulation
   - Rate limiting simulation

2. **Test Scenarios**
   - Successful chat completions
   - API authentication failures
   - Network timeouts
   - Rate limit handling
   - Token limit exceeded
   - Malformed responses
   - Streaming vs non-streaming responses

3. **Provider-Specific Tests**
   - OpenAI-specific endpoints and formats
   - Claude-specific endpoints and formats
   - Provider switching logic
   - Fallback behavior

### Technical Requirements
1. **Test Structure**
   ```rust
   // tests/integration/llm/mod.rs
   mod openai_tests;
   mod claude_tests;
   mod provider_tests;
   mod mock_helpers;
   ```

2. **Mock Helpers**
   ```rust
   // tests/integration/llm/mock_helpers.rs
   pub struct LLMMockServer {
       server: MockServer,
   }
   
   impl LLMMockServer {
       pub async fn new() -> Self;
       pub fn mock_successful_completion(&self, response: &str) -> &Self;
       pub fn mock_auth_failure(&self) -> &Self;
       pub fn mock_rate_limit(&self) -> &Self;
       pub fn mock_timeout(&self) -> &Self;
   }
   ```

3. **Dependency Injection**
   - Refactor LLM adapters to accept HTTP client
   - Create testable interfaces
   - Configuration for test endpoints

## Implementation Steps

1. **Refactor for Testability**
   ```rust
   // In adapters/llm/openai_adapter.rs
   pub struct OpenAIAdapter {
       client: Arc<dyn HttpClient>, // Injectable
       config: OpenAIConfig,
   }
   
   #[cfg(test)]
   impl OpenAIAdapter {
       pub fn with_client(client: Arc<dyn HttpClient>, config: OpenAIConfig) -> Self {
           Self { client, config }
       }
   }
   ```

2. **Core Integration Tests**
   ```rust
   // tests/integration/llm/openai_tests.rs
   #[tokio::test]
   async fn test_successful_completion() {
       let mock_server = LLMMockServer::new().await;
       mock_server.mock_successful_completion("Test response");
       
       let adapter = OpenAIAdapter::with_client(
           mock_client(&mock_server.url()),
           test_config(),
       );
       
       let response = adapter.complete("Test prompt").await;
       assert!(response.is_ok());
       assert_eq!(response.unwrap().content, "Test response");
   }
   
   #[tokio::test]
   async fn test_rate_limit_retry() {
       let mock_server = LLMMockServer::new().await;
       mock_server
           .mock_rate_limit()
           .mock_successful_completion("Success after retry");
       
       let adapter = OpenAIAdapter::with_client(
           mock_client(&mock_server.url()),
           test_config(),
       );
       
       let response = adapter.complete("Test prompt").await;
       assert!(response.is_ok());
       // Verify retry logic worked
   }
   ```

3. **Error Handling Tests**
   ```rust
   #[tokio::test]
   async fn test_network_timeout() {
       let mock_server = LLMMockServer::new().await;
       mock_server.mock_timeout();
       
       let adapter = OpenAIAdapter::with_client(
           mock_client(&mock_server.url()),
           test_config().with_timeout(Duration::from_millis(100)),
       );
       
       let response = adapter.complete("Test prompt").await;
       assert!(matches!(response, Err(Error::NetworkTimeout(_))));
   }
   ```

4. **Contract Tests**
   ```rust
   // Verify our mocks match real API contracts
   #[tokio::test]
   #[ignore] // Run manually against real API
   async fn test_openai_api_contract() {
       let adapter = OpenAIAdapter::new(real_config());
       let response = adapter.complete("Say hello").await;
       
       // Verify response structure matches our expectations
       assert!(response.is_ok());
       let resp = response.unwrap();
       assert!(!resp.content.is_empty());
       assert!(resp.tokens_used > 0);
   }
   ```

## Testing Requirements
- Mock server setup/teardown
- Concurrent test execution support
- Environment variable isolation
- Response time assertions
- Request validation (headers, body)

## Acceptance Criteria
- [ ] All LLM adapters have >80% test coverage
- [ ] Error scenarios are comprehensively tested
- [ ] Tests run in <30 seconds
- [ ] No real API calls in CI tests
- [ ] Retry logic is verified
- [ ] Streaming responses are tested
- [ ] Provider switching is tested

## Performance Considerations
- Use connection pooling in tests
- Parallelize test execution
- Mock server reuse where possible
- Minimal test data fixtures

## Future Enhancements
- Load testing with many concurrent requests
- Chaos testing (random failures)
- Response time benchmarks
- Token counting accuracy tests
- Multi-turn conversation tests