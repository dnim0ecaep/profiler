use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::{DraftAnalysis, PersonProfile};
use crate::ui::components::{self, LoadingState};
use crate::ui::ColorPalette;

pub struct CoachState {
    pub draft_input: String,
    pub selected_profile: Option<PersonProfile>,
    pub analysis: Option<DraftAnalysis>,
    pub mode: CoachMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoachMode {
    SelectProfile,
    EnterDraft,
    ViewAnalysis,
    Processing,
}

impl Default for CoachState {
    fn default() -> Self {
        Self {
            draft_input: String::new(),
            selected_profile: None,
            analysis: None,
            mode: CoachMode::SelectProfile,
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &CoachState, profiles: &[PersonProfile], palette: &ColorPalette, selected_index: usize, loading: Option<&LoadingState>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Message Coach")
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Main content based on mode
    match state.mode {
        CoachMode::SelectProfile => render_select_profile(frame, chunks[1], profiles, palette, selected_index),
        CoachMode::EnterDraft => render_enter_draft(frame, chunks[1], &state.draft_input, &state.selected_profile, palette),
        CoachMode::ViewAnalysis => render_analysis(frame, chunks[1], state.analysis.as_ref(), palette),
        CoachMode::Processing => {
            if let Some(loading) = loading {
                components::render_loading(frame, chunks[1], loading, palette.border, palette.background, palette.foreground, palette.highlight);
            } else {
                render_processing(frame, chunks[1], palette);
            }
        }
    }
    
    // Help bar
    let help_text = match state.mode {
        CoachMode::SelectProfile => "Select target profile | ↑/↓ Navigate | Enter Select | ESC Back",
        CoachMode::EnterDraft => "Enter draft message | Ctrl+S Analyze | ESC Back",
        CoachMode::ViewAnalysis => "R Rewrite | C Copy | N New Draft | ESC Back",
        CoachMode::Processing => "Analyzing draft...",
    };
    
    let help = Paragraph::new(help_text)
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}

fn render_select_profile(frame: &mut Frame, area: Rect, profiles: &[PersonProfile], palette: &ColorPalette, selected_index: usize) {
    let items: Vec<ListItem> = profiles
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let is_selected = idx == selected_index;
            let text = format!("{} - {}", p.name, p.trait_scores.primary_style.as_str());
            
            if is_selected {
                ListItem::new(Line::from(vec![
                    Span::styled("▶ ", Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD)),
                    Span::styled(text, Style::default().fg(palette.highlight)),
                ]))
                .style(Style::default().bg(palette.highlight_bg))
            } else {
                ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(text, Style::default().fg(palette.foreground)),
                ]))
            }
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Select Target Profile")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(list, area);
}

fn render_enter_draft(frame: &mut Frame, area: Rect, draft: &str, profile: &Option<PersonProfile>, palette: &ColorPalette) {
    let title = if let Some(p) = profile {
        format!("Draft for: {} | {} chars", p.name, draft.len())
    } else {
        format!("Draft | {} chars", draft.len())
    };
    
    let display_text = if draft.is_empty() {
        "Paste or type your draft message here...\n\nThis could be:\n- An email\n- A Slack message\n- Meeting notes\n- Any communication\n\nI'll analyze it against the target profile."
    } else {
        draft
    };
    
    let paragraph = Paragraph::new(display_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(paragraph, area);
}

fn render_analysis(frame: &mut Frame, area: Rect, analysis: Option<&DraftAnalysis>, palette: &ColorPalette) {
    if let Some(analysis) = analysis {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Min(5),
            ])
            .split(area);
        
        // Scores
        render_scores(frame, chunks[0], analysis, palette);
        
        // Risky phrases and explanation
        render_details(frame, chunks[1], analysis, palette);
    } else {
        let text = Paragraph::new("No analysis available")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)))
            .style(Style::default().fg(palette.foreground));
        frame.render_widget(text, area);
    }
}

fn render_scores(frame: &mut Frame, area: Rect, analysis: &DraftAnalysis, palette: &ColorPalette) {
    let score_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Analysis Score: {:.0}%", analysis.overall_score * 100.0))
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.background));
    frame.render_widget(block, area);
    
    components::render_trait_bar("Clarity", analysis.subscores.clarity, score_chunks[0], frame);
    components::render_trait_bar("Tone Fit", analysis.subscores.tone_fit, score_chunks[1], frame);
    components::render_trait_bar("Directness", analysis.subscores.directness_fit, score_chunks[2], frame);
}

fn render_details(frame: &mut Frame, area: Rect, analysis: &DraftAnalysis, palette: &ColorPalette) {
    let mut items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Explanation: ", Style::default().fg(palette.info)),
        ])),
        ListItem::new(analysis.explanation.clone()),
        ListItem::new(""),
    ];
    
    if !analysis.risky_phrases.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled("⚠ Risky Phrases:", Style::default().fg(palette.warning).add_modifier(Modifier::BOLD)),
        ])));
        
        for risky in &analysis.risky_phrases {
            items.push(ListItem::new(format!("  \"{}\" - {}", risky.phrase, risky.reason)));
            items.push(ListItem::new(format!("    → {}", risky.suggestion)));
        }
    }
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Details")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(list, area);
}

fn render_processing(frame: &mut Frame, area: Rect, palette: &ColorPalette) {
    let text = Paragraph::new("Analyzing draft against profile...\n\nThis may take a moment.")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Processing")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.warning));
    
    frame.render_widget(text, area);
}
