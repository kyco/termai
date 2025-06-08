use crate::ui::tui::app::{App, FocusedArea, InputMode};
use crate::config::service::config_service;
use crate::llm::common::model::role::Role;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

pub fn draw<R: crate::config::repository::ConfigRepository>(f: &mut Frame, app: &mut App, config_repo: Option<&R>) {
    // Main layout with footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

    if app.show_settings {
        draw_settings_view(f, app, main_chunks[0], config_repo);
    } else {
        // Horizontal split for session list and chat area
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(0)])
            .split(main_chunks[0]);

        // Draw session list on the left
        draw_session_list(f, app, chunks[0]);

        // Draw main chat area on the right
        let chat_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(chunks[1]);

        draw_chat_area(f, app, chat_chunks[0]);
        draw_input_area(f, app, chat_chunks[1]);

        // Update app areas for mouse interaction
        app.update_areas(chunks[0], chat_chunks[0], chat_chunks[1]);
    }

    // Draw footer bar
    draw_footer(f, app, main_chunks[1]);

    // Draw loading indicator if needed
    if app.is_loading {
        draw_loading_popup(f);
    }

    // Draw error message if any
    if let Some(ref error) = app.error_message {
        draw_error_popup(f, error);
    }
}

fn draw_session_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.focused_area, FocusedArea::SessionList);
    
    let sessions: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let style = if i == app.current_session_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let message_count = session.messages.len();
            let display_name = if session.name == "temporary" {
                format!("🔄 Temp ({})", message_count)
            } else {
                format!("💬 {} ({})", session.name, message_count)
            };

            ListItem::new(display_name).style(style)
        })
        .collect();

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Blue)
    };

    let title = if is_focused {
        "Sessions (Focused)"
    } else {
        "Sessions"
    };

    let sessions_list = List::new(sessions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(app.current_session_index));

    f.render_stateful_widget(sessions_list, area, &mut list_state);

    // Draw scrollbar for sessions if needed
    if app.sessions.len() > area.height as usize - 2 {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::new(app.sessions.len())
            .position(app.session_scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn draw_chat_area(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = matches!(app.focused_area, FocusedArea::Chat);
    
    // Create the UI area first
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Chat") // Simplified title for now
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .border_style(if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Blue)
        });

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Get session messages first, completely separate from any borrowing
    let messages = app.current_session()
        .map(|session| session.messages.clone())
        .unwrap_or_default();
    
    let filtered_messages: Vec<_> = messages
        .iter()
        .filter(|msg| msg.role != Role::System)
        .collect();
    
    if !filtered_messages.is_empty() {
        // Create all chat content as a single Text widget
        let mut chat_content = Text::default();
        for (i, message) in filtered_messages.iter().enumerate() {
            if i > 0 {
                chat_content.extend(Text::from("\n"));
            }
            chat_content.extend(format_message(message));
        }
        
        // Calculate actual rendered height considering text wrapping
        let available_width = inner_area.width as usize;
        let mut total_wrapped_lines = 0;
        
        for line in &chat_content.lines {
            // Calculate the display width of the line (considering unicode characters)
            let line_text = line.spans.iter().map(|span| span.content.as_ref()).collect::<String>();
            let line_width = unicode_width::UnicodeWidthStr::width(line_text.as_str());
            
            if line_width == 0 {
                // Empty line
                total_wrapped_lines += 1;
            } else {
                // Calculate how many display lines this text line will occupy when wrapped
                let wrapped_line_count = (line_width + available_width - 1) / available_width.max(1);
                total_wrapped_lines += wrapped_line_count.max(1);
            }
        }
        
        let available_height = inner_area.height as usize;
        
        // Clamp scroll position based on actual wrapped content height
        app.clamp_scroll_to_content_lines(total_wrapped_lines, available_height);
        
        // Use Paragraph's scroll feature for proper line-based scrolling
        let paragraph = Paragraph::new(chat_content)
            .wrap(Wrap { trim: true })
            .scroll((app.scroll_offset as u16, 0));
        f.render_widget(paragraph, inner_area);

        // Draw scrollbar if needed (based on actual wrapped content lines)
        if total_wrapped_lines > available_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let max_scroll = total_wrapped_lines.saturating_sub(available_height);
            let mut scrollbar_state = ScrollbarState::new(max_scroll.max(1))
                .position(app.scroll_offset.min(max_scroll));
            f.render_stateful_widget(
                scrollbar,
                inner_area,
                &mut scrollbar_state,
            );
        }
    } else {
        let welcome_text = Text::from(vec![
            Line::from(vec![
                Span::styled("Welcome to TermAI! 🤖", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("Start a conversation by typing in the input area below."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Controls:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from("• Tab: Cycle through areas (Sessions → Chat → Input)"),
            Line::from("• ↑↓←→: Navigate within focused area"),
            Line::from("• Enter: Edit input (when focused) or send message"),
            Line::from("• Esc: Exit edit mode"),
            Line::from("• Mouse: Click to focus, scroll to navigate"),
        ]);
        
        let paragraph = Paragraph::new(welcome_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner_area);
    }
}

fn draw_input_area(f: &mut Frame, app: &App, area: Rect) {
    let input_focused = matches!(app.focused_area, FocusedArea::Input);
    let is_editing = matches!(app.input_mode, InputMode::Editing);
    
    let border_style = if input_focused {
        if is_editing {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Yellow)
        }
    } else {
        Style::default().fg(Color::Blue)
    };

    let title = if input_focused {
        if is_editing {
            "Input (Editing - Esc to exit)"
        } else {
            "Input (Enter to edit)"
        }
    } else {
        "Input (Tab to focus)"
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Render the text area
    f.render_widget(&app.input_area, inner_area);
}

fn draw_settings_view<R: crate::config::repository::ConfigRepository>(f: &mut Frame, app: &mut App, area: Rect, config_repo: Option<&R>) {
    let is_focused = matches!(app.focused_area, FocusedArea::Settings);
    
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Blue)
    };

    let title = if is_focused {
        "Settings (Focused)"
    } else {
        "Settings"
    };

    // Create settings items
    let settings_items = vec![
        ("Chat GPT API Key", "chat_gpt_api_key"),
        ("Claude API Key", "claude_api_key"),
        ("Provider", "provider_key"),
        ("Redactions", "redacted"),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(i, (display_name, key))| {
            let style = if i == app.settings_selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Get actual values from config
            let value = if let Some(repo) = config_repo {
                match config_service::fetch_by_key(repo, key) {
                    Ok(config) => {
                        if key.contains("api_key") {
                            // Mask API keys for security
                            if config.value.is_empty() {
                                "Not set".to_string()
                            } else {
                                "****".to_string()
                            }
                        } else {
                            config.value
                        }
                    }
                    Err(_) => "Not set".to_string(),
                }
            } else {
                "Config not available".to_string()
            };

            let display_text = format!("{}: {}", display_name, value);
            ListItem::new(display_text).style(style)
        })
        .collect();

    let settings_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    // If editing a setting, split the area to show input
    if app.settings_editing_key.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(area);

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.settings_selected_index));
        f.render_stateful_widget(settings_list, chunks[0], &mut list_state);

        // Draw input area for editing
        draw_settings_input_area(f, app, chunks[1]);
        
        // Update app areas for mouse interaction
        app.settings_area = chunks[0];
    } else {
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.settings_selected_index));
        f.render_stateful_widget(settings_list, area, &mut list_state);
        
        // Update app areas for mouse interaction
        app.settings_area = area;
    }
}

