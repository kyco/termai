# Task: Add Comprehensive TUI Testing Framework

## Priority: High
## Estimated Effort: 4-6 days
## Dependencies: None

## Overview
Implement a testing framework for the Terminal User Interface to ensure UI reliability and prevent regressions. Currently, the TUI has minimal test coverage despite being the primary user interface.

## Requirements

### Functional Requirements
1. **Terminal Emulation**
   - Virtual terminal for rendering tests
   - Keyboard and mouse event simulation
   - Screen capture and comparison
   - State verification

2. **Test Scenarios**
   - Navigation between panels
   - Text input and editing
   - Visual mode selection
   - Session switching
   - Settings interaction
   - Error dialog handling
   - Help modal display

3. **Regression Prevention**
   - Screenshot-based tests
   - Layout verification
   - Color/style checks
   - Widget positioning

### Technical Requirements
1. **Testing Infrastructure**
   ```rust
   // tests/integration/tui/test_framework.rs
   pub struct TuiTestHarness {
       terminal: TestTerminal,
       app: App,
       events: Vec<Event>,
   }
   
   impl TuiTestHarness {
       pub fn new() -> Self;
       pub fn send_key(&mut self, key: KeyCode);
       pub fn send_keys(&mut self, keys: &str);
       pub fn click_at(&mut self, x: u16, y: u16);
       pub fn get_screen(&self) -> Screen;
       pub fn assert_screen_contains(&self, text: &str);
       pub fn assert_focused(&self, component: Component);
   }
   ```

2. **Test Terminal Backend**
   ```rust
   // A backend that captures all rendering
   pub struct TestBackend {
       buffer: Buffer,
       cursor_position: (u16, u16),
   }
   
   impl Backend for TestBackend {
       // Implement required methods
   }
   ```

3. **Screen Assertion Helpers**
   ```rust
   pub struct Screen {
       content: Vec<Vec<Cell>>,
       width: u16,
       height: u16,
   }
   
   impl Screen {
       pub fn find_text(&self, text: &str) -> Option<(u16, u16)>;
       pub fn get_widget_at(&self, x: u16, y: u16) -> Option<&Widget>;
       pub fn snapshot(&self) -> String;
   }
   ```

## Implementation Steps

1. **Create Test Framework**
   ```rust
   // tests/integration/tui/mod.rs
   mod test_framework;
   mod navigation_tests;
   mod input_tests;
   mod session_tests;
   mod visual_mode_tests;
   ```

2. **Navigation Tests**
   ```rust
   #[tokio::test]
   async fn test_tab_navigation() {
       let mut harness = TuiTestHarness::new().await;
       
       // Start with sessions focused
       harness.assert_focused(Component::SessionList);
       
       // Tab to chat
       harness.send_key(KeyCode::Tab);
       harness.assert_focused(Component::Chat);
       
       // Tab to input
       harness.send_key(KeyCode::Tab);
       harness.assert_focused(Component::Input);
       
       // Tab back to sessions
       harness.send_key(KeyCode::Tab);
       harness.assert_focused(Component::SessionList);
   }
   
   #[tokio::test]
   async fn test_arrow_navigation() {
       let mut harness = TuiTestHarness::new().await;
       harness.create_sessions(3).await;
       
       // Navigate sessions with arrows
       harness.send_key(KeyCode::Down);
       harness.assert_selected_session(1);
       
       harness.send_key(KeyCode::Down);
       harness.assert_selected_session(2);
       
       harness.send_key(KeyCode::Up);
       harness.assert_selected_session(1);
   }
   ```

3. **Input Tests**
   ```rust
   #[tokio::test]
   async fn test_message_input() {
       let mut harness = TuiTestHarness::new().await;
       
       // Focus input
       harness.focus_input();
       
       // Enter edit mode
       harness.send_key(KeyCode::Enter);
       harness.assert_mode(Mode::Edit);
       
       // Type message
       harness.send_keys("Hello, AI!");
       harness.assert_input_contains("Hello, AI!");
       
       // Send message
       harness.send_key(KeyCode::Enter);
       harness.assert_message_sent("Hello, AI!");
       harness.assert_input_empty();
   }
   ```

4. **Visual Mode Tests**
   ```rust
   #[tokio::test]
   async fn test_visual_selection() {
       let mut harness = TuiTestHarness::new().await;
       harness.add_message("Line 1\nLine 2\nLine 3").await;
       
       // Enter visual mode
       harness.send_key(KeyCode::Char('v'));
       harness.assert_mode(Mode::Visual);
       
       // Select text
       harness.send_key(KeyCode::Right);
       harness.send_key(KeyCode::Right);
       harness.assert_selection_length(3);
       
       // Copy selection
       harness.send_key(KeyCode::Char('y'));
       harness.assert_clipboard_contains("Lin");
   }
   ```

5. **Screenshot Tests**
   ```rust
   #[tokio::test]
   async fn test_ui_layout() {
       let mut harness = TuiTestHarness::new().await;
       harness.set_terminal_size(80, 24);
       
       let screen = harness.render_and_capture();
       
       // Compare with golden screenshot
       let golden = include_str!("golden/main_layout.txt");
       assert_screen_matches(&screen, golden);
   }
   ```

## Testing Requirements
- Tests must run headlessly in CI
- Support different terminal sizes
- Mock external dependencies (DB, LLM)
- Deterministic rendering
- Fast execution (<100ms per test)

## Acceptance Criteria
- [ ] Test framework supports all UI interactions
- [ ] >80% code coverage for TUI modules
- [ ] All keybindings have tests
- [ ] Visual regression tests in place
- [ ] Tests run in parallel
- [ ] Clear test failure messages

## Tools and Dependencies
- Consider `ratatui-testkit` if available
- Custom test backend implementation
- Screenshot comparison library
- Event simulation helpers

## Future Enhancements
- Accessibility testing
- Performance profiling in tests
- Fuzzing for unexpected inputs
- Cross-terminal compatibility tests
- Theme testing support