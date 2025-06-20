# Task: Add Comprehensive Loading and Progress Indicators

## Priority: Medium
## Estimated Effort: 2-3 days
## Dependencies: None

## Overview
Improve user experience by adding detailed loading indicators, progress bars, and status messages for all operations that take time. This addresses the current lack of feedback during API calls and other operations.

## Requirements

### Functional Requirements
1. **AI Response Loading**
   - Animated "thinking" indicator during API calls
   - Estimated time remaining (based on history)
   - Cancel option for long requests
   - Network status indicators
   - Token counting progress for long responses

2. **Operation Progress**
   - File loading progress bars
   - Database operation indicators
   - Export/import progress tracking
   - Bulk operation progress
   - Session switching feedback

3. **Visual Feedback**
   - Spinner animations
   - Progress bars with percentages
   - Status messages
   - Contextual loading states
   - Error state indicators

### Technical Requirements
1. **Progress Tracking System**
   ```rust
   #[derive(Clone, Debug)]
   pub struct ProgressState {
       pub operation: String,
       pub current: u64,
       pub total: Option<u64>,
       pub message: String,
       pub stage: ProgressStage,
       pub started_at: Instant,
       pub can_cancel: bool,
   }
   
   #[derive(Clone, Debug)]
   pub enum ProgressStage {
       Initializing,
       InProgress,
       Completing,
       Completed,
       Failed(String),
       Cancelled,
   }
   
   pub struct ProgressManager {
       active_operations: HashMap<String, ProgressState>,
       progress_tx: mpsc::Sender<ProgressUpdate>,
   }
   
   #[derive(Clone, Debug)]
   pub struct ProgressUpdate {
       pub operation_id: String,
       pub update: ProgressUpdateType,
   }
   
   #[derive(Clone, Debug)]
   pub enum ProgressUpdateType {
       Started { total: Option<u64>, message: String },
       Progress { current: u64, message: Option<String> },
       Completed,
       Failed(String),
       Cancelled,
   }
   ```

2. **Loading UI Components**
   ```rust
   pub struct LoadingIndicator {
       spinner_frames: Vec<&'static str>,
       current_frame: usize,
       last_update: Instant,
       message: String,
   }
   
   pub struct ProgressBar {
       current: u64,
       total: u64,
       width: u16,
       style: ProgressBarStyle,
   }
   
   #[derive(Clone)]
   pub enum ProgressBarStyle {
       Simple,
       Detailed,
       Circular,
   }
   ```

## Implementation Steps

1. **Progress Manager Service**
   ```rust
   impl ProgressManager {
       pub fn new() -> (Self, mpsc::Receiver<ProgressUpdate>) {
           let (tx, rx) = mpsc::channel(100);
           (Self {
               active_operations: HashMap::new(),
               progress_tx: tx,
           }, rx)
       }
       
       pub async fn start_operation(&mut self, 
           operation_id: String, 
           total: Option<u64>, 
           message: String
       ) -> ProgressTracker {
           let progress = ProgressState {
               operation: operation_id.clone(),
               current: 0,
               total,
               message,
               stage: ProgressStage::Initializing,
               started_at: Instant::now(),
               can_cancel: true,
           };
           
           self.active_operations.insert(operation_id.clone(), progress);
           
           ProgressTracker {
               operation_id,
               progress_tx: self.progress_tx.clone(),
           }
       }
   }
   
   pub struct ProgressTracker {
       operation_id: String,
       progress_tx: mpsc::Sender<ProgressUpdate>,
   }
   
   impl ProgressTracker {
       pub async fn update(&self, current: u64, message: Option<String>) {
           let update = ProgressUpdate {
               operation_id: self.operation_id.clone(),
               update: ProgressUpdateType::Progress { current, message },
           };
           let _ = self.progress_tx.send(update).await;
       }
       
       pub async fn complete(&self) {
           let update = ProgressUpdate {
               operation_id: self.operation_id.clone(),
               update: ProgressUpdateType::Completed,
           };
           let _ = self.progress_tx.send(update).await;
       }
   }
   ```

2. **AI Response Progress Integration**
   ```rust
   // In LLM adapters
   impl OpenAIAdapter {
       pub async fn complete_with_progress(
           &self,
           prompt: &str,
           progress_tracker: Option<ProgressTracker>,
       ) -> Result<CompletionResponse> {
           if let Some(tracker) = &progress_tracker {
               tracker.update(0, Some("Sending request to OpenAI...".to_string())).await;
           }
           
           let request = self.build_request(prompt);
           
           if let Some(tracker) = &progress_tracker {
               tracker.update(25, Some("Waiting for response...".to_string())).await;
           }
           
           let response = self.client.post(&self.config.api_url)
               .json(&request)
               .send()
               .await?;
           
           if let Some(tracker) = &progress_tracker {
               tracker.update(75, Some("Processing response...".to_string())).await;
           }
           
           let completion = response.json::<CompletionResponse>().await?;
           
           if let Some(tracker) = progress_tracker {
               tracker.complete().await;
           }
           
           Ok(completion)
       }
   }
   ```

