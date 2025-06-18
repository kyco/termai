# Task: Implement Error Recovery and Edge Case Tests

## Priority: High
## Estimated Effort: 3-4 days
## Dependencies: Basic test infrastructure

## Overview
Create comprehensive tests for error scenarios and edge cases to ensure the application handles failures gracefully and provides a good user experience even when things go wrong.

## Requirements

### Functional Requirements
1. **Network Error Scenarios**
   - Connection timeouts
   - DNS resolution failures
   - Intermittent connection drops
   - Proxy/firewall issues
   - SSL/TLS errors

2. **API Error Scenarios**
   - Invalid API keys
   - Rate limiting (429 errors)
   - Service unavailable (503)
   - Token limit exceeded
   - Malformed responses
   - Partial responses

3. **Storage Error Scenarios**
   - Database corruption
   - Disk full
   - Permission denied
   - Concurrent access conflicts
   - Migration failures

4. **UI Error Recovery**
   - Graceful degradation
   - Error message display
   - Retry mechanisms
   - State recovery after crashes

### Technical Requirements
1. **Error Simulation Framework**
   ```rust
   // tests/common/error_injection.rs
   pub struct ErrorInjector {
       error_types: HashMap<String, ErrorConfig>,
   }
   
   pub struct ErrorConfig {
       probability: f32,
       error_type: ErrorType,
       duration: Option<Duration>,
   }
   
   impl ErrorInjector {
       pub fn inject_network_errors(&mut self, config: NetworkErrorConfig);
       pub fn inject_storage_errors(&mut self, config: StorageErrorConfig);
       pub fn inject_random_errors(&mut self, probability: f32);
   }
   ```

2. **Chaos Testing Utilities**
   ```rust
   // tests/common/chaos.rs
   pub struct ChaosMonkey {
       injector: ErrorInjector,
       logger: TestLogger,
   }
   
   impl ChaosMonkey {
       pub async fn run_with_errors<F>(&self, test: F) -> TestResult
       where F: Future<Output = ()>;
   }
   ```

## Implementation Steps

1. **Network Error Tests**
   ```rust
   #[tokio::test]
   async fn test_network_timeout_recovery() {
       let mut app = TestApp::new().await;
       let error_injector = ErrorInjector::new();
       
       // Inject timeout after 100ms
       error_injector.inject_network_timeout(Duration::from_millis(100));
       
       // Attempt API call
       let result = app.send_message("Hello").await;
       
       // Verify timeout error shown to user
       assert!(matches!(result, Err(Error::NetworkTimeout(_))));
       assert!(app.ui().shows_error("Network timeout"));
       
       // Verify retry option available
       assert!(app.ui().has_retry_button());
       
       // Clear error and retry
       error_injector.clear_errors();
       app.retry_last_action().await;
       
       // Verify success
       assert!(app.last_message_sent());
   }
   
   #[tokio::test]
   async fn test_rate_limit_handling() {
       let mut app = TestApp::new().await;
       let mock_server = MockServer::start().await;
       
       // Mock rate limit response
       Mock::given(method("POST"))
           .respond_with(
               ResponseTemplate::new(429)
                   .set_body_json(json!({
                       "error": "Rate limit exceeded",
                       "retry_after": 5
                   }))
           )
           .mount(&mock_server)
           .await;
       
       // Send message
       let result = app.send_message("Test").await;
       
       // Verify user-friendly error
       assert!(app.ui().shows_error("Too many requests. Please wait 5 seconds."));
       
       // Verify automatic retry after delay
       tokio::time::sleep(Duration::from_secs(6)).await;
       assert!(app.message_sent_successfully());
   }
   ```

2. **Storage Error Tests**
   ```rust
   #[tokio::test]
   async fn test_database_corruption_recovery() {
       let temp_dir = TempDir::new().unwrap();
       let db_path = temp_dir.path().join("test.db");
       
       // Create corrupted database
       fs::write(&db_path, b"corrupted data").unwrap();
       
       // Attempt to start app
       let result = TestApp::with_db_path(&db_path).await;
       
       // Verify error handling
       assert!(matches!(result, Err(Error::DatabaseCorrupted(_))));
       
       // Verify backup creation
       assert!(db_path.with_extension("backup").exists());
       
       // Verify automatic recreation
       let app = TestApp::with_db_path(&db_path).await.unwrap();
       assert!(app.is_functional());
   }
   
   #[tokio::test]
   async fn test_disk_full_handling() {
       let mut app = TestApp::new().await;
       
       // Simulate disk full
       app.inject_error(ErrorType::DiskFull);
       
       // Attempt to save session
       let result = app.save_session().await;
       
       // Verify graceful handling
       assert!(matches!(result, Err(Error::DiskFull(_))));
       assert!(app.ui().shows_error("Unable to save: Disk full"));
       
       // Verify session kept in memory
       assert!(app.has_unsaved_changes());
       assert!(app.ui().shows_warning("Unsaved changes"));
   }
   ```

3. **Concurrent Access Tests**
   ```rust
   #[tokio::test]
   async fn test_concurrent_session_access() {
       let db_path = TempDir::new().unwrap().path().join("test.db");
       
       // Start multiple app instances
       let app1 = TestApp::with_db_path(&db_path).await.unwrap();
       let app2 = TestApp::with_db_path(&db_path).await.unwrap();
       
       // Create session in app1
       let session_id = app1.create_session("Test").await.unwrap();
       
       // Both apps try to modify same session
       let (r1, r2) = tokio::join!(
           app1.add_message(&session_id, "From app1"),
           app2.add_message(&session_id, "From app2")
       );
       
       // Verify both succeed (proper locking)
       assert!(r1.is_ok());
       assert!(r2.is_ok());
       
       // Verify messages are consistent
       let messages = app1.get_messages(&session_id).await.unwrap();
       assert_eq!(messages.len(), 2);
   }
   ```

4. **UI State Recovery Tests**
   ```rust
   #[tokio::test]
   async fn test_ui_crash_recovery() {
       let mut app = TestApp::new().await;
       
       // Set up state
       app.create_session("Important").await;
       app.enter_message("Unsent message");
       app.focus_settings();
       
       // Simulate crash
       let state_backup = app.backup_state();
       drop(app);
       
       // Restart app
       let mut app = TestApp::new().await;
       app.restore_state(state_backup);
       
       // Verify state recovered
       assert_eq!(app.current_input(), "Unsent message");
       assert!(app.is_focused_on_settings());
       assert!(app.session_exists("Important"));
   }
   ```

## Testing Requirements
- Error injection must be deterministic
- Tests should verify user-facing error messages
- Recovery mechanisms must be tested
- Performance impact of errors should be measured
- Logging during errors must be verified

## Acceptance Criteria
- [ ] All identified error scenarios have tests
- [ ] Error messages are user-friendly
- [ ] Recovery mechanisms work correctly
- [ ] No data loss during errors
- [ ] UI remains responsive during errors
- [ ] Errors are properly logged

## Error Message Guidelines
- Be specific but not technical
- Provide actionable next steps
- Include retry options where appropriate
- Don't expose sensitive information
- Log technical details separately

## Future Enhancements
- Automated error reporting
- Self-healing mechanisms
- Predictive error prevention
- Error analytics dashboard
- A/B testing error messages