use crate::session::model::session::Session;
use crate::llm::common::model::role::Role;
use crate::ui::markdown::MarkdownDisplay;
use tui_textarea::TextArea;
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedArea {
    SessionList,
    Chat,
    Input,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Viewing,
    Editing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionMode {
    None,
    Visual,
    VisualLine,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSelection {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

pub struct App {
    pub focused_area: FocusedArea,
    pub input_mode: InputMode,
    pub sessions: Vec<Session>,
    pub current_session_index: usize,
    pub input_area: TextArea<'static>,
    pub should_quit: bool,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub scroll_offset: usize,
    pub session_scroll_offset: usize,
    // Settings view state
    pub settings_selected_index: usize,
    pub settings_editing_key: Option<String>,
    pub settings_input_area: TextArea<'static>,
    pub settings_provider_selecting: bool,
    pub settings_provider_selected_index: usize,
    pub show_settings: bool,
    pub show_help: bool,
    // Area tracking for mouse interactions
    pub session_list_area: Rect,
    pub chat_area: Rect,
    pub input_area_rect: Rect,
    pub settings_area: Rect,
    // Session refresh tracking
    pub session_needs_refresh: bool,
    // Selection state for vim-style text selection
    pub selection_mode: SelectionMode,
    pub cursor_position: CursorPosition,
    pub selection: Option<TextSelection>,
    pub chat_content_lines: Vec<String>, // Cache for selection operations
    // Markdown rendering
    pub markdown_display: Option<MarkdownDisplay>,
}

impl Default for App {
    fn default() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_placeholder_text("Type your message here...");
        
        let mut settings_input_area = TextArea::default();
        settings_input_area.set_placeholder_text("Enter new value...");
        
        Self {
            focused_area: FocusedArea::SessionList,
            input_mode: InputMode::Viewing,
            sessions: vec![Session::new_temporary()],
            current_session_index: 0,
            input_area,
            should_quit: false,
            is_loading: false,
            error_message: None,
            scroll_offset: 0,
            session_scroll_offset: 0,
            settings_selected_index: 0,
            settings_editing_key: None,
            settings_input_area,
            settings_provider_selecting: false,
            settings_provider_selected_index: 0,
            show_settings: false,
            show_help: false,
            session_list_area: Rect::default(),
            chat_area: Rect::default(),
            input_area_rect: Rect::default(),
            settings_area: Rect::default(),
            session_needs_refresh: false,
            selection_mode: SelectionMode::None,
            cursor_position: CursorPosition { line: 0, column: 0 },
            selection: None,
            chat_content_lines: Vec::new(),
            markdown_display: MarkdownDisplay::new().ok(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_session(&self) -> Option<&Session> {
        if self.sessions.is_empty() {
            None
        } else {
            self.sessions.get(self.current_session_index)
        }
    }

    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        if self.sessions.is_empty() {
            None
        } else {
            self.sessions.get_mut(self.current_session_index)
        }
    }

    pub fn add_message_to_current_session(&mut self, content: String, role: Role) {
        if let Some(session) = self.current_session_mut() {
            session.add_raw_message(content, role);
        } else {
            // Create a new session if none exists
            let mut new_session = Session::new_temporary();
            new_session.add_raw_message(content, role);
            self.sessions.push(new_session);
            self.current_session_index = self.sessions.len() - 1;
        }
    }

    /// Check if there are any sessions available
    #[cfg(test)]
    pub fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    /// Get the total number of sessions
    #[cfg(test)]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Ensure the current session index is valid
    fn ensure_valid_session_index(&mut self) {
        if !self.sessions.is_empty() && self.current_session_index >= self.sessions.len() {
            self.current_session_index = self.sessions.len() - 1;
        }
    }

    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
        
        // Safely handle index bounds
        if self.sessions.is_empty() {
            self.current_session_index = 0; // Safe even for empty vec
        } else if self.current_session_index >= self.sessions.len() {
            self.current_session_index = self.sessions.len() - 1; // Last valid index
        }
        
        // Reset scroll when sessions change
        self.scroll_offset = 0;
        self.session_scroll_offset = 0;
    }


    pub fn switch_to_session_by_id(&mut self, session_id: &str) -> bool {
        // Find the session with the matching ID
        for (index, session) in self.sessions.iter().enumerate() {
            if session.id == session_id {
                if index != self.current_session_index {
                    self.current_session_index = index;
                    self.scroll_offset = 0;
                    self.session_needs_refresh = true;
                }
                return true;
            }
        }
        false
    }

    /// Safely remove a session by ID, handling index adjustments
    #[cfg(test)]
    pub fn remove_session(&mut self, session_id: &str) -> bool {
        if let Some(index) = self.sessions.iter().position(|s| s.id == session_id) {
            self.sessions.remove(index);
            
            // Adjust current index after removal
            if self.sessions.is_empty() {
                // Add a new temporary session if all sessions were removed
                self.sessions.push(Session::new_temporary());
                self.current_session_index = 0;
            } else if self.current_session_index >= self.sessions.len() {
                self.current_session_index = self.sessions.len() - 1;
            } else if index <= self.current_session_index && self.current_session_index > 0 {
                self.current_session_index -= 1;
            }
            
            self.scroll_offset = 0;
            self.session_scroll_offset = 0;
            return true;
        }
        false
    }

    pub fn refresh_current_session<SR: crate::session::repository::SessionRepository, MR: crate::session::repository::MessageRepository>(
        &mut self,
        session_repo: &SR,
        message_repo: &MR,
    ) {
        if let Some(current_session) = self.current_session() {
            let session_id = current_session.id.clone();
            match crate::session::service::sessions_service::session_by_id(session_repo, message_repo, &session_id) {
                Ok(updated_session) => {
                    if let Some(session_slot) = self.sessions.get_mut(self.current_session_index) {
                        *session_slot = updated_session;
                    }
                }
                Err(_) => {
                    // If we can't fetch the session, keep the current one
                }
            }
        }
        self.session_needs_refresh = false;
    }

    pub fn next_session(&mut self) {
        if self.sessions.len() <= 1 {
            return; // No navigation needed
        }
        
        // Ensure current index is valid before navigation
        self.ensure_valid_session_index();
        
        let new_index = (self.current_session_index + 1) % self.sessions.len();
        if new_index != self.current_session_index {
            self.current_session_index = new_index;
            self.scroll_offset = 0;
            self.session_needs_refresh = true;
        }
    }

    pub fn previous_session(&mut self) {
        if self.sessions.len() <= 1 {
            return; // No navigation needed
        }
        
        // Ensure current index is valid before navigation
        self.ensure_valid_session_index();
        
        let new_index = if self.current_session_index == 0 {
            self.sessions.len() - 1
        } else {
            self.current_session_index - 1
        };
        
        if new_index != self.current_session_index {
            self.current_session_index = new_index;
            self.scroll_offset = 0;
            self.session_needs_refresh = true;
        }
    }

    pub fn create_new_session(&mut self) {
        let new_session = Session::new_temporary();
        self.sessions.insert(0, new_session);
        self.current_session_index = 0;
        self.scroll_offset = 0;
        self.session_scroll_offset = 0;
        self.focused_area = FocusedArea::Input;
        self.input_mode = InputMode::Editing;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        // Simply increment scroll offset - clamping will be handled by UI
        self.scroll_offset += 1;
    }

    pub fn scroll_to_bottom(&mut self) {
        // Set scroll to a large value - clamping will bring it to the actual bottom
        self.scroll_offset = usize::MAX;
    }


    pub fn clamp_scroll_to_content_lines(&mut self, content_lines: usize, available_height: usize) {
        if content_lines > 0 {
            // If content fits on screen, don't allow scrolling
            if content_lines <= available_height {
                self.scroll_offset = 0;
            } else {
                // Calculate the maximum useful scroll position
                // This ensures we can always see content and can reach the bottom
                let max_scroll = content_lines.saturating_sub(available_height);
                self.scroll_offset = self.scroll_offset.min(max_scroll);
            }
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn session_scroll_up(&mut self) {
        if self.session_scroll_offset > 0 {
            self.session_scroll_offset -= 1;
        }
    }

    pub fn session_scroll_down(&mut self) {
        if self.session_scroll_offset + 1 < self.sessions.len() {
            self.session_scroll_offset += 1;
        }
    }

    pub fn get_input_text(&self) -> String {
        self.input_area.lines().join("\n")
    }

    pub fn clear_input(&mut self) {
        self.input_area = TextArea::default();
        self.input_area.set_placeholder_text("Type your message here...");
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn cycle_focus(&mut self) {
        self.focused_area = match self.focused_area {
            FocusedArea::SessionList => FocusedArea::Chat,
            FocusedArea::Chat => FocusedArea::Input,
            FocusedArea::Input => if self.show_settings { FocusedArea::Settings } else { FocusedArea::SessionList },
            FocusedArea::Settings => FocusedArea::SessionList,
        };
        // Reset input mode when leaving input area
        if !matches!(self.focused_area, FocusedArea::Input) {
            self.input_mode = InputMode::Viewing;
        }
    }

    pub fn select_current_session(&mut self) {
        // Focus on the chat area for the currently selected session
        self.focused_area = FocusedArea::Chat;
        self.scroll_to_bottom(); // Scroll to bottom to show most recent messages, like a chat app
    }

    pub fn handle_directional_input(&mut self, direction: Direction) {
        match self.focused_area {
            FocusedArea::SessionList => {
                match direction {
                    Direction::Up => self.previous_session(), // Move up in visual list (to newer session)
                    Direction::Down => self.next_session(), // Move down in visual list (to older session)
                    Direction::Right => {
                        // Select current session and move to chat
                        self.select_current_session();
                    }
                    Direction::Left => {
                        // Could implement session deletion or other actions
                    }
                }
            }
            FocusedArea::Chat => {
                match direction {
                    Direction::Up => self.scroll_up(),
                    Direction::Down => self.scroll_down(),
                    Direction::Left => self.focused_area = FocusedArea::SessionList,
                    Direction::Right => self.focused_area = FocusedArea::Input,
                }
            }
            FocusedArea::Input => {
                // Directional keys in input area only work when not editing
                if matches!(self.input_mode, InputMode::Viewing) {
                    match direction {
                        Direction::Up => self.focused_area = FocusedArea::Chat,
                        Direction::Down => {
                            // Could scroll to bottom of chat or other action
                        }
                        Direction::Left => self.focused_area = FocusedArea::Chat,
                        Direction::Right => {
                            // Could implement other actions
                        }
                    }
                }
            }
            FocusedArea::Settings => {
                // Only handle navigation when not editing a setting
                if self.settings_editing_key.is_none() {
                    match direction {
                        Direction::Up => self.settings_previous_item(4), // We have 4 settings
                        Direction::Down => self.settings_next_item(4),
                        Direction::Left => self.focused_area = FocusedArea::SessionList,
                        Direction::Right => {
                            // Could implement other actions
                        }
                    }
                }
            }
        }
    }

    pub fn enter_input_edit_mode(&mut self) {
        if matches!(self.focused_area, FocusedArea::Input) {
            self.input_mode = InputMode::Editing;
        }
    }

    pub fn exit_input_edit_mode(&mut self) {
        self.input_mode = InputMode::Viewing;
    }

    pub fn is_input_editing(&self) -> bool {
        (matches!(self.focused_area, FocusedArea::Input) && matches!(self.input_mode, InputMode::Editing)) ||
        (matches!(self.focused_area, FocusedArea::Settings) && (self.settings_editing_key.is_some() || self.settings_provider_selecting))
    }

    pub fn update_areas(&mut self, session_list: Rect, chat: Rect, input: Rect) {
        self.session_list_area = session_list;
        self.chat_area = chat;
        self.input_area_rect = input;
    }

    pub fn handle_mouse_click(&mut self, x: u16, y: u16) {
        // Check if click is in session list area
        if self.session_list_area.x <= x 
            && x < self.session_list_area.x + self.session_list_area.width
            && self.session_list_area.y <= y 
            && y < self.session_list_area.y + self.session_list_area.height {
            
            // Calculate which session was clicked (accounting for borders)
            let relative_y = y.saturating_sub(self.session_list_area.y + 1); // +1 for top border
            let displayed_index = relative_y as usize + self.session_scroll_offset;
            
            // Convert from displayed index to actual session index
            if displayed_index < self.sessions.len() {
                let actual_session_index = displayed_index;
                // Use session ID to ensure we're selecting the right session
                if let Some(session) = self.sessions.get(actual_session_index) {
                    let session_id = session.id.clone();
                    self.switch_to_session_by_id(&session_id);
                }
            }
        }
        // Check if click is in input area
        else if self.input_area_rect.x <= x 
            && x < self.input_area_rect.x + self.input_area_rect.width
            && self.input_area_rect.y <= y 
            && y < self.input_area_rect.y + self.input_area_rect.height {
            
            self.focused_area = FocusedArea::Input;
        }
        // Check if click is in chat area
        else if self.chat_area.x <= x 
            && x < self.chat_area.x + self.chat_area.width
            && self.chat_area.y <= y 
            && y < self.chat_area.y + self.chat_area.height {
            
            self.focused_area = FocusedArea::Chat;
        }
    }

    pub fn handle_mouse_scroll(&mut self, x: u16, y: u16, direction: ScrollDirection) {
        // Check if scroll is in session list area
        if self.session_list_area.x <= x 
            && x < self.session_list_area.x + self.session_list_area.width
            && self.session_list_area.y <= y 
            && y < self.session_list_area.y + self.session_list_area.height {
            
            match direction {
                ScrollDirection::Up => self.session_scroll_up(),
                ScrollDirection::Down => self.session_scroll_down(),
            }
        }
        // Check if scroll is in chat area
        else if self.chat_area.x <= x 
            && x < self.chat_area.x + self.chat_area.width
            && self.chat_area.y <= y 
            && y < self.chat_area.y + self.chat_area.height {
            
            match direction {
                ScrollDirection::Up => self.scroll_up(),
                ScrollDirection::Down => self.scroll_down(),
            }
        }
    }

    pub fn toggle_settings(&mut self) {
        self.show_settings = !self.show_settings;
        if self.show_settings {
            self.focused_area = FocusedArea::Settings;
        } else if matches!(self.focused_area, FocusedArea::Settings) {
            self.focused_area = FocusedArea::SessionList;
        }
    }

    pub fn settings_next_item(&mut self, max_items: usize) {
        if max_items > 0 {
            self.settings_selected_index = (self.settings_selected_index + 1) % max_items;
        }
    }

    pub fn settings_previous_item(&mut self, max_items: usize) {
        if max_items > 0 {
            self.settings_selected_index = if self.settings_selected_index == 0 {
                max_items - 1
            } else {
                self.settings_selected_index - 1
            };
        }
    }

    pub fn start_editing_setting(&mut self, key: String, current_value: String) {
        self.settings_editing_key = Some(key);
        self.settings_input_area = TextArea::default();
        self.settings_input_area.set_placeholder_text("Enter new value...");
        // Pre-populate with current value if not sensitive
        if !current_value.starts_with("*") {
            self.settings_input_area.insert_str(current_value);
        }
        self.input_mode = InputMode::Editing;
    }

    pub fn cancel_settings_edit(&mut self) {
        self.settings_editing_key = None;
        self.settings_input_area = TextArea::default();
        self.settings_input_area.set_placeholder_text("Enter new value...");
        self.input_mode = InputMode::Viewing;
    }

    pub fn get_settings_input_text(&self) -> String {
        self.settings_input_area.lines().join("\n")
    }


    pub fn start_provider_selection_with_current<R: crate::config::repository::ConfigRepository>(&mut self, repo: &R) {
        self.settings_provider_selecting = true;
        self.input_mode = InputMode::Editing;
        
        // Set current selection based on saved provider
        if let Ok(config) = crate::config::service::config_service::fetch_by_key(repo, "provider_key") {
            self.settings_provider_selected_index = match config.value.as_str() {
                "openapi" => 0,
                "claude" => 1,
                _ => 1, // default to claude
            };
        }
    }

    pub fn cancel_provider_selection(&mut self) {
        self.settings_provider_selecting = false;
        self.input_mode = InputMode::Viewing;
    }

    pub fn provider_selection_next(&mut self) {
        self.settings_provider_selected_index = (self.settings_provider_selected_index + 1) % 2; // 2 providers: OpenAI and Claude
    }

    pub fn provider_selection_previous(&mut self) {
        self.settings_provider_selected_index = if self.settings_provider_selected_index == 0 { 1 } else { 0 };
    }

    pub fn get_selected_provider(&self) -> &'static str {
        match self.settings_provider_selected_index {
            0 => "openapi",
            1 => "claude",
            _ => "claude", // default
        }
    }


    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    // Selection mode methods
    pub fn enter_visual_mode(&mut self) {
        if matches!(self.focused_area, FocusedArea::Chat) && !self.is_input_editing() {
            self.selection_mode = SelectionMode::Visual;
            // Start with cursor-only mode (no selection)
            self.selection = None;
            self.update_chat_content_cache();
        }
    }

    pub fn enter_visual_line_mode(&mut self) {
        if matches!(self.focused_area, FocusedArea::Chat) && !self.is_input_editing() {
            self.selection_mode = SelectionMode::VisualLine;
            // Start line selection at current cursor position
            self.selection = Some(TextSelection {
                start: CursorPosition { line: self.cursor_position.line, column: 0 },
                end: CursorPosition { 
                    line: self.cursor_position.line, 
                    column: self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0) 
                },
            });
            self.update_chat_content_cache();
        }
    }

    pub fn exit_visual_mode(&mut self) {
        self.selection_mode = SelectionMode::None;
        self.selection = None;
    }

    pub fn is_in_visual_mode(&self) -> bool {
        matches!(self.selection_mode, SelectionMode::Visual | SelectionMode::VisualLine)
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        if !self.is_in_visual_mode() || self.chat_content_lines.is_empty() {
            return;
        }

        let max_line = self.chat_content_lines.len().saturating_sub(1);
        let old_position = self.cursor_position.clone();
        
        match direction {
            Direction::Up => {
                if self.cursor_position.line > 0 {
                    self.cursor_position.line -= 1;
                    // Clamp column to line length (allow cursor at end of line)
                    let line_len = self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    self.cursor_position.column = self.cursor_position.column.min(line_len);
                }
            }
            Direction::Down => {
                if self.cursor_position.line < max_line {
                    self.cursor_position.line += 1;
                    // Clamp column to line length (allow cursor at end of line)
                    let line_len = self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    self.cursor_position.column = self.cursor_position.column.min(line_len);
                }
            }
            Direction::Left => {
                if self.cursor_position.column > 0 {
                    self.cursor_position.column -= 1;
                } else if self.cursor_position.line > 0 {
                    // Move to end of previous line
                    self.cursor_position.line -= 1;
                    self.cursor_position.column = self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                }
            }
            Direction::Right => {
                let current_line_len = self.chat_content_lines.get(self.cursor_position.line)
                    .map(|l| l.len())
                    .unwrap_or(0);
                    
                if self.cursor_position.column < current_line_len {
                    self.cursor_position.column += 1;
                } else if self.cursor_position.line < max_line {
                    // Move to beginning of next line
                    self.cursor_position.line += 1;
                    self.cursor_position.column = 0;
                }
            }
        }

        // Update selection based on mode
        match self.selection_mode {
            SelectionMode::Visual => {
                // In visual mode, start selection on first movement if not already started
                if self.selection.is_none() {
                    self.selection = Some(TextSelection {
                        start: old_position,
                        end: self.cursor_position.clone(),
                    });
                } else if let Some(ref mut selection) = self.selection {
                    selection.end = self.cursor_position.clone();
                }
            }
            SelectionMode::VisualLine => {
                // In visual line mode, always select full lines
                if let Some(ref mut selection) = self.selection {
                    // Set start to beginning of line and end to end of line
                    selection.start.column = 0;
                    let line_len = self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    selection.end = CursorPosition { 
                        line: self.cursor_position.line, 
                        column: line_len 
                    };
                } else {
                    // Initialize selection if not present
                    let line_len = self.chat_content_lines.get(self.cursor_position.line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    self.selection = Some(TextSelection {
                        start: CursorPosition { line: self.cursor_position.line, column: 0 },
                        end: CursorPosition { line: self.cursor_position.line, column: line_len },
                    });
                }
            }
            SelectionMode::None => {}
        }
    }

    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;
        
        if self.chat_content_lines.is_empty() {
            return None;
        }

        let start = &selection.start;
        let end = &selection.end;
        
        // Ensure start is before end
        let (start, end) = if start.line < end.line || (start.line == end.line && start.column <= end.column) {
            (start, end)
        } else {
            (end, start)
        };

        let mut selected_text = String::new();
        
        if start.line == end.line {
            // Single line selection
            if let Some(line) = self.chat_content_lines.get(start.line) {
                let start_col = start.column.min(line.len());
                let end_col = end.column.min(line.len());
                if start_col < end_col {
                    selected_text = line[start_col..end_col].to_string();
                }
            }
        } else {
            // Multi-line selection
            for line_idx in start.line..=end.line {
                if let Some(line) = self.chat_content_lines.get(line_idx) {
                    if line_idx == start.line {
                        // First line: from start column to end
                        let start_col = start.column.min(line.len());
                        selected_text.push_str(&line[start_col..]);
                    } else if line_idx == end.line {
                        // Last line: from beginning to end column
                        let end_col = end.column.min(line.len());
                        selected_text.push_str(&line[..end_col]);
                    } else {
                        // Middle lines: entire line
                        selected_text.push_str(line);
                    }
                    
                    // Add newline except for the last line
                    if line_idx < end.line {
                        selected_text.push('\n');
                    }
                }
            }
        }

        if selected_text.is_empty() {
            None
        } else {
            Some(selected_text)
        }
    }

    pub fn copy_selection_to_clipboard(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(text) = self.get_selected_text() {
            // Use arboard crate for clipboard access
            use arboard::Clipboard;
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(text)?;
            Ok(())
        } else {
            Err("No text selected".into())
        }
    }

    pub fn update_chat_content_cache(&mut self) {
        self.chat_content_lines.clear();
        
        // Clone the messages to avoid borrowing conflicts
        let messages = if let Some(session) = self.current_session() {
            session.messages.clone()
        } else {
            Vec::new()
        };
        
        let filtered_messages: Vec<_> = messages
            .iter()
            .filter(|msg| msg.role != Role::System)
            .collect();
        
        for (i, message) in filtered_messages.iter().enumerate() {
            if i > 0 {
                self.chat_content_lines.push(String::new()); // Empty line between messages
            }
            
            // Add role prefix
            let role_prefix = match message.role {
                Role::User => "👤 You:",
                Role::Assistant => "🤖 AI:",
                Role::System => "⚙️  System:",
            };
            self.chat_content_lines.push(role_prefix.to_string());
            
            // Add message content lines
            for line in message.content.lines() {
                self.chat_content_lines.push(line.to_string());
            }
            
            self.chat_content_lines.push(String::new()); // Empty line after message
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::session::Session;

    #[test]
    fn test_empty_sessions_handling() {
        let mut app = App::new();
        app.set_sessions(vec![]);
        
        assert_eq!(app.current_session(), None);
        assert_eq!(app.session_count(), 0);
        assert!(!app.has_sessions());
        assert_eq!(app.current_session_index, 0); // Should be 0 even for empty
    }
    
    #[test]
    fn test_session_index_bounds() {
        let mut app = App::new();
        let sessions = vec![
            Session::new_temporary(),
            Session::new_temporary(),
        ];
        
        app.current_session_index = 5; // Invalid index
        app.set_sessions(sessions);
        
        assert_eq!(app.current_session_index, 1); // Should be clamped to last valid
        assert!(app.current_session().is_some());
    }
    
    #[test]
    fn test_session_navigation_empty() {
        let mut app = App::new();
        app.set_sessions(vec![]);
        
        let initial_index = app.current_session_index;
        
        app.next_session();
        assert_eq!(app.current_session_index, initial_index);
        
        app.previous_session();
        assert_eq!(app.current_session_index, initial_index);
    }
    
    #[test]
    fn test_session_navigation_single() {
        let mut app = App::new();
        app.set_sessions(vec![Session::new_temporary()]);
        
        let initial_index = app.current_session_index;
        
        app.next_session();
        assert_eq!(app.current_session_index, initial_index); // Should not change
        
        app.previous_session();
        assert_eq!(app.current_session_index, initial_index); // Should not change
    }
    
    #[test]
    fn test_session_navigation_multiple() {
        let mut app = App::new();
        let sessions = vec![
            Session::new_temporary(),
            Session::new_temporary(),
            Session::new_temporary(),
        ];
        app.set_sessions(sessions);
        app.current_session_index = 1; // Start in middle
        
        // Test next
        app.next_session();
        assert_eq!(app.current_session_index, 2);
        
        // Test wrap around
        app.next_session();
        assert_eq!(app.current_session_index, 0);
        
        // Test previous
        app.previous_session();
        assert_eq!(app.current_session_index, 2);
        
        // Test previous again
        app.previous_session();
        assert_eq!(app.current_session_index, 1);
    }
    
    #[test]
    fn test_session_removal_edge_cases() {
        let mut app = App::new();
        let mut session1 = Session::new_temporary();
        session1.id = "session1".to_string();
        let mut session2 = Session::new_temporary();
        session2.id = "session2".to_string();
        
        app.set_sessions(vec![session1, session2]);
        app.current_session_index = 1;
        
        // Remove current session
        assert!(app.remove_session("session2"));
        assert_eq!(app.current_session_index, 0);
        assert_eq!(app.session_count(), 1);
        
        // Remove last session - should add a new temporary one
        assert!(app.remove_session("session1"));
        assert_eq!(app.session_count(), 1); // Should add temporary session
        assert!(app.current_session().is_some());
        assert!(app.current_session().unwrap().temporary);
    }
    
    #[test]
    fn test_add_message_to_empty_sessions() {
        let mut app = App::new();
        app.set_sessions(vec![]);
        
        // Should create a new session
        app.add_message_to_current_session("Hello".to_string(), Role::User);
        
        assert_eq!(app.session_count(), 1);
        assert!(app.current_session().is_some());
        assert_eq!(app.current_session().unwrap().messages.len(), 1);
    }
    
    #[test]
    fn test_ensure_valid_session_index() {
        let mut app = App::new();
        let sessions = vec![Session::new_temporary(), Session::new_temporary()];
        app.set_sessions(sessions);
        
        // Manually corrupt the index
        app.current_session_index = 10;
        app.ensure_valid_session_index();
        
        assert_eq!(app.current_session_index, 1); // Should be last valid index
    }
    
    #[test]
    fn test_session_switching_by_id() {
        let mut app = App::new();
        let mut session1 = Session::new_temporary();
        session1.id = "test1".to_string();
        let mut session2 = Session::new_temporary();
        session2.id = "test2".to_string();
        
        app.set_sessions(vec![session1, session2]);
        app.current_session_index = 0;
        
        // Switch to existing session
        assert!(app.switch_to_session_by_id("test2"));
        assert_eq!(app.current_session_index, 1);
        
        // Try to switch to non-existent session
        assert!(!app.switch_to_session_by_id("nonexistent"));
        assert_eq!(app.current_session_index, 1); // Should remain unchanged
    }

    #[test]
    fn test_session_selection_with_enter() {
        let mut app = App::new();
        
        // Set up sessions
        let session1 = Session::new_temporary();
        let session2 = Session::new_temporary();
        app.set_sessions(vec![session1, session2]);
        
        // Start in session list
        app.focused_area = FocusedArea::SessionList;
        app.current_session_index = 1;
        app.scroll_offset = 10; // Set some scroll offset
        
        // Select current session
        app.select_current_session();
        
        // Should now be focused on chat area
        assert_eq!(app.focused_area, FocusedArea::Chat);
        // Scroll should be set to bottom (usize::MAX gets clamped by UI)
        assert_eq!(app.scroll_offset, usize::MAX);
        // Session index should remain the same
        assert_eq!(app.current_session_index, 1);
    }
} 