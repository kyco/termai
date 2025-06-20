use crate::ui::tui::app::{App, FocusedArea, InputMode};
use crate::config::service::config_service;
use crate::llm::common::model::role::Role;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

    // Draw help modal if needed
    if app.show_help {
        draw_help_modal(f);
    }
}

fn draw_session_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = matches!(app.focused_area, FocusedArea::SessionList);
    
    let sessions: Vec<ListItem> = app
        .sessions
        .iter()
        .enumerate()
        .map(|(i, session)| {
            let original_index = i;
            let style = if original_index == app.current_session_index {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(border_style);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Add subtle padding for the list content
    let padded_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y,
        width: inner_area.width.saturating_sub(1),
        height: inner_area.height,
    };

    let sessions_list = List::new(sessions)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    let reversed_index = app.current_session_index;
    list_state.select(Some(reversed_index));

    f.render_stateful_widget(sessions_list, padded_area, &mut list_state);

    // Draw scrollbar for sessions if needed
    if app.sessions.len() > padded_area.height as usize {
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state = ScrollbarState::new(app.sessions.len())
            .position(app.session_scroll_offset);
        f.render_stateful_widget(
            scrollbar,
            padded_area,
            &mut scrollbar_state,
        );
    }
}

fn draw_chat_area(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = matches!(app.focused_area, FocusedArea::Chat);
    let is_visual_mode = app.is_in_visual_mode();
    
    // Create the UI area first
    let title = if is_visual_mode {
        match app.selection_mode {
            crate::ui::tui::app::SelectionMode::Visual => "Chat (VISUAL MODE - move to select, V for line mode, y to copy, Esc to exit)",
            crate::ui::tui::app::SelectionMode::VisualLine => "Chat (VISUAL LINE MODE - move to select lines, y to copy, Esc to exit)",
            _ => "Chat"
        }
    } else {
        "Chat"
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .border_style(if is_focused {
            if is_visual_mode {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::Yellow)
            }
        } else {
            Style::default().fg(Color::Blue)
        });

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Add subtle padding for better visual spacing
    let padded_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y + 1,
        width: inner_area.width.saturating_sub(2),
        height: inner_area.height.saturating_sub(2),
    };

    // Get session messages first, completely separate from any borrowing
    let messages = app.current_session()
        .map(|session| session.messages.clone())
        .unwrap_or_default();
    
    let filtered_messages: Vec<_> = messages
        .iter()
        .filter(|msg| msg.role != Role::System)
        .collect();
    
    if !filtered_messages.is_empty() {
        // Update chat content cache if needed for selection
        if is_visual_mode && app.chat_content_lines.is_empty() {
            app.update_chat_content_cache();
        }
        
        // Create all chat content as a single Text widget with selection highlighting
        let chat_content = if is_visual_mode {
            format_messages_with_selection(&filtered_messages, app)
        } else {
            format_messages_with_markdown(&filtered_messages, app)
        };
        
        // Calculate actual rendered height considering text wrapping
        let available_width = padded_area.width as usize;
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
        
        let available_height = padded_area.height as usize;
        
        // Clamp scroll position based on actual wrapped content height
        app.clamp_scroll_to_content_lines(total_wrapped_lines, available_height);
        
        // Use Paragraph's scroll feature for proper line-based scrolling
        let paragraph = Paragraph::new(chat_content)
            .wrap(Wrap { trim: true })
            .scroll((app.scroll_offset as u16, 0));
        f.render_widget(paragraph, padded_area);

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
                padded_area,
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
        f.render_widget(paragraph, padded_area);
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

    // Add subtle horizontal padding for input
    let padded_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y,
        width: inner_area.width.saturating_sub(2),
        height: inner_area.height,
    };

    // Render the text area
    f.render_widget(&app.input_area, padded_area);
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

            let display_text = if *key == "provider_key" && app.settings_provider_selecting && i == app.settings_selected_index {
                format!("{}: [Select Provider]", display_name)
            } else {
                format!("{}: {}", display_name, value)
            };
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

    // If editing a setting or selecting provider, split the area to show input/selection
    if app.settings_editing_key.is_some() || app.settings_provider_selecting {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .split(area);

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.settings_selected_index));
        f.render_stateful_widget(settings_list, chunks[0], &mut list_state);

        // Draw input area for editing or provider selection
        if app.settings_provider_selecting {
            draw_provider_selection_area(f, app, chunks[1]);
        } else {
            draw_settings_input_area(f, app, chunks[1]);
        }
        
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

