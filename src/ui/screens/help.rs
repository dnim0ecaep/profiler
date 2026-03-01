use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::ui::ColorPalette;

pub fn render(frame: &mut Frame, area: Rect, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);
    
    // Title
    let title = Paragraph::new("Help & Keyboard Shortcuts")
        .style(Style::default()
            .fg(palette.title)
            .bg(palette.background)
            .add_modifier(Modifier::BOLD))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(palette.border)));
    frame.render_widget(title, chunks[0]);
    
    // Content - split into two columns
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // Left column: Navigation & Global
    render_left_column(frame, content_chunks[0], palette);
    
    // Right column: Screen-specific shortcuts
    render_right_column(frame, content_chunks[1], palette);
    
    // Help bar
    let help = Paragraph::new("Press ESC or ? to close help")
        .style(Style::default()
            .bg(palette.help_bar_bg)
            .fg(palette.help_bar_fg));
    frame.render_widget(help, chunks[2]);
}

fn render_left_column(frame: &mut Frame, area: Rect, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(5),
        ])
        .split(area);
    
    // Global shortcuts
    let global_items = vec![
        shortcut_line("Ctrl+Q", "Quit application", palette),
        shortcut_line("Ctrl+T", "Cycle theme", palette),
        shortcut_line("ESC", "Go back / Dashboard", palette),
        shortcut_line("↑ / ↓", "Navigate lists", palette),
        shortcut_line("Enter", "Select / Confirm", palette),
        shortcut_line("?", "Toggle this help screen", palette),
    ];
    
    let global = List::new(global_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Global Shortcuts")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(global, chunks[0]);
    
    // Dashboard shortcuts
    let dashboard_items = vec![
        shortcut_line("N", "New Profile", palette),
        shortcut_line("C", "Coach Message", palette),
        shortcut_line("P", "View Profiles", palette),
        shortcut_line("M", "Compare Profiles", palette),
        shortcut_line("S", "Settings", palette),
        shortcut_line("L", "View Logs", palette),
    ];
    
    let dashboard = List::new(dashboard_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Dashboard")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(dashboard, chunks[1]);
    
    // Themes
    let theme_items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("Available themes: ", Style::default().fg(palette.foreground)),
        ])),
        ListItem::new(Line::from(Span::styled("  1. Midnight Commander", Style::default().fg(palette.info)))),
        ListItem::new(Line::from(Span::styled("  2. Default", Style::default().fg(palette.info)))),
        ListItem::new(Line::from(Span::styled("  3. Dark", Style::default().fg(palette.info)))),
        ListItem::new(Line::from(Span::styled("  4. Minimal", Style::default().fg(palette.info)))),
        ListItem::new(Line::from(Span::styled("  5. Monokai", Style::default().fg(palette.info)))),
        ListItem::new(Line::from(Span::styled("  6. Solarized Dark", Style::default().fg(palette.info)))),
    ];
    
    let themes = List::new(theme_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Themes (Ctrl+T to cycle)")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(themes, chunks[2]);
}

fn render_right_column(frame: &mut Frame, area: Rect, palette: &ColorPalette) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Min(5),
        ])
        .split(area);
    
    // Profile View shortcuts
    let profile_items = vec![
        shortcut_line("C", "Coach with this profile", palette),
        shortcut_line("D", "Delete profile", palette),
        shortcut_line("M", "Compare with another profile", palette),
        shortcut_line("ESC", "Back to dashboard", palette),
    ];
    
    let profile = List::new(profile_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Profile View")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(profile, chunks[0]);
    
    // Create Profile shortcuts
    let create_items = vec![
        shortcut_line("↑ / ↓", "Select profile type", palette),
        shortcut_line("Enter", "Confirm selection / name", palette),
        shortcut_line("Ctrl+S", "Save & analyze text", palette),
        shortcut_line("ESC", "Cancel", palette),
    ];
    
    let create = List::new(create_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Create Profile")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(create, chunks[1]);
    
    // Compare shortcuts
    let compare_items = vec![
        shortcut_line("↑ / ↓", "Navigate profile list", palette),
        shortcut_line("Enter", "Select profile", palette),
        shortcut_line("N", "New comparison", palette),
        shortcut_line("ESC", "Back", palette),
    ];
    
    let compare = List::new(compare_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Compare Profiles")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(compare, chunks[2]);
    
    // About / Tips
    let tips_items = vec![
        ListItem::new(Line::from(Span::styled(
            "Tips:",
            Style::default().fg(palette.title).add_modifier(Modifier::BOLD),
        ))),
        ListItem::new(Line::from(Span::styled(
            "• Paste text with Ctrl+V in text areas",
            Style::default().fg(palette.foreground),
        ))),
        ListItem::new(Line::from(Span::styled(
            "• More text = better profile analysis",
            Style::default().fg(palette.foreground),
        ))),
        ListItem::new(Line::from(Span::styled(
            "• Profiles saved in ./data/profiles/",
            Style::default().fg(palette.foreground),
        ))),
        ListItem::new(Line::from(Span::styled(
            "• Edit TOML files directly if needed",
            Style::default().fg(palette.foreground),
        ))),
        ListItem::new(Line::from(Span::styled(
            "• Drop files in sources/ for re-analysis",
            Style::default().fg(palette.foreground),
        ))),
    ];
    
    let tips = List::new(tips_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Tips & Info")
            .border_style(Style::default().fg(palette.border))
            .style(Style::default().bg(palette.background)));
    frame.render_widget(tips, chunks[3]);
}

fn shortcut_line<'a>(key: &'a str, desc: &'a str, palette: &ColorPalette) -> ListItem<'a> {
    ListItem::new(Line::from(vec![
        Span::styled(
            format!("  {:>8}  ", key),
            Style::default().fg(palette.highlight).add_modifier(Modifier::BOLD),
        ),
        Span::styled(desc, Style::default().fg(palette.foreground)),
    ]))
}
