use crate::repository::Repository;
use crossterm::event::{KeyCode, KeyModifiers};

pub enum Focus {
    Sidebar,
    Main,
    Input,
}

pub struct App<R>
where
    R: Repository,
{
    pub input: String,
    pub messages: Vec<String>,
    repo: R,
    pub focus: Focus,
}

impl<R> App<R>
where
    R: Repository,
    R::Error: std::error::Error,
{
    pub fn new(repo: R) -> Result<Self, R::Error> {
        let messages = repo.get_messages()?;
        Ok(Self {
            input: String::new(),
            messages,
            repo,
            focus: Focus::Input,
        })
    }

    pub fn on_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<(), R::Error> {
        if modifiers.contains(KeyModifiers::CONTROL) && modifiers.contains(KeyModifiers::SHIFT) {
            match key {
                KeyCode::Left => self.switch_focus_left(),
                KeyCode::Right => self.switch_focus_right(),
                _ => {}
            }
        } else {
            match self.focus {
                Focus::Sidebar => {
                    // Handle sidebar interactions if needed
                }
                Focus::Main => {
                    // Handle main area interactions if needed
                }
                Focus::Input => match key {
                    KeyCode::Char(c) => self.input.push(c),
                    KeyCode::Backspace => {
                        self.input.pop();
                    }
                    KeyCode::Enter => {
                        if !self.input.is_empty() {
                            self.repo.add_message(self.input.clone())?;
                            self.messages.push(self.input.clone());
                            self.input.clear();
                        }
                    }
                    _ => {}
                },
            }
        }
        Ok(())
    }

    fn switch_focus_left(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Input,
            Focus::Main => Focus::Sidebar,
            Focus::Input => Focus::Main,
        };
    }

    fn switch_focus_right(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Main,
            Focus::Main => Focus::Input,
            Focus::Input => Focus::Sidebar,
        };
    }
}
