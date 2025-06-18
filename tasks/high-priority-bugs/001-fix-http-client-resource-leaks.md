# Task: Fix HTTP Client Resource Leaks and Connection Management

## Priority: High
## Estimated Effort: 1-2 days
## Dependencies: None
## Files Affected: `src/llm/openai/adapter/open_ai_adapter.rs`, `src/llm/claude/adapter/claude_adapter.rs`

## Overview
Fix resource leaks caused by creating new HTTP clients for every API request instead of reusing connections. This leads to connection exhaustion, poor performance, and potential memory leaks under load.

## Bug Description
Both OpenAI and Claude adapters create a new `reqwest::Client` for every API call, which prevents connection reuse and can exhaust system resources. Each client creates its own connection pool that is discarded after use.

## Root Cause Analysis
1. **Resource Waste**: New client created for each request
2. **No Connection Pooling**: Each request establishes new TCP connection
3. **Memory Inefficiency**: Client objects accumulate without reuse
4. **Performance Impact**: Handshake overhead for every request
5. **Scale Issues**: System resource exhaustion under high load

## Current Buggy Code
```rust
// In openai_adapter.rs and claude_adapter.rs
pub async fn chat(request: &ChatCompletionRequest, api_key: &str) -> Result<ChatCompletionResponse> {
    let client = Client::new(); // BUG: Creates new client every time
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        // ... rest of request
}
```

## Implementation Steps

### 1. Create Shared HTTP Client Manager
```rust
// src/http/client_manager.rs
use reqwest::{Client, ClientBuilder};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use anyhow::Result;

static HTTP_CLIENT: OnceLock<Arc<Client>> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool_idle_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub max_retries: u32,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            connect_timeout: Duration::from_secs(10),
            pool_idle_timeout: Duration::from_secs(90),
            pool_max_idle_per_host: 10,
            max_retries: 3,
            user_agent: format!("TermAI/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

pub struct HttpClientManager;

impl HttpClientManager {
    /// Get or create the global HTTP client
    pub fn client() -> Arc<Client> {
        HTTP_CLIENT.get_or_init(|| {
            Arc::new(Self::create_client(HttpClientConfig::default()))
        }).clone()
    }
    
    /// Get client with custom configuration
    pub fn client_with_config(config: HttpClientConfig) -> Arc<Client> {
        Arc::new(Self::create_client(config))
    }
    
    /// Create a new client with the given configuration
    fn create_client(config: HttpClientConfig) -> Client {
        ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_idle_timeout(config.pool_idle_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .user_agent(config.user_agent)
            .use_rustls_tls() // Use rustls for better performance
            .build()
            .expect("Failed to create HTTP client")
    }
    
    /// Initialize the global client with custom config
    pub fn initialize(config: HttpClientConfig) -> Result<()> {
        HTTP_CLIENT.set(Arc::new(Self::create_client(config)))
            .map_err(|_| anyhow::anyhow!("HTTP client already initialized"))?;
        Ok(())
    }
    
    /// Get client statistics (if available)
    pub fn stats() -> ClientStats {
        // Note: reqwest doesn't expose detailed connection pool stats
        // This would need to be implemented with custom metrics
        ClientStats {
            active_connections: 0, // Would need custom tracking
            idle_connections: 0,   // Would need custom tracking
            total_requests: 0,     // Would need custom tracking
        }
    }
}

#[derive(Debug, Default)]
pub struct ClientStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
}
```