fn draw_provider_selection_area(f: &mut Frame, app: &App, area: Rect) {
    let border_style = Style::default().fg(Color::Green);

    let providers = vec!["OpenAI", "Claude"];
    let provider_items: Vec<ListItem> = providers
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let style = if i == app.settings_provider_selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(*provider).style(style)
        })
        .collect();

    let provider_list = List::new(provider_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Provider (↑↓ to navigate, Enter to select, Esc to cancel)")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                .border_style(border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.settings_provider_selected_index));
    f.render_stateful_widget(provider_list, area, &mut list_state);
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
                Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Help", Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                Span::styled("Focus: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(current_focus, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Cycle ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Navigate ", Style::default().fg(Color::Gray)),
                Span::styled("Ctrl+N", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": New Session ", Style::default().fg(Color::Gray)),
                Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(": Help", Style::default().fg(Color::Gray)),
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


fn format_messages_with_markdown(messages: &[&crate::session::model::message::Message], app: &mut crate::ui::tui::app::App) -> Text<'static> {
    let mut lines = Vec::new();
    
    for (i, message) in messages.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        
        // Add role header
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

        lines.push(Line::from(vec![
            Span::styled(role_prefix.to_string(), role_style),
        ]));
        
        // Try to render message content with markdown
        if let Some(ref mut markdown_display) = app.markdown_display {
            match markdown_display.render_to_text(&message.content) {
                Ok(rendered_text) => {
                    // Add the markdown-rendered content
                    for line in rendered_text.lines {
                        lines.push(line);
                    }
                }
                Err(_) => {
                    // Fallback to basic formatting on error
                    let fallback_text = format_message_content_basic(&message.content);
                    for line in fallback_text.lines {
                        lines.push(line);
                    }
                }
            }
        } else {
            // No markdown renderer available, use basic formatting
            let fallback_text = format_message_content_basic(&message.content);
            for line in fallback_text.lines {
                lines.push(line);
            }
        }
        
        lines.push(Line::from(""));
    }
    
    Text::from(lines)
}

fn format_message_content_basic(content: &str) -> Text<'static> {
    let mut lines = Vec::new();
    
    for line in content.lines() {
        if line.trim().starts_with("```") {
            // Code block delimiter
            lines.push(Line::from(vec![
                Span::styled(line.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        } else if line.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(vec![
                Span::styled(line.to_string(), Style::default().fg(Color::White)),
            ]));
        }
    }
    
    Text::from(lines)
}

fn format_messages_with_selection(messages: &[&crate::session::model::message::Message], app: &mut crate::ui::tui::app::App) -> Text<'static> {
    let mut lines = Vec::new();
    let mut current_line = 0;
    
    let selection = app.selection.as_ref();
    let cursor_pos = &app.cursor_position;
    
    // Get selection bounds if exists
    let selection_bounds = selection.map(|sel| {
        let start = &sel.start;
        let end = &sel.end;
        if start.line < end.line || (start.line == end.line && start.column <= end.column) {
            (start, end)
        } else {
            (end, start)
        }
    });
    
    for (msg_idx, message) in messages.iter().enumerate() {
        if msg_idx > 0 {
            lines.push(Line::from(""));
            current_line += 1;
        }
        
        // Add role prefix line
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
        
        lines.push(create_line_with_selection(role_prefix, current_line, cursor_pos, selection_bounds, role_style, true));
        current_line += 1;
        
        // Add message content lines
        for content_line in message.content.lines() {
            let base_style = if content_line.trim().starts_with("```") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            
            lines.push(create_line_with_selection(content_line, current_line, cursor_pos, selection_bounds, base_style, true));
            current_line += 1;
        }
        
        lines.push(Line::from(""));
        current_line += 1;
    }
    
    Text::from(lines)
}