3. **Loading Indicator Widgets**
   ```rust
   impl LoadingIndicator {
       pub fn new(message: String) -> Self {
           Self {
               spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
               current_frame: 0,
               last_update: Instant::now(),
               message,
           }
       }
       
       pub fn tick(&mut self) {
           if self.last_update.elapsed() > Duration::from_millis(100) {
               self.current_frame = (self.current_frame + 1) % self.spinner_frames.len();
               self.last_update = Instant::now();
           }
       }
       
       pub fn render(&self) -> String {
           format!("{} {}", self.spinner_frames[self.current_frame], self.message)
       }
   }
   
   // In UI rendering
   fn draw_loading_overlay(f: &mut Frame, app: &App, area: Rect) {
       if let Some(progress) = &app.current_progress {
           let loading_area = centered_rect(60, 20, area);
           
           let loading_text = match progress.total {
               Some(total) => {
                   let percentage = (progress.current * 100 / total) as u16;
                   format!("{}\n{}%", progress.message, percentage)
               }
               None => {
                   let spinner = app.loading_indicator.render();
                   format!("{}\n{}", spinner, progress.message)
               }
           };
           
           let loading_widget = Paragraph::new(loading_text)
               .style(Style::default().fg(Color::Yellow))
               .block(Block::default()
                   .title("Loading")
                   .borders(Borders::ALL)
                   .border_style(Style::default().fg(Color::Yellow))
               )
               .alignment(Alignment::Center);
           
           f.render_widget(Clear, loading_area);
           f.render_widget(loading_widget, loading_area);
           
           // Progress bar if total is known
           if let Some(total) = progress.total {
               let progress_area = Rect {
                   y: loading_area.y + loading_area.height - 2,
                   height: 1,
                   ..loading_area
               };
               
               draw_progress_bar(f, progress.current, total, progress_area);
           }
       }
   }
   
   fn draw_progress_bar(f: &mut Frame, current: u64, total: u64, area: Rect) {
       let percentage = ((current * 100) / total) as u16;
       let gauge = Gauge::default()
           .block(Block::default().borders(Borders::NONE))
           .gauge_style(Style::default().fg(Color::Green))
           .percent(percentage);
       
       f.render_widget(gauge, area);
   }
   ```

4. **Status Bar with Progress**
   ```rust
   fn draw_status_bar_with_progress(f: &mut Frame, app: &App, area: Rect) {
       let chunks = Layout::default()
           .direction(Direction::Horizontal)
           .constraints([
               Constraint::Min(0),      // Main status
               Constraint::Length(30),  // Progress info
           ])
           .split(area);
       
       // Main status
       let status_text = format!(
           "Session: {} | Messages: {} | Provider: {}",
           app.current_session().map(|s| s.name.as_str()).unwrap_or("None"),
           app.message_count(),
           app.current_provider()
       );
       
       let status = Paragraph::new(status_text)
           .style(Style::default().fg(Color::White));
       f.render_widget(status, chunks[0]);
       
       // Progress info
       if let Some(progress) = &app.current_progress {
           let progress_text = match progress.stage {
               ProgressStage::InProgress => {
                   let elapsed = progress.started_at.elapsed().as_secs();
                   format!("⏳ {}... ({}s)", progress.operation, elapsed)
               }
               ProgressStage::Completing => "✓ Completing...".to_string(),
               ProgressStage::Failed(ref error) => format!("❌ Failed: {}", error),
               _ => String::new(),
           };
           
           let progress_widget = Paragraph::new(progress_text)
               .style(Style::default().fg(Color::Yellow));
           f.render_widget(progress_widget, chunks[1]);
       }
   }
   ```

5. **Cancellation Support**
   ```rust
   pub struct CancellableOperation {
       progress_tracker: ProgressTracker,
       cancel_token: Arc<AtomicBool>,
   }
   
   impl CancellableOperation {
       pub fn new(operation_id: String, progress_tx: mpsc::Sender<ProgressUpdate>) -> Self {
           Self {
               progress_tracker: ProgressTracker { operation_id, progress_tx },
               cancel_token: Arc::new(AtomicBool::new(false)),
           }
       }
       
       pub fn cancel(&self) {
           self.cancel_token.store(true, Ordering::Relaxed);
       }
       
       pub fn is_cancelled(&self) -> bool {
           self.cancel_token.load(Ordering::Relaxed)
       }
   }
   
   // In event handling
   fn handle_loading_events(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
       if let Some(progress) = &app.current_progress {
           if progress.can_cancel && key.code == KeyCode::Esc {
               app.cancel_current_operation();
               return Ok(());
           }
       }
       Ok(())
   }
   ```

## Different Loading States

1. **AI Response Generation**
   - "🤖 AI is thinking..."
   - Token count progress (if streaming)
   - Estimated time based on history

2. **File Operations**
   - "📁 Loading file..."
   - File size progress bar
   - "💾 Saving session..."

3. **Database Operations**
   - "🗄️ Searching sessions..."
   - "📝 Creating backup..."
   - Bulk operation progress

4. **Network Operations**
   - "🌐 Connecting to API..."
   - "📤 Sending request..."
   - "📥 Receiving response..."

## Testing Requirements
- Unit tests for progress tracking
- UI tests for loading indicators
- Integration tests for cancellation
- Performance tests for progress overhead
- Animation timing tests

## Acceptance Criteria
- [ ] All long-running operations show progress
- [ ] Users can cancel operations where appropriate
- [ ] Progress indicators are visually appealing
- [ ] Estimated time remaining is reasonably accurate
- [ ] Loading states don't impact performance
- [ ] Error states are clearly communicated
- [ ] Progress persists across UI refreshes

## Future Enhancements
- Smart time estimation using historical data
- Background operation notifications
- Progress history and analytics
- Customizable loading animations
- Voice progress announcements (accessibility)
- Progress sharing in collaborative sessions