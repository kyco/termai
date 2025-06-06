use crate::ui::tui::app::{App, FocusedArea, InputMode};
use crate::llm::common::model::role::Role;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};
use unicode_width::UnicodeWidthStr;

pub fn draw(f: &mut Frame, app: &mut App) {
    // Main layout with footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(f.area());

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

fn draw_chat_area(f: &mut Frame, app: &App, area: Rect) {
    let current_session = app.current_session();
    let is_focused = matches!(app.focused_area, FocusedArea::Chat);
    
    let title = if let Some(session) = current_session {
        let base_title = if session.name == "temporary" {
            "Chat - Temporary Session".to_string()
        } else {
            format!("Chat - {}", session.name)
        };
        
        if is_focused {
            format!("{} (Focused)", base_title)
        } else {
            base_title
        }
    } else {
        if is_focused {
            "Chat (Focused)".to_string()
        } else {
            "Chat".to_string()
        }
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Blue)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if let Some(session) = current_session {
        let messages = &session.messages;
        let visible_messages: Vec<Text> = messages
            .iter()
            .filter(|msg| msg.role != Role::System)
            .skip(app.scroll_offset)
            .map(|msg| format_message(msg))
            .collect();

        if !visible_messages.is_empty() {
            let mut chat_content = Text::default();
            for (i, message) in visible_messages.iter().enumerate() {
                if i > 0 {
                    chat_content.extend(Text::from("\n"));
                }
                chat_content.extend(message.clone());
            }
            let paragraph = Paragraph::new(chat_content)
                .wrap(Wrap { trim: true })
                .scroll((0, 0));
            f.render_widget(paragraph, inner_area);
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

        // Draw scrollbar if needed
        let total_messages = messages.iter().filter(|msg| msg.role != Role::System).count();
        if total_messages > inner_area.height as usize {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));
            let mut scrollbar_state = ScrollbarState::new(total_messages)
                .position(app.scroll_offset);
            f.render_stateful_widget(
                scrollbar,
                inner_area,
                &mut scrollbar_state,
            );
        }
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
    };

    let keybindings = vec![
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
            Span::styled("Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(": Exit Edit", Style::default().fg(Color::Gray)),
        ]),
    ];

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