fn create_line_with_selection(
    text: &str,
    line_idx: usize,
    cursor_pos: &crate::ui::tui::app::CursorPosition,
    selection_bounds: Option<(&crate::ui::tui::app::CursorPosition, &crate::ui::tui::app::CursorPosition)>,
    base_style: Style,
    is_visual_mode: bool,
) -> Line<'static> {
    
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    
    // Build spans character by character to handle cursor positioning properly
    let mut char_idx = 0;
    
    while char_idx <= chars.len() {
        let is_cursor_pos = is_visual_mode && line_idx == cursor_pos.line && char_idx == cursor_pos.column;
        let is_selected = if let Some((start, end)) = selection_bounds {
            let (start, end) = if start.line < end.line || (start.line == end.line && start.column <= end.column) {
                (start, end)
            } else {
                (end, start)
            };
            
            if line_idx == start.line && line_idx == end.line {
                // Single line selection
                char_idx >= start.column && char_idx < end.column
            } else if line_idx == start.line {
                // First line of multi-line selection
                char_idx >= start.column
            } else if line_idx == end.line {
                // Last line of multi-line selection
                char_idx < end.column
            } else if line_idx > start.line && line_idx < end.line {
                // Middle line of multi-line selection
                true
            } else {
                false
            }
        } else {
            false
        };
        
        if is_cursor_pos {
            // Render cursor with high visibility - always show cursor even if text is selected
            if char_idx < chars.len() {
                // Highlight the character under the cursor with bright yellow background
                let char_at_cursor = chars[char_idx];
                let cursor_style = if is_selected {
                    // Cursor on selected text: use magenta background to distinguish from selection
                    Style::default().fg(Color::White).bg(Color::Magenta).add_modifier(Modifier::BOLD)
                } else {
                    // Cursor on normal text: use yellow background
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                };
                spans.push(Span::styled(char_at_cursor.to_string(), cursor_style));
            } else {
                // Cursor at end of line - show block cursor
                spans.push(Span::styled(
                    "█",
                    Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::BOLD)
                ));
            }
        } else if char_idx < chars.len() {
            // Regular character
            let char_at_pos = chars[char_idx];
            let style = if is_selected {
                base_style.bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                base_style
            };
            spans.push(Span::styled(char_at_pos.to_string(), style));
        }
        
        char_idx += 1;
    }
    
    // If no spans were created (empty line), add at least an empty span
    if spans.is_empty() {
        if is_visual_mode && line_idx == cursor_pos.line && cursor_pos.column == 0 {
            // Show cursor on empty line
            spans.push(Span::styled(
                "█",
                Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::BOLD)
            ));
        } else {
            spans.push(Span::styled("", base_style));
        }
    }
    
    Line::from(spans)
}


fn draw_help_modal(f: &mut Frame) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Help - TermAI Shortcuts")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = Text::from(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("        - Cycle focus between Sessions → Chat → Input", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("       - Navigate within focused area", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("      - Edit input (when focused) or send message", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("        - Exit edit mode or dismiss dialogs", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sessions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+N", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("     - Create new session", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Settings & Help:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("     - Toggle settings view", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("          - Show this help dialog", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Text Selection (vim-style):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  v", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("          - Enter visual mode (cursor only, move to start selecting)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  V", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("          - Enter visual line mode (select full lines)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  ↑↓←→", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("       - Move cursor and extend selection (in visual mode)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  y", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("          - Copy selected text to clipboard (in visual mode)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("        - Exit visual mode", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mouse Support:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Click", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("      - Focus area or select session", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Scroll", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("     - Navigate through content", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Exit:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("     - Quit application", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Alt+Q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("      - Quit application", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ? or Esc to close this help", Style::default().fg(Color::Gray)),
        ]),
    ]);

    let paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
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
