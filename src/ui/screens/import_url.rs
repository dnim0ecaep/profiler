use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::scraper;
use crate::ui::components::{self, LoadingState};
use crate::ui::ColorPalette;

/// State for the Import URL screen
pub struct ImportUrlState {
    pub mode: ImportUrlMode,
    pub url_input: String,
    pub platform_label: String,
    pub loading: Option<LoadingState>,
    pub result_message: Option<String>,
    pub result_chars: usize,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportUrlMode {
    EnterUrl,
    Scraping,
    Done,
    Error,
}

impl Default for ImportUrlState {
    fn default() -> Self {
        Self {
            mode: ImportUrlMode::EnterUrl,
            url_input: String::new(),
            platform_label: String::new(),
            loading: None,
            result_message: None,
            result_chars: 0,
            is_error: false,
        }
    }
}

impl ImportUrlState {
    /// Update the detected platform label based on the current URL input
    pub fn update_platform_detection(&mut self) {
        if self.url_input.is_empty() {
            self.platform_label = String::new();
        } else {
            let platform = scraper::detect_platform(&self.url_input);
            self.platform_label = format!("Detected: {}", platform.display_name());
        }
    }

    /// Start the scraping loading state
    pub fn start_scraping(&mut self) {
        let platform = scraper::detect_platform(&self.url_input);
        self.mode = ImportUrlMode::Scraping;
        self.loading = Some(LoadingState::new(
            "Scraping",
            &format!("Launching browser for {}...", platform.display_name()),
        ));
    }

    /// Update loading to show navigation phase
    pub fn update_navigating(&mut self) {
        if let Some(ref mut loading) = self.loading {
            let platform = scraper::detect_platform(&self.url_input);
            loading.update(
                "Scraping",
                &format!("Extracting posts from {}...", platform.display_name()),
            );
        }
    }

    /// Mark scraping as complete
    pub fn finish_success(&mut self, chars: usize, filename: &str) {
        self.mode = ImportUrlMode::Done;
        self.result_chars = chars;
        self.is_error = false;
        self.result_message = Some(format!(
            "✅ Saved {} characters to sources/{}",
            chars, filename
        ));
        self.loading = None;
    }

    /// Mark scraping as failed
    pub fn finish_error(&mut self, error: &str) {
        self.mode = ImportUrlMode::Error;
        self.is_error = true;
        // Truncate very long error messages for display but keep them informative
        let display_error = if error.len() > 500 {
            format!("{}...", &error[..500])
        } else {
            error.to_string()
        };
        self.result_message = Some(format!("❌ {}", display_error));
        self.loading = None;
    }

    /// Update loading to show scrolling phase
    pub fn update_scrolling(&mut self) {
        if let Some(ref mut loading) = self.loading {
            let platform = scraper::detect_platform(&self.url_input);
            loading.update(
                "Scraping",
                &format!("Scrolling & loading posts from {}...", platform.display_name()),
            );
        }
    }

    /// Update loading to show extraction phase
    pub fn update_extracting(&mut self) {
        if let Some(ref mut loading) = self.loading {
            loading.update(
                "Scraping",
                "Extracting text content from page...",
            );
        }
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &ImportUrlState,
    profile_name: &str,
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
    let title = Paragraph::new(format!("Import from URL → {}", profile_name))
        .style(
            Style::default()
                .fg(palette.title)
                .bg(palette.background)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(palette.border)),
        );
    frame.render_widget(title, chunks[0]);

    // Main content based on mode
    match state.mode {
        ImportUrlMode::EnterUrl => render_enter_url(frame, chunks[1], state, palette),
        ImportUrlMode::Scraping => render_scraping(frame, chunks[1], state, palette),
        ImportUrlMode::Done => render_result(frame, chunks[1], state, palette),
        ImportUrlMode::Error => render_result(frame, chunks[1], state, palette),
    }

    // Help bar
    let help_text = match state.mode {
        ImportUrlMode::EnterUrl => "Paste URL | Enter Scrape | ESC Back",
        ImportUrlMode::Scraping => "Scraping in progress... Please wait",
        ImportUrlMode::Done => "R Reanalyze Profile | Enter New URL | ESC Back",
        ImportUrlMode::Error => "Enter Retry | ESC Back",
    };

    let help = Paragraph::new(help_text).style(
        Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg),
    );
    frame.render_widget(help, chunks[2]);
}

fn render_enter_url(frame: &mut Frame, area: Rect, state: &ImportUrlState, palette: &ColorPalette) {
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    // URL input
    let url_display = if state.url_input.is_empty() {
        "Paste a LinkedIn, Facebook, or Twitter/X profile URL here..."
    } else {
        &state.url_input
    };

    let url_style = if state.url_input.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(palette.highlight)
    };

    let input = Paragraph::new(url_display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("URL")
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .style(url_style)
        .wrap(Wrap { trim: false });
    frame.render_widget(input, inner_chunks[0]);

    // Platform detection
    if !state.platform_label.is_empty() {
        let platform_widget = Paragraph::new(state.platform_label.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Platform")
                    .border_style(Style::default().fg(palette.border))
                    .style(Style::default().bg(palette.background)),
            )
            .style(Style::default().fg(palette.info));
        frame.render_widget(platform_widget, inner_chunks[1]);
    }

    // Instructions
    let instructions = Paragraph::new(
        "Supported platforms:\n\
         \n\
         • LinkedIn  - linkedin.com/in/username or linkedin.com/in/username/recent-activity/\n\
         • Facebook  - facebook.com/username\n\
         • Twitter/X - twitter.com/username or x.com/username\n\
         \n\
         The scraper will launch a headless Firefox browser, navigate to the URL,\n\
         scroll to load posts, and extract all visible text content.\n\
         \n\
         Note: Requires Firefox and geckodriver installed. Private/authenticated content\n\
         may not be accessible in headless mode.",
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("How It Works")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)),
    )
    .style(Style::default().fg(palette.foreground))
    .wrap(Wrap { trim: false });
    frame.render_widget(instructions, inner_chunks[2]);
}

fn render_scraping(frame: &mut Frame, area: Rect, state: &ImportUrlState, palette: &ColorPalette) {
    if let Some(ref loading) = state.loading {
        components::render_loading(
            frame,
            area,
            loading,
            palette.border,
            palette.background,
            palette.foreground,
            palette.highlight,
        );
    }
}

fn render_result(frame: &mut Frame, area: Rect, state: &ImportUrlState, palette: &ColorPalette) {
    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(5),
            Constraint::Percentage(30),
        ])
        .split(area);

    if let Some(ref message) = state.result_message {
        let style = if state.is_error {
            Style::default().fg(palette.error)
        } else {
            Style::default().fg(palette.success)
        };

        let result_widget = Paragraph::new(message.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(if state.is_error { "Error" } else { "Success" })
                    .border_style(Style::default().fg(if state.is_error {
                        palette.error
                    } else {
                        palette.success
                    }))
                    .style(Style::default().bg(palette.background)),
            )
            .style(style)
            .wrap(Wrap { trim: false });
        frame.render_widget(result_widget, inner_chunks[1]);
    }
}
