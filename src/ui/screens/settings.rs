use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::config::Config;
use crate::ui::ColorPalette;

#[derive(Default)]
pub struct SettingsState {
    // Placeholder for future editing state
}

pub fn render(frame: &mut Frame, area: Rect, config: &Config, _state: &SettingsState, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Settings")
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Settings list
    let items = vec![
        ListItem::new(vec![
            Line::from("Ollama Configuration"),
            Line::from(format!("  Host: {}", config.ollama_host)),
            Line::from(format!("  Chat Model: {}", config.chat_model)),
            Line::from(format!("  Embedding Model: {}", config.embedding_model)),
        ]),
        ListItem::new(""),
        ListItem::new(vec![
            Line::from("Data Directory"),
            Line::from(format!("  Path: {}", config.data_path.display())),
        ]),
        ListItem::new(""),
        ListItem::new(vec![
            Line::from("Privacy"),
            Line::from(format!("  Store Evidence: {}", config.privacy.store_evidence)),
            Line::from(format!("  Store Draft Text: {}", config.privacy.store_draft_text)),
        ]),
        ListItem::new(""),
        ListItem::new(vec![
            Line::from("Debug"),
            Line::from(format!("  Logging: {}", config.debug_logging)),
        ]),
    ];
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(list, chunks[1]);
    
    // Help bar
    let help = Paragraph::new("Edit config file at ./config.toml (in current directory) | Ctrl+T Theme | ESC Back")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}
