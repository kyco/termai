use crate::session::model::session::Session;
use crate::session::model::message::Message;
use crate::llm::common::model::role::Role;
use std::collections::HashMap;
use tui_textarea::TextArea;
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq)]
pub enum FocusedArea {
    SessionList,
    Chat,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Viewing,
    Editing,
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
    // Area tracking for mouse interactions
    pub session_list_area: Rect,
    pub chat_area: Rect,
    pub input_area_rect: Rect,
}

impl Default for App {
    fn default() -> Self {
        let mut input_area = TextArea::default();
        input_area.set_placeholder_text("Type your message here...");
        
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
            session_list_area: Rect::default(),
            chat_area: Rect::default(),
            input_area_rect: Rect::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_session(&self) -> Option<&Session> {
        self.sessions.get(self.current_session_index)
    }

    pub fn current_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions.get_mut(self.current_session_index)
    }

    pub fn add_message_to_current_session(&mut self, content: String, role: Role) {
        if let Some(session) = self.current_session_mut() {
            session.add_raw_message(content, role);
        }
    }

    pub fn set_sessions(&mut self, sessions: Vec<Session>) {
        self.sessions = sessions;
        if self.current_session_index >= self.sessions.len() {
            self.current_session_index = if self.sessions.is_empty() { 0 } else { self.sessions.len() - 1 };
        }
    }

    pub fn switch_to_session(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.current_session_index = index;
            self.scroll_offset = 0;
        }
    }

    pub fn next_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session_index = (self.current_session_index + 1) % self.sessions.len();
            self.scroll_offset = 0;
        }
    }

    pub fn previous_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session_index = if self.current_session_index == 0 {
                self.sessions.len() - 1
            } else {
                self.current_session_index - 1
            };
            self.scroll_offset = 0;
        }
    }

    pub fn create_new_session(&mut self) {
        let new_session = Session::new_temporary();
        self.sessions.push(new_session);
        self.current_session_index = self.sessions.len() - 1;
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
        self.scroll_offset += 1;
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
            FocusedArea::Input => FocusedArea::SessionList,
        };
        // Reset input mode when leaving input area
        if !matches!(self.focused_area, FocusedArea::Input) {
            self.input_mode = InputMode::Viewing;
        }
    }

    pub fn handle_directional_input(&mut self, direction: Direction) {
        match self.focused_area {
            FocusedArea::SessionList => {
                match direction {
                    Direction::Up => self.previous_session(),
                    Direction::Down => self.next_session(),
                    Direction::Right => {
                        // Select current session and move to chat
                        self.focused_area = FocusedArea::Chat;
                        self.scroll_offset = 0;
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
        matches!(self.focused_area, FocusedArea::Input) && matches!(self.input_mode, InputMode::Editing)
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
            let session_index = relative_y as usize + self.session_scroll_offset;
            
            if session_index < self.sessions.len() {
                self.current_session_index = session_index;
                self.scroll_offset = 0; // Reset chat scroll when switching sessions
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