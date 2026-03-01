use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{CompatibilityReport, PersonProfile};
use crate::ui::components::{self, LoadingState};
use crate::ui::ColorPalette;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    profiles: &[PersonProfile],
    profile1_id: Option<&str>,
    profile2_id: Option<&str>,
    selecting: usize,
    selected: usize,
    report: Option<&CompatibilityReport>,
    palette: &ColorPalette,
    loading: Option<&LoadingState>,
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
    let title_text = match (profile1_id, profile2_id) {
        (Some(p1), Some(p2)) => format!("Compare: {} vs {}", p1, p2),
        (Some(p1), None) => format!("Compare: {} vs ???", p1),
        _ => "Compare Profiles".to_string(),
    };
    
    let title_widget = Paragraph::new(title_text)
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title_widget, chunks[0]);
    
    // Main content
    if let Some(report) = report {
        render_comparison(frame, chunks[1], report, palette);
    } else if profile1_id.is_some() && profile2_id.is_some() {
        // Both selected, comparison in progress
        if let Some(loading) = loading {
            components::render_loading(frame, chunks[1], loading, palette.border, palette.background, palette.foreground, palette.highlight);
        } else {
            let text = Paragraph::new("Comparing profiles... This may take a moment.\n\nThe AI is analyzing compatibility, alignment areas, and friction points.")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("⏳ Comparing")
                    .border_style(Style::default().fg(palette.border))
                    .style(Style::default().bg(palette.background)))
                .style(Style::default().fg(palette.warning));
            frame.render_widget(text, chunks[1]);
        }
    } else {
        // Profile selection mode
        render_profile_selection(frame, chunks[1], profiles, profile1_id, selecting, selected, palette);
    }
    
    // Help bar
    let help_text = if report.is_some() {
        "N New Comparison | Ctrl+T Theme | ESC Back"
    } else {
        "↑/↓ Navigate | Enter Select | N New | Ctrl+T Theme | ESC Back"
    };
    let help = Paragraph::new(help_text)
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}

fn render_profile_selection(
    frame: &mut Frame,
    area: Rect,
    profiles: &[PersonProfile],
    profile1_id: Option<&str>,
    selecting: usize,
    selected: usize,
    palette: &ColorPalette,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);
    
    // Selection status
    let step = if selecting == 0 { "Step 1: Select first profile" } else { "Step 2: Select second profile to compare with" };
    let status_text = if let Some(p1) = profile1_id {
        format!("{}\n  Profile 1: {}", step, p1)
    } else {
        step.to_string()
    };
    
    let status = Paragraph::new(status_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Selection")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.info));
    frame.render_widget(status, chunks[0]);
    
    // Profile list
    if profiles.is_empty() {
        let empty = Paragraph::new("No profiles available. Create profiles first!")
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Profiles")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)))
            .style(Style::default().fg(palette.foreground));
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = profiles
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let prefix = if i == selected { "► " } else { "  " };
                let already_selected = profile1_id.map_or(false, |id| id == p.id);
                let suffix = if already_selected { " ✓ (selected)" } else { "" };
                
                let style = if i == selected {
                    Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD)
                } else if already_selected {
                    Style::default().fg(palette.success)
                } else {
                    Style::default().fg(palette.foreground)
                };
                
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(&p.name, style),
                    Span::styled(
                        format!(" [{}]", p.trait_scores.primary_style.as_str()),
                        Style::default().fg(palette.secondary),
                    ),
                    Span::styled(suffix, Style::default().fg(palette.success)),
                ]))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Select a Profile")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)));
        frame.render_widget(list, chunks[1]);
    }
}

fn render_comparison(frame: &mut Frame, area: Rect, report: &CompatibilityReport, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);
    
    // Compatibility score
    let score_text = format!(
        "Compatibility Score: {:.0}%",
        report.compatibility_score * 100.0
    );
    let score = Paragraph::new(score_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default()
            .fg(components::score_color(report.compatibility_score))
            .add_modifier(Modifier::BOLD));
    frame.render_widget(score, chunks[0]);
    
    // Split into two columns
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Alignment areas
    let alignment_items: Vec<ListItem> = report
        .alignment_areas
        .iter()
        .map(|a| {
            ListItem::new(vec![
                Line::from(Span::styled(&a.dimension, Style::default().fg(palette.success).add_modifier(Modifier::BOLD))),
                Line::from(format!("  {}", a.description)),
            ])
        })
        .collect();
    
    let alignments = List::new(alignment_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("✓ Alignment Areas")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    frame.render_widget(alignments, content_chunks[0]);
    
    // Friction points
    let friction_items: Vec<ListItem> = report
        .friction_points
        .iter()
        .map(|f| {
            ListItem::new(vec![
                Line::from(Span::styled(&f.dimension, Style::default().fg(palette.error).add_modifier(Modifier::BOLD))),
                Line::from(format!("  {}", f.description)),
                Line::from(Span::styled(format!("  → {}", f.mitigation), Style::default().fg(palette.warning))),
            ])
        })
        .collect();
    
    let frictions = List::new(friction_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("⚠ Friction Points")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    frame.render_widget(frictions, content_chunks[1]);
    
    // Recommendations
    let rec_items: Vec<ListItem> = report
        .recommendations
        .iter()
        .map(|r| ListItem::new(format!("• {}", r)))
        .collect();
    
    let recommendations = List::new(rec_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Recommendations")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.info));
    frame.render_widget(recommendations, chunks[2]);
}
