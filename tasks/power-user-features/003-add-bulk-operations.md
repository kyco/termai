# Task: Add Bulk Operations for Session Management

## Priority: Medium
## Estimated Effort: 3-4 days
## Dependencies: Session organization system

## Overview
Implement bulk operations to allow users to efficiently manage multiple sessions at once. This addresses the pain point of managing large numbers of sessions by providing select-all, multi-select, and batch operations.

## Requirements

### Functional Requirements
1. **Selection Modes**
   - Multi-select with checkboxes
   - Select all/none toggle
   - Range selection (Shift+click)
   - Filter-based selection (select all matching search)

2. **Bulk Operations**
   - Delete multiple sessions
   - Move sessions to folder
   - Apply tags to multiple sessions
   - Export selected sessions
   - Mark sessions as archived

3. **UI/UX**
   - Visual feedback for selected sessions
   - Bulk action toolbar
   - Confirmation dialogs for destructive actions
   - Progress indicators for long operations
   - Undo capability for recent bulk actions

### Technical Requirements
1. **Selection State Management**
   ```rust
   // In app.rs
   pub struct BulkSelectionState {
       selected_sessions: HashSet<String>,
       selection_mode: SelectionMode,
       last_selected_index: Option<usize>,
   }
   
   pub enum SelectionMode {
       None,
       Multi,
       Range,
   }
   
   impl BulkSelectionState {
       pub fn toggle_session(&mut self, session_id: &str);
       pub fn select_range(&mut self, start: usize, end: usize, sessions: &[Session]);
       pub fn select_all(&mut self, sessions: &[Session]);
       pub fn clear_selection(&mut self);
       pub fn is_selected(&self, session_id: &str) -> bool;
   }
   ```

2. **Bulk Operations Service**
   ```rust
   pub struct BulkOperationsService {
       session_repo: Arc<SessionRepository>,
       message_repo: Arc<MessageRepository>,
       export_service: Arc<ExportService>,
   }
   
   impl BulkOperationsService {
       pub async fn delete_sessions(&self, session_ids: Vec<String>) -> Result<BulkOperationResult>;
       pub async fn move_sessions(&self, session_ids: Vec<String>, folder_id: &str) -> Result<BulkOperationResult>;
       pub async fn apply_tags(&self, session_ids: Vec<String>, tags: Vec<String>) -> Result<BulkOperationResult>;
       pub async fn export_sessions(&self, session_ids: Vec<String>, format: ExportFormat) -> Result<BulkOperationResult>;
   }
   
   pub struct BulkOperationResult {
       pub successful: Vec<String>,
       pub failed: Vec<(String, String)>, // (session_id, error_message)
       pub operation_id: String, // For undo operations
   }
   ```

3. **Undo System**
   ```rust
   pub struct UndoManager {
       operations: VecDeque<UndoableOperation>,
       max_operations: usize,
   }
   
   pub enum UndoableOperation {
       BulkDelete {
           sessions: Vec<Session>,
           messages: HashMap<String, Vec<Message>>,
           operation_id: String,
       },
       BulkMove {
           session_moves: Vec<(String, Option<String>)>, // (session_id, old_folder_id)
           operation_id: String,
       },
   }
   ```

## Implementation Steps

1. **Add Selection State to App**
   ```rust
   // In app.rs
   impl App {
       pub fn enter_bulk_mode(&mut self) {
           self.bulk_selection = Some(BulkSelectionState::new());
           self.mode = Mode::BulkSelection;
       }
       
       pub fn exit_bulk_mode(&mut self) {
           self.bulk_selection = None;
           self.mode = Mode::Normal;
       }
       
       pub fn toggle_session_selection(&mut self, session_id: &str) {
           if let Some(ref mut selection) = self.bulk_selection {
               selection.toggle_session(session_id);
           }
       }
   }
   ```

2. **Update Event Handling**
   ```rust
   // In events.rs
   impl EventHandler {
       fn handle_bulk_selection_mode(&mut self, app: &mut App, key: KeyEvent) -> Result<()> {
           match key.code {
               KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                   app.select_all_sessions();
               }
               KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                   app.deselect_all_sessions();
               }
               KeyCode::Delete => {
                   app.show_bulk_delete_confirmation();
               }
               KeyCode::Char('m') => {
                   app.show_bulk_move_dialog();
               }
               KeyCode::Char('t') => {
                   app.show_bulk_tag_dialog();
               }
               KeyCode::Char('e') => {
                   app.show_bulk_export_dialog();
               }
               KeyCode::Space => {
                   app.toggle_current_session_selection();
               }
               KeyCode::Esc => {
                   app.exit_bulk_mode();
               }
               _ => {}
           }
           Ok(())
       }
   }
   ```

