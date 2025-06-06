use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    sender: mpsc::UnboundedSender<AppEvent>,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handler = {
            let sender = sender.clone();
            tokio::spawn(async move {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| Duration::from_secs(0));

                    if event::poll(timeout).unwrap() {
                        match event::read().unwrap() {
                            Event::Key(key_event) => {
                                if let Err(_) = sender.send(AppEvent::Key(key_event)) {
                                    break;
                                }
                            }
                            Event::Mouse(mouse_event) => {
                                if let Err(_) = sender.send(AppEvent::Mouse(mouse_event)) {
                                    break;
                                }
                            }
                            Event::Resize(width, height) => {
                                if let Err(_) = sender.send(AppEvent::Resize(width, height)) {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if let Err(_) = sender.send(AppEvent::Tick) {
                            break;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.handler.abort();
    }
}

pub fn handle_key_event(key_event: KeyEvent) -> Option<KeyAction> {
    match key_event.code {
        KeyCode::Char('q') if key_event.modifiers.contains(KeyModifiers::ALT) => {
            Some(KeyAction::Quit)
        }
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(KeyAction::Quit)
        }
        KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(KeyAction::NewSession)
        }
        KeyCode::Tab => Some(KeyAction::CycleFocus),
        KeyCode::Enter => Some(KeyAction::EnterEditMode),
        KeyCode::Esc => Some(KeyAction::ExitEditMode),
        KeyCode::Up => Some(KeyAction::DirectionalMove(Direction::Up)),
        KeyCode::Down => Some(KeyAction::DirectionalMove(Direction::Down)),
        KeyCode::Left => Some(KeyAction::DirectionalMove(Direction::Left)),
        KeyCode::Right => Some(KeyAction::DirectionalMove(Direction::Right)),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    Quit,
    CycleFocus,
    EnterEditMode,
    ExitEditMode,
    DirectionalMove(Direction),
    NewSession,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    ScrollUp(u16, u16),
    ScrollDown(u16, u16),
    Click(u16, u16),
    FocusInput(u16, u16),
    SelectSession(u16, u16),
}

pub fn handle_mouse_event(mouse_event: MouseEvent) -> Option<MouseAction> {
    match mouse_event.kind {
        MouseEventKind::ScrollUp => {
            Some(MouseAction::ScrollUp(mouse_event.column, mouse_event.row))
        }
        MouseEventKind::ScrollDown => {
            Some(MouseAction::ScrollDown(mouse_event.column, mouse_event.row))
        }
        MouseEventKind::Down(MouseButton::Left) => {
            Some(MouseAction::Click(mouse_event.column, mouse_event.row))
        }
        _ => None,
    }
} 