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
    profiles: &[PersonProfile],
    selected_index: usize,
    palette: &ColorPalette,
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
    let title = Paragraph::new(format!("Saved Profiles ({})", profiles.len()))
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Profile list
    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let confidence_color = if p.confidence >= 0.7 {
                palette.success
            } else if p.confidence >= 0.5 {
                palette.warning
            } else {
                palette.error
            };
            
            let is_selected = idx == selected_index;
            let prefix = if is_selected { "▶ " } else { "  " };
            
            let name_style = if is_selected {
                Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::BOLD).fg(palette.foreground)
            };
            
            let detail_style = if is_selected {
                Style::default().fg(palette.highlight)
            } else {
                Style::default().fg(palette.foreground)
            };
            
            let item = ListItem::new(vec![
                Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(&p.name, name_style),
                    Span::raw(" "),
                    Span::styled(
                        format!("[{:.0}%]", p.confidence * 100.0),
                        Style::default().fg(confidence_color),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!(
                            "  {} | {} | {}",
                            p.trait_scores.primary_style.as_str(),
                            p.created_at.format("%Y-%m-%d"),
                            if p.tags.is_empty() {
                                "No tags".to_string()
                            } else {
                                p.tags.join(", ")
                            }
                        ),
                        detail_style,
                    ),
                ]),
            ]);
            
            if is_selected {
                item.style(Style::default().bg(palette.highlight_bg))
            } else {
                item
            }
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Profiles")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(list, chunks[1]);
    
    // Help bar
    let help = Paragraph::new("↑/↓ Navigate | Enter View | N New | D Delete | / Search | Ctrl+T Theme | ESC Back")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}
