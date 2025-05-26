use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_help_text(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("• Tab: Switch focus between input and chat"),
        Line::from("• Ctrl+Enter: Send message"),
        Line::from("• Ctrl+←/→: Switch between sessions"),
        Line::from("• Ctrl+↑/↓: Scroll chat history"),
        Line::from("• Ctrl+Q: Quit application"),
        Line::from("• Esc: Exit current mode/dismiss popups"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Blue)),
        );

    f.render_widget(paragraph, area);
}

pub fn render_status_bar(f: &mut Frame, area: Rect, status: &str, provider: &str) {
    let status_text = vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(status, Style::default().fg(Color::Green)),
            Span::styled(" | Provider: ", Style::default().fg(Color::Gray)),
            Span::styled(provider, Style::default().fg(Color::Cyan)),
        ]),
    ];

    let paragraph = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(paragraph, area);
} 