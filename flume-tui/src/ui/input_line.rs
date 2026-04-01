use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use flume_core::config::keybindings::KeybindingMode;

use crate::app::{App, InputMode, ViMode};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let (prompt_spans, display_text) = match &app.input_mode {
        InputMode::Passphrase(label) => {
            let prompt = format!("{}: ", label);
            let masked = "*".repeat(app.input.len());
            (vec![Span::styled(prompt, Style::default().fg(theme.input_fg))], masked)
        }
        InputMode::Normal => {
            let mut spans = Vec::new();

            // Vi mode indicator
            if app.keybinding_mode == KeybindingMode::Vi {
                let (label, style) = match app.vi_mode {
                    ViMode::Normal => (
                        "[N] ",
                        Style::default()
                            .fg(theme.status_bar_fg)
                            .bg(theme.status_bar_bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    ViMode::Insert => (
                        "[I] ",
                        Style::default()
                            .fg(theme.input_fg)
                            .add_modifier(Modifier::BOLD),
                    ),
                };
                spans.push(Span::styled(label, style));
            }

            let target_prompt = match app.active_target() {
                Some(target) => format!("[{}] ", target),
                None => format!("[{}] ", app.active_server_name()),
            };
            spans.push(Span::styled(target_prompt, Style::default().fg(theme.input_fg)));

            (spans, app.input.clone())
        }
    };

    let prompt_len: usize = prompt_spans.iter().map(|s| s.width()).sum();
    let available_width = (area.width as usize).saturating_sub(prompt_len);

    // Calculate horizontal scroll offset to keep cursor visible
    let cursor_in_text = app.cursor_pos;
    let scroll_offset = if cursor_in_text >= available_width {
        cursor_in_text - available_width + 1
    } else {
        0
    };

    // Slice the display text to the visible window
    let visible_text = if scroll_offset < display_text.len() {
        let end = (scroll_offset + available_width).min(display_text.len());
        &display_text[scroll_offset..end]
    } else {
        ""
    };

    let mut all_spans = prompt_spans;
    all_spans.push(Span::raw(visible_text.to_string()));

    let line = Line::from(all_spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);

    // Position cursor relative to the visible window
    let cursor_x = area.x + prompt_len as u16 + (cursor_in_text - scroll_offset) as u16;
    let cursor_y = area.y;
    frame.set_cursor_position((cursor_x.min(area.x + area.width - 1), cursor_y));
}
