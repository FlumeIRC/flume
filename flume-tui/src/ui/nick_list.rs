use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let nicks = match app.active_server_state() {
        Some(ss) => &ss.active_buf().nicks,
        None => return,
    };

    if nicks.is_empty() {
        return;
    }

    // Group nicks by status: ops (@), halfops (%), voiced (+), regular
    let mut ops: Vec<&str> = Vec::new();
    let mut halfops: Vec<&str> = Vec::new();
    let mut voiced: Vec<&str> = Vec::new();
    let mut regular: Vec<&str> = Vec::new();

    for cn in nicks {
        if cn.prefix.contains('@') {
            ops.push(&cn.nick);
        } else if cn.prefix.contains('%') {
            halfops.push(&cn.nick);
        } else if cn.prefix.contains('+') {
            voiced.push(&cn.nick);
        } else {
            regular.push(&cn.nick);
        }
    }

    // Alphabetize within each group
    ops.sort_unstable_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    halfops.sort_unstable_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    voiced.sort_unstable_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    regular.sort_unstable_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let height = area.height as usize;
    let dim = Modifier::DIM;
    let mut lines: Vec<Line> = Vec::new();

    if !ops.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" ops ({})", ops.len()),
            Style::default().fg(theme.nick_list_op).add_modifier(dim),
        )));
        for nick in &ops {
            lines.push(Line::from(Span::styled(
                format!(" @{}", nick),
                Style::default().fg(theme.nick_list_op),
            )));
        }
    }

    if !halfops.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" halfops ({})", halfops.len()),
            Style::default().fg(theme.nick_list_voice).add_modifier(dim),
        )));
        for nick in &halfops {
            lines.push(Line::from(Span::styled(
                format!(" %{}", nick),
                Style::default().fg(theme.nick_list_voice),
            )));
        }
    }

    if !voiced.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" voiced ({})", voiced.len()),
            Style::default().fg(theme.nick_list_voice).add_modifier(dim),
        )));
        for nick in &voiced {
            lines.push(Line::from(Span::styled(
                format!(" +{}", nick),
                Style::default().fg(theme.nick_list_voice),
            )));
        }
    }

    if !regular.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" users ({})", regular.len()),
            Style::default().fg(theme.nick_list_fg).add_modifier(dim),
        )));
        for nick in &regular {
            lines.push(Line::from(Span::styled(
                format!("  {}", nick),
                Style::default().fg(theme.nick_list_fg),
            )));
        }
    }

    lines.truncate(height);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
