use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::LlmRequestLog;
use crate::ui::ColorPalette;

pub fn render(frame: &mut Frame, area: Rect, logs: &[LlmRequestLog], palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new(format!("Request Logs ({})", logs.len()))
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Logs list
    let items: Vec<ListItem> = logs
        .iter()
        .map(|log| {
            let status_color = if log.success {
                palette.success
            } else {
                palette.error
            };
            
            let status_icon = if log.success { "✓" } else { "✗" };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(status_icon, Style::default().fg(status_color)),
                    Span::raw(" "),
                    Span::raw(&log.request_type),
                    Span::raw(" | "),
                    Span::raw(format!("{}ms", log.duration_ms)),
                    Span::raw(" | "),
                    Span::raw(log.timestamp.format("%H:%M:%S").to_string()),
                ]),
                Line::from(format!("  Model: {} | ID: {}", log.model, &log.id[..8])),
            ])
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Recent Requests")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(list, chunks[1]);
    
    // Help bar
    let help = Paragraph::new("R Refresh | C Clear Logs | Ctrl+T Theme | ESC Back")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}