### 2. Add Request Timeout and Retry Logic
```rust
// src/http/request_handler.rs
use reqwest::{Client, Response, StatusCode};
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Result, Context};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retry_on_status: Vec<StatusCode>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retry_on_status: vec![
                StatusCode::TOO_MANY_REQUESTS,
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::BAD_GATEWAY,
                StatusCode::SERVICE_UNAVAILABLE,
                StatusCode::GATEWAY_TIMEOUT,
            ],
        }
    }
}

pub struct RequestHandler {
    client: Arc<Client>,
    retry_config: RetryConfig,
}

impl RequestHandler {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            retry_config: RetryConfig::default(),
        }
    }
    
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }
    
    pub async fn post_json<T: Serialize>(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &T,
    ) -> Result<Response> {
        let mut attempt = 0;
        let mut delay = self.retry_config.base_delay;
        
        loop {
            attempt += 1;
            
            let mut request = self.client
                .post(url)
                .json(body);
            
            // Add headers
            for (key, value) in headers {
                request = request.header(*key, *value);
            }
            
            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }
                    
                    // Check if we should retry based on status code
                    if attempt < self.retry_config.max_attempts &&
                       self.retry_config.retry_on_status.contains(&response.status()) {
                        
                        let retry_after = self.get_retry_after(&response);
                        let actual_delay = retry_after.unwrap_or(delay);
                        
                        eprintln!("Request failed with {}, retrying in {:?} (attempt {}/{})", 
                                response.status(), actual_delay, attempt, self.retry_config.max_attempts);
                        
                        sleep(actual_delay).await;
                        delay = self.calculate_next_delay(delay);
                        continue;
                    }
                    
                    return Ok(response); // Return even if not successful for caller to handle
                }
                Err(e) => {
                    if attempt < self.retry_config.max_attempts && self.is_retryable_error(&e) {
                        eprintln!("Request failed with error: {}, retrying in {:?} (attempt {}/{})", 
                                e, delay, attempt, self.retry_config.max_attempts);
                        
                        sleep(delay).await;
                        delay = self.calculate_next_delay(delay);
                        continue;
                    }
                    
                    return Err(anyhow::anyhow!("Request failed after {} attempts: {}", attempt, e));
                }
            }
        }
    }
    
    fn get_retry_after(&self, response: &Response) -> Option<Duration> {
        response.headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .map(|seconds| Duration::from_secs(seconds))
    }
    
    fn calculate_next_delay(&self, current_delay: Duration) -> Duration {
        let next = Duration::from_millis(
            (current_delay.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64
        );
        next.min(self.retry_config.max_delay)
    }
    
    fn is_retryable_error(&self, error: &reqwest::Error) -> bool {
        error.is_timeout() || 
        error.is_connect() ||
        error.is_request() && !error.is_body()
    }
}
```

### 3. Update OpenAI Adapter with Shared Client
```rust
// src/llm/openai/adapter/open_ai_adapter.rs
use crate::http::client_manager::HttpClientManager;
use crate::http::request_handler::{RequestHandler, RetryConfig};
use crate::llm::openai::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::openai::model::chat_completion_response::ChatCompletionResponse;
use anyhow::{Result, Context};
use std::time::Duration;

pub struct OpenAIAdapter {
    request_handler: RequestHandler,
    base_url: String,
}

impl OpenAIAdapter {
    pub fn new() -> Self {
        let client = HttpClientManager::client();
        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            retry_on_status: vec![
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                reqwest::StatusCode::BAD_GATEWAY,
                reqwest::StatusCode::SERVICE_UNAVAILABLE,
            ],
        };
        
        Self {
            request_handler: RequestHandler::new(client).with_retry_config(retry_config),
            base_url: "https://api.openai.com".to_string(),
        }
    }
    
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

impl Default for OpenAIAdapter {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<ChatCompletionResponse> {
    let adapter = OpenAIAdapter::new();
    adapter.chat_completion(request, api_key).await
}

impl OpenAIAdapter {
    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
        api_key: &str,
    ) -> Result<ChatCompletionResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        let headers = [
            ("Content-Type", "application/json"),
            ("Authorization", &format!("Bearer {}", api_key)),
        ];
        
        let response = self.request_handler
            .post_json(&url, &headers, request)
            .await
            .context("Failed to send request to OpenAI API")?;
        
        if response.status().is_success() {
            let completion: ChatCompletionResponse = response
                .json()
                .await
                .context("Failed to parse OpenAI API response")?;
            Ok(completion)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            Err(anyhow::anyhow!(
                "OpenAI API error ({}): {}", 
                status, 
                error_text
            ))
        }
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        // Simple health check endpoint (if available)
        let url = format!("{}/v1/models", self.base_url);
        let headers = [("Content-Type", "application/json")];
        
        match self.request_handler.post_json(&url, &headers, &serde_json::json!({})).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
```

