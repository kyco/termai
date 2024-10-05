use crate::app::{App, Focus};
use crate::repository::Repository;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::{io, time::Duration};

pub fn run<R>(app: &mut App<R>) -> Result<()>
where
    R: Repository,
    R::Error: std::error::Error,
{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                match app.on_key(key.code, key.modifiers) {
                    Ok(_) => {}
                    Err(err) => println!("{:#?}", err),
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui<R>(f: &mut Frame, app: &App<R>)
where
    R: Repository,
{
    // Layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.area());

    // Sidebar
    let sidebar = Block::default()
        .title("Sidebar")
        .borders(Borders::ALL)
        .style(match app.focus {
            Focus::Sidebar => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::White),
        });
    f.render_widget(sidebar, chunks[0]);

    // Right side layout
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(chunks[1]);

    // Messages
    let messages: String = app.messages.join("\n");
    let messages_paragraph = Paragraph::new(messages)
        .block(Block::default().title("Messages").borders(Borders::ALL))
        .style(match app.focus {
            Focus::Main => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .wrap(Wrap { trim: true });
    f.render_widget(messages_paragraph, right_chunks[0]);

    // Input
    let input = Paragraph::new(app.input.clone())
        .style(match app.focus {
            Focus::Input => Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::White),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, right_chunks[1]);

    // Set cursor position
    let cursor_x = chunks[1].x + 1 + app.input.len() as u16;
    let cursor_y = right_chunks[1].y + 1;
    let position = Position::new(cursor_x, cursor_y);
    f.set_cursor_position(position);
}
