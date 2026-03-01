use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::PersonProfile;
use crate::ui::ColorPalette;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    profile: &PersonProfile,
    palette: &ColorPalette,
    scroll_offset: usize,
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
    let title = Paragraph::new(format!("🔍 Profile Reasoning: {}", profile.name))
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

    // Build all content lines
    let lines = build_reasoning_lines(profile, palette);

    // Apply scroll offset and render
    let visible_height = chunks[1].height.saturating_sub(2) as usize; // minus border
    let total_lines = lines.len();
    let clamped_offset = scroll_offset.min(total_lines.saturating_sub(visible_height));

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(clamped_offset)
        .take(visible_height)
        .collect();

    let scroll_indicator = if total_lines > visible_height {
        format!(
            " [{}-{}/{}]",
            clamped_offset + 1,
            (clamped_offset + visible_height).min(total_lines),
            total_lines
        )
    } else {
        String::new()
    };

    let content = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("How This Profile Was Built{}", scroll_indicator))
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.background)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(content, chunks[1]);

    // Help bar
    let help = Paragraph::new("↑↓ Scroll | Ctrl+T Theme | ESC Back")
        .style(
            Style::default()
                .bg(palette.help_bar_bg)
                .fg(palette.help_bar_fg),
        );
    frame.render_widget(help, chunks[2]);
}

fn build_reasoning_lines<'a>(profile: &PersonProfile, palette: &ColorPalette) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = Vec::new();

    match &profile.reasoning {
        Some(reasoning) => {
            // Overall summary section
            lines.push(Line::from(vec![Span::styled(
                "── Overview ──────────────────────────────────────────────",
                Style::default()
                    .fg(palette.title)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            // Wrap the overall summary
            for line_text in word_wrap(&reasoning.overall_summary, 76) {
                lines.push(Line::from(Span::styled(
                    line_text,
                    Style::default().fg(palette.foreground),
                )));
            }
            lines.push(Line::from(""));

            // Confidence
            lines.push(Line::from(vec![
                Span::styled(
                    "  Confidence: ",
                    Style::default()
                        .fg(palette.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:.0}%", profile.confidence * 100.0),
                    Style::default().fg(confidence_color(profile.confidence, palette)),
                ),
            ]));
            lines.push(Line::from(""));

            // Per-trait explanations
            for trait_exp in &reasoning.trait_explanations {
                // Trait header with value
                lines.push(Line::from(vec![Span::styled(
                    format!(
                        "── {} : {} ──────────────────────────────────────",
                        trait_exp.trait_name, trait_exp.value_chosen
                    ),
                    Style::default()
                        .fg(palette.info)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                // Reasoning explanation
                for line_text in word_wrap(&trait_exp.reasoning, 74) {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(line_text, Style::default().fg(palette.foreground)),
                    ]));
                }

                // Supporting phrases
                if !trait_exp.supporting_phrases.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            "Supporting phrases:",
                            Style::default()
                                .fg(palette.secondary)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));

                    for phrase in &trait_exp.supporting_phrases {
                        // Truncate very long phrases for display
                        let display_phrase = if phrase.len() > 70 {
                            format!("{}...", &phrase[..67])
                        } else {
                            phrase.clone()
                        };
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled("\"", Style::default().fg(palette.warning)),
                            Span::styled(
                                display_phrase,
                                Style::default()
                                    .fg(palette.warning)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                            Span::styled("\"", Style::default().fg(palette.warning)),
                        ]));
                    }
                }

                lines.push(Line::from(""));
            }

            // Caveats section
            if !reasoning.caveats.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    "── ⚠ Caveats ─────────────────────────────────────────────",
                    Style::default()
                        .fg(palette.warning)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(""));

                for caveat in &reasoning.caveats {
                    for (i, line_text) in word_wrap(caveat, 72).into_iter().enumerate() {
                        if i == 0 {
                            lines.push(Line::from(vec![
                                Span::raw("  "),
                                Span::styled("• ", Style::default().fg(palette.warning)),
                                Span::styled(line_text, Style::default().fg(palette.secondary)),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::raw("    "),
                                Span::styled(line_text, Style::default().fg(palette.secondary)),
                            ]));
                        }
                    }
                }
                lines.push(Line::from(""));
            }
        }
        None => {
            // No reasoning available
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  No reasoning data available for this profile.",
                Style::default()
                    .fg(palette.warning)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  This profile may have been created before the reasoning feature",
                Style::default().fg(palette.secondary),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "  was added, or was imported without reasoning data.",
                Style::default().fg(palette.secondary),
            )]));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "  To get reasoning, create a new profile from text using 'N' from the dashboard.",
                Style::default().fg(palette.foreground),
            )]));
        }
    }

    lines
}

/// Simple word wrapping for a string to fit within max_width characters
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() > max_width {
            lines.push(current_line);
            current_line = word.to_string();
        } else {
            current_line.push(' ');
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn confidence_color(confidence: f32, palette: &ColorPalette) -> Color {
    if confidence >= 0.7 {
        palette.success
    } else if confidence >= 0.5 {
        palette.warning
    } else {
        palette.error
    }
}