### 4. Update Claude Adapter with Shared Client
```rust
// src/llm/claude/adapter/claude_adapter.rs
use crate::http::client_manager::HttpClientManager;
use crate::http::request_handler::{RequestHandler, RetryConfig};
use crate::llm::claude::model::chat_completion_request::ChatCompletionRequest;
use crate::llm::claude::model::chat_completion_response::ChatCompletionResponse;
use anyhow::{Result, Context};
use reqwest::StatusCode;
use std::time::Duration;

pub struct ClaudeAdapter {
    request_handler: RequestHandler,
    base_url: String,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let client = HttpClientManager::client();
        let retry_config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(120), // Claude can have longer delays
            backoff_multiplier: 2.0,
            retry_on_status: vec![
                StatusCode::TOO_MANY_REQUESTS,
                StatusCode::INTERNAL_SERVER_ERROR,
                StatusCode::BAD_GATEWAY,
                StatusCode::SERVICE_UNAVAILABLE,
            ],
        };
        
        Self {
            request_handler: RequestHandler::new(client).with_retry_config(retry_config),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn chat(
    request: &ChatCompletionRequest,
    api_key: &str,
) -> Result<(StatusCode, ChatCompletionResponse)> {
    let adapter = ClaudeAdapter::new();
    adapter.chat_completion(request, api_key).await
}

impl ClaudeAdapter {
    pub async fn chat_completion(
        &self,
        request: &ChatCompletionRequest,
        api_key: &str,
    ) -> Result<(StatusCode, ChatCompletionResponse)> {
        let url = format!("{}/v1/messages", self.base_url);
        
        let headers = [
            ("Content-Type", "application/json"),
            ("x-api-key", api_key),
            ("anthropic-version", "2023-06-01"),
        ];
        
        let response = self.request_handler
            .post_json(&url, &headers, request)
            .await
            .context("Failed to send request to Claude API")?;
        
        let status = response.status();
        
        if status.is_success() {
            let completion: ChatCompletionResponse = response
                .json()
                .await
                .context("Failed to parse Claude API response")?;
            Ok((status, completion))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            Err(anyhow::anyhow!(
                "Claude API error ({}): {}", 
                status, 
                error_text
            ))
        }
    }
}
```

### 5. Add Connection Pool Monitoring
```rust
// src/http/metrics.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct HttpMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    retry_count: AtomicU64,
    total_duration: AtomicU64, // in milliseconds
}

impl HttpMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            retry_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
        })
    }
    
    pub fn record_request(&self, duration: Duration, success: bool, retries: u32) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_duration.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        self.retry_count.fetch_add(retries as u64, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    pub fn get_stats(&self) -> HttpStats {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_duration = self.total_duration.load(Ordering::Relaxed);
        let retries = self.retry_count.load(Ordering::Relaxed);
        
        HttpStats {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate: if total > 0 { successful as f64 / total as f64 } else { 0.0 },
            average_duration_ms: if total > 0 { total_duration as f64 / total as f64 } else { 0.0 },
            total_retries: retries,
            average_retries: if total > 0 { retries as f64 / total as f64 } else { 0.0 },
        }
    }
    
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.retry_count.store(0, Ordering::Relaxed);
        self.total_duration.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct HttpStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub total_retries: u64,
    pub average_retries: f64,
}

impl Default for HttpMetrics {
    fn default() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            retry_count: AtomicU64::new(0),
            total_duration: AtomicU64::new(0),
        }
    }
}
```

