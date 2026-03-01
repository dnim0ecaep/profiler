use std::time::Instant;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

/// Spinner frames for animated loading indicator
const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// State for an animated loading indicator
#[derive(Debug, Clone)]
pub struct LoadingState {
    pub message: String,
    pub phase: String,
    pub started_at: Instant,
    pub frame: usize,
}

impl LoadingState {
    pub fn new(phase: &str, message: &str) -> Self {
        Self {
            message: message.to_string(),
            phase: phase.to_string(),
            started_at: Instant::now(),
            frame: 0,
        }
    }

    /// Advance the spinner to the next frame
    pub fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }

    /// Update the message and phase
    pub fn update(&mut self, phase: &str, message: &str) {
        self.phase = phase.to_string();
        self.message = message.to_string();
    }

    /// Get the current spinner character
    pub fn spinner(&self) -> &str {
        SPINNER_FRAMES[self.frame]
    }

    /// Get the elapsed time as a formatted string
    pub fn elapsed(&self) -> String {
        let secs = self.started_at.elapsed().as_secs_f64();
        format!("{:.1}s", secs)
    }
}

/// Render an animated loading indicator with spinner, message, and elapsed time
pub fn render_loading(
    frame: &mut ratatui::Frame,
    area: Rect,
    loading: &LoadingState,
    palette_border: Color,
    palette_bg: Color,
    palette_fg: Color,
    palette_highlight: Color,
) {
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Percentage(30),
        ])
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", loading.phase))
        .border_style(Style::default().fg(palette_highlight))
        .style(Style::default().bg(palette_bg));
    frame.render_widget(block, area);

    // Spinner + message line
    let spinner_line = Line::from(vec![
        Span::styled(
            format!(" {} ", loading.spinner()),
            Style::default().fg(palette_highlight).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            &loading.message,
            Style::default().fg(palette_fg),
        ),
    ]);
    let spinner_widget = Paragraph::new(spinner_line)
        .alignment(Alignment::Center);
    frame.render_widget(spinner_widget, inner_chunks[1]);

    // Elapsed time
    let elapsed_line = Line::from(vec![
        Span::styled(
            format!("Elapsed: {}", loading.elapsed()),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let elapsed_widget = Paragraph::new(elapsed_line)
        .alignment(Alignment::Center);
    frame.render_widget(elapsed_widget, inner_chunks[2]);

    // Progress dots animation
    let dots_count = (loading.frame % 4) + 1;
    let dots: String = ".".repeat(dots_count);
    let dots_line = Line::from(Span::styled(
        format!("Please wait{:<4}", dots),
        Style::default().fg(Color::DarkGray),
    ));
    let dots_widget = Paragraph::new(dots_line)
        .alignment(Alignment::Center);
    frame.render_widget(dots_widget, inner_chunks[3]);
}

pub fn render_trait_bar(name: &str, value: f32, area: Rect, frame: &mut ratatui::Frame) {
    let percentage = (value * 100.0) as u16;
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(format!("{}: {:.0}%", name, value * 100.0))
        .percent(percentage);
    
    frame.render_widget(gauge, area);
}

pub fn score_color(score: f32) -> Color {
    if score >= 0.8 {
        Color::Green
    } else if score >= 0.6 {
        Color::Yellow
    } else if score >= 0.4 {
        Color::LightYellow
    } else {
        Color::Red
    }
}

pub fn render_help_bar(shortcuts: &[(&str, &str)]) -> Paragraph<'static> {
    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {} ", desc)),
            ]
        })
        .collect();
    
    Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray))
}
