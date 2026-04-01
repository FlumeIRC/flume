use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let fg = Style::default().fg(theme.status_bar_fg);
    let sep = Span::styled(" | ", Style::default().fg(theme.inactive));

    let mut spans = Vec::new();

    // Time of day (left side)
    let time_str = chrono::Local::now().format("%H:%M").to_string();
    spans.push(Span::styled(format!(" {}", time_str), fg));

    // [nick(+modes)]
    let modes = app
        .active_server_state()
        .map(|s| s.user_modes.as_str())
        .unwrap_or("");
    if modes.is_empty() {
        spans.push(Span::styled(format!(" [{}]", app.active_nick()), fg));
    } else {
        spans.push(Span::styled(
            format!(" [{}({})]", app.active_nick(), modes),
            fg,
        ));
    }

    // [#channel(+modes)]
    if let Some(target) = app.active_target() {
        spans.push(sep.clone());
        let chan_modes = app
            .active_server_state()
            .and_then(|ss| ss.active_buf().channel_modes.as_deref())
            .unwrap_or("");
        if chan_modes.is_empty() {
            spans.push(Span::styled(format!("[{}]", target), fg));
        } else {
            spans.push(Span::styled(
                format!("[{}({})]", target, chan_modes),
                fg,
            ));
        }

        // [N users, X ops, Y halfops, Z voiced]
        if let Some(ss) = app.active_server_state() {
            let buf = ss.active_buf();
            if !buf.nicks.is_empty() {
                let total = buf.nicks.len();
                let ops = buf.nicks.iter().filter(|n| n.prefix.contains('@')).count();
                let halfops = buf.nicks.iter().filter(|n| n.prefix.contains('%') && !n.prefix.contains('@')).count();
                let voiced = buf.nicks.iter().filter(|n| n.prefix.contains('+') && !n.prefix.contains('@') && !n.prefix.contains('%')).count();
                spans.push(sep.clone());
                let mut parts = vec![format!("{} users", total)];
                if ops > 0 { parts.push(format!("{} ops", ops)); }
                if halfops > 0 { parts.push(format!("{} halfops", halfops)); }
                if voiced > 0 { parts.push(format!("{} voiced", voiced)); }
                spans.push(Span::styled(format!("[{}]", parts.join(", ")), fg));
            }
        }
    }

    // Connection state (only if not connected)
    let conn_state = app.active_connection_state();
    if conn_state != flume_core::event::ConnectionState::Connected {
        let state_color = match conn_state {
            flume_core::event::ConnectionState::Connecting
            | flume_core::event::ConnectionState::Registering => theme.state_connecting,
            flume_core::event::ConnectionState::Disconnected => theme.state_disconnected,
            _ => theme.state_connected,
        };
        spans.push(sep.clone());
        spans.push(Span::styled(
            format!("{}", conn_state),
            Style::default().fg(state_color),
        ));
    }

    // Other servers with unread
    for name in &app.server_order {
        if Some(name.as_str()) == app.active_server.as_deref() {
            continue;
        }
        if let Some(ss) = app.servers.get(name) {
            let unread = ss.total_unread();
            let highlights = ss.total_highlights();
            if unread > 0 || highlights > 0 {
                spans.push(sep.clone());
                if highlights > 0 {
                    spans.push(Span::styled(
                        format!("{}({}!)", name, unread),
                        Style::default().fg(theme.chat_highlight),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!("{}({})", name, unread),
                        Style::default().fg(theme.unread),
                    ));
                }
            }
        }
    }

    // DCC transfers
    for t in &app.dcc_transfers {
        if let flume_core::dcc::DccTransferState::Active {
            bytes_transferred,
            total,
        } = &t.state
        {
            let name = t.offer.filename.as_deref().unwrap_or("chat");
            let pct = if *total > 0 {
                format!("{}%", (*bytes_transferred * 100) / total)
            } else {
                flume_core::dcc::format_size(*bytes_transferred)
            };
            spans.push(sep.clone());
            spans.push(Span::styled(
                format!("DCC {} {}", name, pct),
                Style::default().fg(theme.unread),
            ));
        }
    }

    // Generation status
    if app.generating {
        spans.push(sep.clone());
        spans.push(Span::styled(
            "generating...",
            Style::default().fg(theme.unread),
        ));
    }

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.status_bar_bg));
    frame.render_widget(bar, area);
}