### 6. Initialize HTTP Client on Application Startup
```rust
// src/main.rs
use crate::http::client_manager::{HttpClientManager, HttpClientConfig};

fn main() -> Result<()> {
    // Initialize HTTP client with custom configuration
    let http_config = HttpClientConfig {
        timeout: Duration::from_secs(120), // Longer timeout for AI APIs
        connect_timeout: Duration::from_secs(10),
        pool_idle_timeout: Duration::from_secs(300), // Keep connections alive longer
        pool_max_idle_per_host: 20, // More connections for better performance
        max_retries: 3,
        user_agent: format!("TermAI/{}", env!("CARGO_PKG_VERSION")),
    };
    
    HttpClientManager::initialize(http_config)
        .context("Failed to initialize HTTP client")?;
    
    // ... rest of application initialization
    
    Ok(())
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_http_client_reuse() {
        // Get two client instances
        let client1 = HttpClientManager::client();
        let client2 = HttpClientManager::client();
        
        // Should be the same instance (Arc comparison)
        assert!(Arc::ptr_eq(&client1, &client2));
    }
    
    #[tokio::test]
    async fn test_retry_logic() {
        let client = HttpClientManager::client();
        let handler = RequestHandler::new(client);
        
        // Test with mock server that returns 429 then 200
        // This would require a mock HTTP server for proper testing
        // Implementation depends on testing framework choice
    }
    
    #[tokio::test]
    async fn test_timeout_handling() {
        let config = HttpClientConfig {
            timeout: Duration::from_millis(100), // Very short timeout
            ..Default::default()
        };
        
        let client = HttpClientManager::client_with_config(config);
        let handler = RequestHandler::new(client);
        
        // Test request to slow endpoint (would need mock server)
        // Should timeout and return appropriate error
    }
    
    #[test]
    fn test_metrics_tracking() {
        let metrics = HttpMetrics::new();
        
        // Record some requests
        metrics.record_request(Duration::from_millis(100), true, 0);
        metrics.record_request(Duration::from_millis(200), false, 2);
        
        let stats = metrics.get_stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.success_rate, 0.5);
        assert_eq!(stats.total_retries, 2);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_openai_adapter_connection_reuse() {
    let adapter1 = OpenAIAdapter::new();
    let adapter2 = OpenAIAdapter::new();
    
    // Both should use the same underlying client
    // This test would need access to internal client reference
    // or behavioral testing showing connection reuse
}

#[tokio::test]
async fn test_concurrent_requests() {
    let adapter = OpenAIAdapter::new();
    
    // Send multiple concurrent requests
    let handles: Vec<_> = (0..10).map(|i| {
        let adapter = adapter.clone(); // If we make it clonable
        tokio::spawn(async move {
            // Make test request (would need mock or test API key)
            // adapter.health_check().await
        })
    }).collect();
    
    // All requests should complete successfully
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}
```

### Load Tests
```rust
#[ignore] // Only run during load testing
#[tokio::test]
async fn test_connection_pool_under_load() {
    let adapter = OpenAIAdapter::new();
    
    // Simulate high load
    let handles: Vec<_> = (0..1000).map(|_| {
        let adapter = adapter.clone();
        tokio::spawn(async move {
            adapter.health_check().await
        })
    }).collect();
    
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Most requests should succeed
    let success_count = results.into_iter().filter(|r| r.is_ok()).count();
    assert!(success_count > 900); // Allow some failures under extreme load
}
```

## Performance Monitoring

### Add Debug Output for Connection Pool
```rust
// In settings or debug mode
pub fn show_http_stats() {
    let stats = HttpClientManager::stats();
    eprintln!("HTTP Client Stats:");
    eprintln!("  Active connections: {}", stats.active_connections);
    eprintln!("  Idle connections: {}", stats.idle_connections);
    eprintln!("  Total requests: {}", stats.total_requests);
}
```

## Acceptance Criteria
- [ ] Single shared HTTP client used for all requests
- [ ] Connection pooling works correctly
- [ ] Retry logic handles transient failures
- [ ] Timeouts are properly configured
- [ ] No resource leaks under load
- [ ] Performance improves with connection reuse
- [ ] Error handling is robust
- [ ] Metrics are available for monitoring

## Performance Expectations
- **Connection Reuse**: 90%+ of requests should reuse existing connections
- **Response Time**: 50% improvement in average response time due to reduced handshake overhead
- **Memory Usage**: Stable memory usage even under high request volume
- **Resource Limits**: No connection exhaustion up to 1000 concurrent requests

## Rollback Plan
1. Keep old implementation as feature-flagged fallback
2. Monitor resource usage and performance metrics
3. Gradual rollout with ability to disable connection pooling

## Future Enhancements
- Add HTTP/2 support for better multiplexing
- Implement circuit breaker pattern for failing endpoints
- Add request/response caching layer
- Implement custom connection pool with better metrics
- Add distributed tracing for request flows