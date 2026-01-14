//! Help overlay widget.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Renders help overlay centered on screen.
pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let help_text = vec![
        Line::from(Span::styled(
            "Prefix Commands (Ctrl+a then key):",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Cyan)),
            Span::raw("  Quit TUI"),
        ]),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(Color::Cyan)),
            Span::raw("  Show this help"),
        ]),
        Line::from(vec![
            Span::styled("  p", Style::default().fg(Color::Cyan)),
            Span::raw("  Pause/resume loop"),
        ]),
        Line::from(vec![
            Span::styled("  n", Style::default().fg(Color::Cyan)),
            Span::raw("  Skip to next iteration"),
        ]),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(Color::Cyan)),
            Span::raw("  Abort loop"),
        ]),
        Line::from(vec![
            Span::styled("  [", Style::default().fg(Color::Cyan)),
            Span::raw("  Enter scroll mode"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to dismiss",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left);

    let popup_area = centered_rect(50, 60, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
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
