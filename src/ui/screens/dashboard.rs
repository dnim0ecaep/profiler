use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::PersonProfile;
use crate::ui::ColorPalette;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    recent_profiles: &[PersonProfile],
    ollama_connected: bool,
    palette: &ColorPalette,
    selected_index: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Profiler - Communication Intelligence Coach")
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Main content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Recent profiles
    let profile_items: Vec<ListItem> = recent_profiles
        .iter()
        .take(10)
        .map(|p| {
            let confidence_color = if p.confidence >= 0.7 {
                palette.success
            } else if p.confidence >= 0.5 {
                palette.warning
            } else {
                palette.error
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&p.name, Style::default().fg(palette.foreground)),
                Span::raw(" "),
                Span::styled(
                    format!("[{:.0}%]", p.confidence * 100.0),
                    Style::default().fg(confidence_color),
                ),
            ]))
        })
        .collect();
    
    let profiles_list = List::new(profile_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Profiles")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .style(Style::default().fg(palette.foreground).bg(palette.background));
    
    frame.render_widget(profiles_list, content_chunks[0]);
    
    // System status
    let status_color = if ollama_connected {
        palette.success
    } else {
        palette.error
    };
    
    let status_text = if ollama_connected {
        "● Connected"
    } else {
        "● Disconnected"
    };
    
    // Menu items with selection highlighting
    let menu_items = [
        ("N", "New Profile"),
        ("C", "Coach Message"),
        ("P", "View Profiles"),
        ("M", "Compare Profiles"),
        ("S", "Settings"),
        ("L", "Logs"),
    ];
    
    let mut status_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Ollama: ", Style::default().fg(palette.foreground)),
            Span::styled(status_text, Style::default().fg(status_color)),
        ])),
        ListItem::new(""),
        ListItem::new(Span::styled("Quick Actions:", Style::default().fg(palette.title))),
    ];
    
    for (idx, (key, label)) in menu_items.iter().enumerate() {
        let is_selected = idx == selected_index;
        let item = if is_selected {
            // Highlighted selection
            ListItem::new(Line::from(vec![
                Span::styled("► ", Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD)),
                Span::styled("[", Style::default().fg(palette.secondary)),
                Span::styled(*key, Style::default().fg(palette.selected).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(palette.secondary)),
                Span::styled(*label, Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD)),
            ]))
            .style(Style::default().bg(palette.selected_bg))
        } else {
            ListItem::new(Line::from(vec![
                Span::styled("  [", Style::default().fg(palette.secondary)),
                Span::styled(*key, Style::default().fg(palette.status_bar_key).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(palette.secondary)),
                Span::styled(*label, Style::default().fg(palette.foreground)),
            ]))
        };
        status_items.push(item);
    }
    
    let status_list = List::new(status_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("System & Shortcuts")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .style(Style::default().fg(palette.foreground).bg(palette.background));
    
    frame.render_widget(status_list, content_chunks[1]);
    
    // Help bar
    let help = Paragraph::new("↑/↓ Select | Enter Open | ? Help | Q Quit | Ctrl+T Theme | ESC Back")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}