fn draw_settings_input_area(f: &mut Frame, app: &App, area: Rect) {
    let is_editing = app.settings_editing_key.is_some();
    
    let border_style = if is_editing {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Blue)
    };

    let title = if let Some(ref key) = app.settings_editing_key {
        format!("Editing: {} (Enter to save, Esc to cancel)", key)
    } else {
        "Settings Input".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Render the settings input text area
    f.render_widget(&app.settings_input_area, inner_area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let current_focus = match app.focused_area {
        FocusedArea::Input => {
            if matches!(app.input_mode, InputMode::Editing) {
                "INPUT (EDITING)"
            } else {
                "INPUT"
            }
        },
        FocusedArea::Chat => "CHAT",
        FocusedArea::SessionList => "SESSIONS",
        FocusedArea::Settings => {
            if app.settings_editing_key.is_some() {
                "SETTINGS (EDITING)"
            } else {
                "SETTINGS"
            }
        },
    };

    let keybindings = if app.show_settings {
        vec![
            Line::from(vec![
                Span::styled("Focus: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(current_focus, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("↑↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Edit ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Close Settings ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Cancel Edit", Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Focus: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(current_focus, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Cycle Focus ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Edit/Send ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+N", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": New Session ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Settings ", Style::default().fg(Color::Gray)),
            ]),
        ]
    };

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(keybindings)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn draw_loading_popup(f: &mut Frame) {
    let area = centered_rect(30, 10, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Processing")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Yellow));

    let loading_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("🤖 AI is thinking...", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Please wait", Style::default().fg(Color::Gray)),
        ]),
    ]);

    let paragraph = Paragraph::new(loading_text)
        .alignment(Alignment::Center)
        .block(block);

    f.render_widget(paragraph, area);
}

fn draw_error_popup(f: &mut Frame, error: &str) {
    let area = centered_rect(60, 15, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Error")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Red));

    let error_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("❌ An error occurred:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(error),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Esc to dismiss", Style::default().fg(Color::Gray)),
        ]),
    ]);

    let paragraph = Paragraph::new(error_text)
        .alignment(Alignment::Center)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn format_message(message: &crate::session::model::message::Message) -> Text {
    let role_style = match message.role {
        Role::User => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        Role::Assistant => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        Role::System => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    };

    let role_prefix = match message.role {
        Role::User => "👤 You:",
        Role::Assistant => "🤖 AI:",
        Role::System => "⚙️  System:",
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(role_prefix, role_style),
        ]),
    ];

    // Split message content into lines and format
    for line in message.content.lines() {
        if line.trim().starts_with("```") {
            // Code block delimiter
            lines.push(Line::from(vec![
                Span::styled(line, Style::default().fg(Color::Yellow)),
            ]));
        } else if line.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(vec![
                Span::styled(line, Style::default().fg(Color::White)),
            ]));
        }
    }

    lines.push(Line::from(""));
    Text::from(lines)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 