3. **UI Components**
   ```rust
   // In ui.rs
   fn draw_bulk_selection_toolbar(f: &mut Frame, app: &App, area: Rect) {
       let selected_count = app.get_selected_session_count();
       let actions = vec![
           format!("Delete ({})", selected_count),
           "Move to Folder",
           "Add Tags",
           "Export",
           "Archive",
       ];
       
       let toolbar = List::new(actions)
           .style(Style::default().fg(Color::White))
           .highlight_style(Style::default().bg(Color::Blue))
           .block(Block::default().title("Bulk Actions").borders(Borders::ALL));
       
       f.render_stateful_widget(toolbar, area, &mut app.bulk_action_state);
   }
   
   fn draw_session_list_with_selection(f: &mut Frame, app: &App, area: Rect) {
       let sessions = app.get_filtered_sessions();
       let list_items: Vec<ListItem> = sessions.iter().enumerate().map(|(i, session)| {
           let checkbox = if app.is_session_selected(&session.id) {
               "☑ "
           } else {
               "☐ "
           };
           
           let content = format!("{}{}", checkbox, session.name);
           let style = if app.is_session_selected(&session.id) {
               Style::default().bg(Color::DarkGray)
           } else {
               Style::default()
           };
           
           ListItem::new(content).style(style)
       }).collect();
       
       let list = List::new(list_items)
           .highlight_style(Style::default().bg(Color::Blue))
           .block(Block::default().borders(Borders::ALL));
       
       f.render_stateful_widget(list, area, &mut app.session_list_state);
   }
   ```

4. **Bulk Operations Implementation**
   ```rust
   impl BulkOperationsService {
       pub async fn delete_sessions(&self, session_ids: Vec<String>) -> Result<BulkOperationResult> {
           let operation_id = Uuid::new_v4().to_string();
           let mut successful = Vec::new();
           let mut failed = Vec::new();
           let mut deleted_sessions = Vec::new();
           let mut deleted_messages = HashMap::new();
           
           for session_id in session_ids {
               match self.delete_single_session(&session_id).await {
                   Ok((session, messages)) => {
                       successful.push(session_id.clone());
                       deleted_sessions.push(session);
                       deleted_messages.insert(session_id, messages);
                   }
                   Err(e) => {
                       failed.push((session_id, e.to_string()));
                   }
               }
           }
           
           // Store for undo
           if !deleted_sessions.is_empty() {
               self.undo_manager.push(UndoableOperation::BulkDelete {
                   sessions: deleted_sessions,
                   messages: deleted_messages,
                   operation_id: operation_id.clone(),
               });
           }
           
           Ok(BulkOperationResult {
               successful,
               failed,
               operation_id,
           })
       }
   }
   ```

5. **Progress Indicators**
   ```rust
   pub struct BulkOperationProgress {
       total: usize,
       completed: usize,
       current_operation: String,
       errors: Vec<String>,
   }
   
   impl BulkOperationProgress {
       pub fn new(total: usize) -> Self {
           Self {
               total,
               completed: 0,
               current_operation: String::new(),
               errors: Vec::new(),
           }
       }
       
       pub fn progress(&self) -> f64 {
           if self.total == 0 { 1.0 } else { self.completed as f64 / self.total as f64 }
       }
   }
   ```

## Testing Requirements
- Unit tests for bulk operations
- Integration tests for undo functionality
- UI tests for selection interactions
- Performance tests with large selections (1000+ sessions)
- Error handling tests for partial failures

## Acceptance Criteria
- [ ] Users can enter bulk selection mode
- [ ] Multiple sessions can be selected via checkbox or keyboard
- [ ] Bulk delete works with confirmation
- [ ] Bulk move to folder works
- [ ] Bulk tag application works
- [ ] Progress is shown for long operations
- [ ] Undo works for recent bulk operations
- [ ] Partial failures are handled gracefully

## Keyboard Shortcuts
- `Ctrl+B`: Enter bulk selection mode
- `Space`: Toggle current session selection
- `Ctrl+A`: Select all visible sessions
- `Ctrl+D`: Deselect all sessions
- `Shift+Arrow`: Range selection
- `Delete`: Bulk delete selected
- `M`: Move selected to folder
- `T`: Add tags to selected
- `E`: Export selected
- `Ctrl+Z`: Undo last bulk operation
- `Esc`: Exit bulk mode

## Future Enhancements
- Smart selection (by date range, tag, etc.)
- Scheduled bulk operations
- Bulk operation history
- Custom bulk operation scripts
- Bulk operation templates
- Keyboard shortcuts customization