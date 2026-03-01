use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::PersonProfile;
use crate::ui::components::{self, LoadingState};
use crate::ui::ColorPalette;

pub fn render(frame: &mut Frame, area: Rect, profile: &PersonProfile, palette: &ColorPalette, loading: Option<&LoadingState>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new(format!("Profile: {}", profile.name))
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Main content - show loading overlay or profile details
    if let Some(loading) = loading {
        components::render_loading(frame, chunks[1], loading, palette.border, palette.background, palette.foreground, palette.highlight);
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);
        
        // Left: Traits and preferences
        render_traits_panel(frame, content_chunks[0], profile, palette);
        
        // Right: Evidence and insights
        render_insights_panel(frame, content_chunks[1], profile, palette);
    }
    
    // Help bar
    let help = Paragraph::new("U Import URL | R Reanalyze | X Explain | D Delete | C Coach | M Compare | ESC Back")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}

fn render_traits_panel(frame: &mut Frame, area: Rect, profile: &PersonProfile, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Min(5),
        ])
        .split(area);
    
    // Summary
    let summary_text = format!(
        "Style: {}\nConfidence: {:.0}%",
        profile.trait_scores.primary_style.as_str(),
        profile.confidence * 100.0
    );
    let summary = Paragraph::new(summary_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Summary")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    frame.render_widget(summary, chunks[0]);
    
    // Trait bars
    let trait_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(chunks[1]);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Traits")
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.background));
    frame.render_widget(block, chunks[1]);
    
    components::render_trait_bar("Directness", profile.trait_scores.directness, trait_chunks[0], frame);
    components::render_trait_bar("Pace", profile.trait_scores.pace, trait_chunks[1], frame);
    components::render_trait_bar("People Focus", profile.trait_scores.people_vs_task, trait_chunks[2], frame);
    components::render_trait_bar("Detail", profile.trait_scores.detail_orientation, trait_chunks[3], frame);
    components::render_trait_bar("Risk", profile.trait_scores.risk_tolerance, trait_chunks[4], frame);
    components::render_trait_bar("Formality", profile.trait_scores.formality, trait_chunks[5], frame);
    
    // Communication preferences
    let pref_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Do: ", Style::default().fg(palette.success)),
            Span::styled(
                profile.communication_preferences.do_list.join(", "),
                Style::default().fg(palette.foreground),
            ),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("Don't: ", Style::default().fg(palette.error)),
            Span::styled(
                profile.communication_preferences.dont_list.join(", "),
                Style::default().fg(palette.foreground),
            ),
        ])),
    ];
    
    let prefs = List::new(pref_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Communication")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(prefs, chunks[2]);
}

fn render_insights_panel(frame: &mut Frame, area: Rect, profile: &PersonProfile, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);
    
    // Strengths
    let strength_items: Vec<ListItem> = profile
        .strengths
        .iter()
        .map(|s| ListItem::new(format!("• {}", s)))
        .collect();
    let strengths = List::new(strength_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Strengths")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.success));
    frame.render_widget(strengths, chunks[0]);
    
    // Blind spots
    let blindspot_items: Vec<ListItem> = profile
        .blind_spots
        .iter()
        .map(|s| ListItem::new(format!("• {}", s)))
        .collect();
    let blind_spots = List::new(blindspot_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Blind Spots")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.warning));
    frame.render_widget(blind_spots, chunks[1]);
    
    // Motivators
    let motivator_items: Vec<ListItem> = profile
        .motivators
        .primary
        .iter()
        .map(|m| ListItem::new(format!("• {}", m)))
        .collect();
    let motivators = List::new(motivator_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Motivators")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.info));
    frame.render_widget(motivators, chunks[2]);
}
