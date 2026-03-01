use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::components::{self, LoadingState};
use crate::ui::ColorPalette;

pub struct CreateProfileState {
    pub mode: CreateMode,
    pub name_input: String,
    pub text_input: String,
    pub status_message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateMode {
    SelectType,
    EnterName,
    EnterText,
    Processing,
}

impl Default for CreateProfileState {
    fn default() -> Self {
        Self {
            mode: CreateMode::SelectType,
            name_input: String::new(),
            text_input: String::new(),
            status_message: String::new(),
        }
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &CreateProfileState, palette: &ColorPalette, selected_index: usize, loading: Option<&LoadingState>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Create Profile")
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
        CreateMode::SelectType => render_select_type(frame, chunks[1], palette, selected_index),
        CreateMode::EnterName => render_enter_name(frame, chunks[1], &state.name_input, palette),
        CreateMode::EnterText => render_enter_text(frame, chunks[1], &state.text_input, palette),
        CreateMode::Processing => {
            if let Some(loading) = loading {
                components::render_loading(frame, chunks[1], loading, palette.border, palette.background, palette.foreground, palette.highlight);
            } else {
                render_processing(frame, chunks[1], palette);
            }
        }
    }
    
    // Status/help bar
    let help_text = match state.mode {
        CreateMode::SelectType => "Select profile type: ↑/↓ Navigate | Enter Select | ESC Back",
        CreateMode::EnterName => "Enter profile name | Enter Continue | ESC Back",
        CreateMode::EnterText => "Paste/type text | Ctrl+S Save | ESC Cancel",
        CreateMode::Processing => "Processing profile...",
    };
    
    let mut help = Paragraph::new(help_text)
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    
    if !state.status_message.is_empty() {
        help = Paragraph::new(state.status_message.as_str())
            .style(Style::default()
                .bg(palette.info)
                .fg(palette.background));
    }
    
    frame.render_widget(help, chunks[2]);
}

fn render_select_type(frame: &mut Frame, area: Rect, palette: &ColorPalette, selected_index: usize) {
    let items = vec![
        ListItem::new("Text Inference - Analyze pasted text or writing samples"),
        ListItem::new("File Import - Import from .txt, .md, .json files"),
        ListItem::new("Manual Notes - Create from observations about a person"),
    ];
    
    // Create highlighted items manually
    let items: Vec<ListItem> = vec![
        "Text Inference - Analyze pasted text or writing samples",
        "File Import - Import from .txt, .md, .json files",
        "Manual Notes - Create from observations about a person",
    ]
    .into_iter()
    .enumerate()
    .map(|(idx, text)| {
        let is_selected = idx == selected_index;
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Profile Type")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        );
    
    frame.render_widget(list, area);
}

fn render_enter_name(frame: &mut Frame, area: Rect, name: &str, palette: &ColorPalette) {
    let input = Paragraph::new(name)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Profile Name")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .style(Style::default().fg(palette.highlight));
    
    frame.render_widget(input, area);
}

fn render_enter_text(frame: &mut Frame, area: Rect, text: &str, palette: &ColorPalette) {
    let display_text = if text.is_empty() {
        "Paste or type text samples here...\n\nThis can be:\n- Email messages\n- Chat transcripts\n- Writing samples\n- Meeting notes\n\nThe more text, the better the analysis."
    } else {
        text
    };
    
    let input = Paragraph::new(display_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Text Input ({} chars)", text.len()))
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(palette.foreground));
    
    frame.render_widget(input, area);
}

fn render_processing(frame: &mut Frame, area: Rect, palette: &ColorPalette) {
    let text = Paragraph::new("Processing profile with AI...\n\nThis may take a moment.")
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Processing")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)))
        .style(Style::default().fg(palette.warning))
        .wrap(Wrap { trim: false });
    
    frame.render_widget(text, area);
}
