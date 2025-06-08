use crate::ui::tui::app::{App, FocusedArea, InputMode, ScrollDirection, Direction};
use crate::ui::tui::events::{AppEvent, EventHandler, KeyAction, MouseAction, handle_key_event, handle_mouse_event};
use crate::ui::tui::ui;
use crate::ui::tui::chat;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::llm::common::model::role::Role;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;
use tui_textarea::Input;

// Helper function to get setting value with masking for sensitive data

// Helper function to get actual setting value for editing
fn get_setting_actual_value<R: ConfigRepository>(repo: &R, key: &str) -> String {
    match config_service::fetch_by_key(repo, key) {
        Ok(config) => config.value,
        Err(_) => "".to_string(),
    }
}

pub async fn run_tui<R, SR, MR>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository,
    SR: SessionRepository,
    MR: MessageRepository,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();
    
    // Load existing sessions
    match fetch_all_sessions_for_ui(session_repository, message_repository) {
        Ok(sessions) => {
            if !sessions.is_empty() {
                app.set_sessions(sessions);
            }
        }
        Err(_) => {
            // Keep default temporary session
        }
    }

    // Create event handler
    let mut events = EventHandler::new(Duration::from_millis(250));

    // Main event loop
    loop {
        // Check if current session needs refresh and refresh it
        if app.session_needs_refresh {
            app.refresh_current_session(session_repository, message_repository);
        }

        terminal.draw(|f| ui::draw(f, &mut app, Some(repo)))?;

        if app.should_quit {
            break;
        }

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key_event) => {
                    if let Some(action) = handle_key_event(key_event) {
                        match action {
                            KeyAction::Quit => app.quit(),
                            KeyAction::CycleFocus => {
                                app.cycle_focus();
                            }
                            KeyAction::EnterEditMode => {
                                if matches!(app.focused_area, FocusedArea::Input) && matches!(app.input_mode, InputMode::Viewing) {
                                    app.enter_input_edit_mode();
                                } else if matches!(app.focused_area, FocusedArea::Settings) && app.settings_editing_key.is_none() {
                                    // Start editing the selected setting
                                    let settings_items = vec![
                                        ("Chat GPT API Key", "chat_gpt_api_key"),
                                        ("Claude API Key", "claude_api_key"),
                                        ("Provider", "provider_key"),
                                        ("Redactions", "redacted"),
                                    ];
                                    if let Some((_, key)) = settings_items.get(app.settings_selected_index) {
                                        let current_value = get_setting_actual_value(repo, key);
                                        app.start_editing_setting(key.to_string(), current_value);
                                    }
                                } else if app.is_input_editing() {
                                    if matches!(app.focused_area, FocusedArea::Settings) {
                                        // Save settings value
                                        let new_value = app.get_settings_input_text().trim().to_string();
                                        if let Some(key) = &app.settings_editing_key.clone() {
                                            if let Err(e) = config_service::write_config(repo, key, &new_value) {
                                                app.set_error(Some(format!("Failed to save setting: {}", e)));
                                            }
                                        }
                                        app.cancel_settings_edit();
                                    } else {
                                        // Send message when Enter is pressed while editing
                                        let message = app.get_input_text().trim().to_string();
                                        if !message.is_empty() {
                                            // Immediately add user message to current session and update UI
                                            app.add_message_to_current_session(message.clone(), Role::User);
                                            app.clear_input();
                                            app.scroll_to_bottom(); // Auto-scroll to show new message
                                            app.set_loading(true);
                                            
                                            // Force a redraw to show the user message immediately
                                            terminal.draw(|f| ui::draw(f, &mut app, Some(repo)))?;
                                            
                                            // Now do the API call
                                            let chat_result = if let Some(session) = app.current_session_mut() {
                                                let was_temporary = session.temporary;
                                                let result = chat::send_message_async(
                                                    repo,
                                                    session_repository,
                                                    message_repository,
                                                    session,
                                                    message,
                                                ).await;
                                                
                                                // Check if we need to convert temporary session
                                                let should_convert = was_temporary && session.messages.len() >= 2;
                                                (result, should_convert)
                                            } else {
                                                (Err(anyhow::anyhow!("No current session")), false)
                                            };
                                            
                                            match chat_result.0 {
                                                Ok(_) => {
                                                    app.set_error(None);
                                                    app.scroll_to_bottom(); // Auto-scroll to show AI response
                                                    
                                                    // If this was a temporary session, convert it to permanent
                                                    if chat_result.1 {
                                                        if let Some(session) = app.current_session_mut() {
                                                            match chat::convert_temporary_session_to_permanent(
                                                                repo,
                                                                session_repository,
                                                                message_repository,
                                                                session,
                                                            ).await {
                                                                Ok(_) => {
                                                                    // Session has been converted to permanent with new title
                                                                    // The UI will update on next draw
                                                                }
                                                                Err(e) => {
                                                                    // If title generation fails, just log it but don't show error to user
                                                                    eprintln!("Failed to generate session title: {}", e);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    app.set_error(Some(format!("Error: {}", e)));
                                                }
                                            }
                                            
                                            app.set_loading(false);
                                        }
                                    }
                                }
                            }
                            KeyAction::ExitEditMode => {
                                if app.show_help {
                                    app.toggle_help();
                                } else if app.error_message.is_some() {
                                    app.set_error(None);
                                } else if matches!(app.focused_area, FocusedArea::Settings) && app.settings_editing_key.is_some() {
                                    app.cancel_settings_edit();
                                } else {
                                    app.exit_input_edit_mode();
                                }
                            }
                            KeyAction::DirectionalMove(direction) => {
                                let app_direction = match direction {
                                    crate::ui::tui::events::Direction::Up => Direction::Up,
                                    crate::ui::tui::events::Direction::Down => Direction::Down,
                                    crate::ui::tui::events::Direction::Left => Direction::Left,
                                    crate::ui::tui::events::Direction::Right => Direction::Right,
                                };
                                app.handle_directional_input(app_direction);
                            }
                            KeyAction::NewSession => {
                                app.create_new_session();
                            }
                            KeyAction::ToggleSettings => {
                                app.toggle_settings();
                            }
                            KeyAction::ToggleHelp => {
                                app.toggle_help();
                            }
                        }
                    } else {
                        // Handle other key events for input when editing
                        if app.is_input_editing() {
                            if matches!(app.focused_area, FocusedArea::Settings) && app.settings_editing_key.is_some() {
                                app.settings_input_area.input(Input::from(key_event));
                            } else {
                                app.input_area.input(Input::from(key_event));
                            }
                        }
                    }
                }
                AppEvent::Mouse(mouse_event) => {
                    if let Some(action) = handle_mouse_event(mouse_event) {
                        match action {
                            MouseAction::ScrollUp(x, y) => {
                                app.handle_mouse_scroll(x, y, ScrollDirection::Up);
                            }
                            MouseAction::ScrollDown(x, y) => {
                                app.handle_mouse_scroll(x, y, ScrollDirection::Down);
                            }
                            MouseAction::Click(x, y) => {
                                app.handle_mouse_click(x, y);
                            }
                            MouseAction::FocusInput(x, y) => {
                                app.handle_mouse_click(x, y);
                            }
                            MouseAction::SelectSession(x, y) => {
                                app.handle_mouse_click(x, y);
                            }
                        }
                    }
                }
                AppEvent::Resize(_, _) => {
                    // Terminal was resized, will be handled on next draw
                }
                AppEvent::Tick => {
                    // Regular tick for animations, etc.
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// Helper function to fetch sessions for UI
fn fetch_all_sessions_for_ui<SR: SessionRepository, MR: MessageRepository>(
    session_repo: &SR,
    message_repository: &MR,
) -> Result<Vec<crate::session::model::session::Session>> {
    let session_entities = session_repo.fetch_all_sessions().unwrap_or_default();
    let mut sessions = Vec::new();
    
    for entity in session_entities {
        let mut session = crate::session::model::session::Session::from(&entity);
        let messages = message_repository
            .fetch_messages_for_session(&session.id)
            .unwrap_or_default()
            .iter()
            .map(|m| crate::session::model::message::Message::from(m))
            .collect();
        session = session.copy_with_messages(messages);
        sessions.push(session);
    }
    
    Ok(sessions)
} 