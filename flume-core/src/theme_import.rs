//! Import external color schemes and convert to Flume themes.
//!
//! Supports:
//! - Omarchy `colors.toml` format (accent, foreground, background, color0-15)

use std::collections::HashMap;

use crate::config::theme::{
    ElementColors, NickColorConfig, ThemeColors, ThemeConfig, ThemeMeta,
};

/// Parsed colors from an external source.
pub struct ImportedColors {
    pub name: String,
    pub foreground: String,
    pub background: String,
    pub accent: String,
    pub selection_fg: Option<String>,
    pub selection_bg: Option<String>,
    /// ANSI palette: color0 through color15.
    pub palette: [String; 16],
}

/// Darken a hex color slightly for UI element backgrounds (title bar, status bar).
fn darken_hex(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return format!("#{}", hex);
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!(
        "#{:02x}{:02x}{:02x}",
        r.saturating_sub(15),
        g.saturating_sub(15),
        b.saturating_sub(15)
    )
}

/// Parse an Omarchy `colors.toml` file.
pub fn parse_omarchy_colors(content: &str, name: &str) -> Result<ImportedColors, String> {
    let table: HashMap<String, String> =
        toml::from_str(content).map_err(|e| format!("Failed to parse colors.toml: {}", e))?;

    let get = |key: &str| -> String {
        table.get(key).cloned().unwrap_or_else(|| "#888888".to_string())
    };

    let mut palette = [
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
        String::new(), String::new(), String::new(), String::new(),
    ];
    for i in 0..16 {
        palette[i] = get(&format!("color{}", i));
    }

    Ok(ImportedColors {
        name: name.to_string(),
        foreground: get("foreground"),
        background: get("background"),
        accent: get("accent"),
        selection_fg: table.get("selection_foreground").cloned(),
        selection_bg: table.get("selection_background").cloned(),
        palette,
    })
}

/// Convert imported colors to a Flume ThemeConfig.
pub fn map_to_theme(colors: &ImportedColors) -> ThemeConfig {
    let bg = &colors.background;
    let fg = &colors.foreground;
    let accent = &colors.accent;
    let dim = &colors.palette[0];        // black/dim
    let red = &colors.palette[1];        // red
    let green = &colors.palette[2];      // green
    let yellow = &colors.palette[3];     // yellow/orange
    let blue = &colors.palette[4];       // blue
    let magenta = &colors.palette[5];    // magenta
    let cyan = &colors.palette[6];       // cyan
    let white = &colors.palette[7];      // white/light

    let bar_bg = darken_hex(bg);

    ThemeConfig {
        meta: ThemeMeta {
            name: colors.name.clone(),
            author: "Imported".to_string(),
            transparent: false,
        },
        colors: ThemeColors {
            background: bg.clone(),
            foreground: fg.clone(),
            highlight: accent.clone(),
            error: red.clone(),
            warning: yellow.clone(),
            success: green.clone(),
        },
        nick_colors: NickColorConfig {
            palette: vec![
                red.clone(),
                yellow.clone(),
                green.clone(),
                cyan.clone(),
                magenta.clone(),
                blue.clone(),
                white.clone(),
                colors.palette[9].clone(),   // bright red
                colors.palette[10].clone(),  // bright green
                colors.palette[14].clone(),  // bright cyan
            ],
        },
        elements: ElementColors {
            title_bar_bg: bar_bg.clone(),
            title_bar_fg: accent.clone(),
            status_bar_bg: bar_bg.clone(),
            status_bar_fg: fg.clone(),
            input_bg: bg.clone(),
            input_fg: fg.clone(),
            nick_list_bg: bg.clone(),
            nick_list_fg: dim.clone(),
            nick_list_op: green.clone(),
            nick_list_voice: cyan.clone(),
            chat_timestamp: dim.clone(),
            chat_nick: fg.clone(),
            chat_message: fg.clone(),
            chat_own_nick: accent.clone(),
            chat_action: magenta.clone(),
            chat_notice: blue.clone(),
            chat_server: yellow.clone(),
            chat_system: white.clone(),
            chat_highlight: accent.clone(),
            chat_url: blue.clone(),
            unread: yellow.clone(),
            inactive: dim.clone(),
            active: accent.clone(),
            scroll_indicator: yellow.clone(),
            search_match_bg: colors.selection_bg.clone().unwrap_or_else(|| accent.clone()),
            search_match_fg: colors.selection_fg.clone().unwrap_or_else(|| bg.clone()),
            state_connected: green.clone(),
            state_connecting: yellow.clone(),
            state_disconnected: red.clone(),
        },
    }
}

/// Convert imported colors to a transparent (glass) Flume ThemeConfig.
pub fn map_to_glass_theme(colors: &ImportedColors) -> ThemeConfig {
    let mut theme = map_to_theme(colors);
    theme.meta.name = format!("{}-glass", colors.name);
    theme.meta.transparent = true;
    theme
}

/// Extract a theme name from a GitHub repo URL.
/// e.g. "https://github.com/OldJobobo/omarchy-miasma-theme" → "omarchy-miasma-theme"
pub fn extract_name_from_url(url: &str) -> String {
    let url = url.trim_end_matches('/');
    url.rsplit('/')
        .next()
        .unwrap_or("imported-theme")
        .to_string()
}

/// Convert a GitHub repo URL to the raw content URL for colors.toml.
/// "https://github.com/user/repo" → "https://raw.githubusercontent.com/user/repo/main/colors.toml"
pub fn github_raw_url(repo_url: &str, file: &str) -> Option<String> {
    let url = repo_url.trim_end_matches('/');
    // Match github.com/<user>/<repo>
    let parts: Vec<&str> = url.split('/').collect();
    let gh_idx = parts.iter().position(|&p| p == "github.com")?;
    if parts.len() < gh_idx + 3 {
        return None;
    }
    let user = parts[gh_idx + 1];
    let repo = parts[gh_idx + 2];
    Some(format!(
        "https://raw.githubusercontent.com/{}/{}/main/{}",
        user, repo, file
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_omarchy() {
        let content = r##"
accent = "#78824b"
foreground = "#c2c2b0"
background = "#222222"
selection_foreground = "#c2c2b0"
selection_background = "#78824b"
color0 = "#666666"
color1 = "#685742"
color2 = "#5f875f"
color3 = "#b36d43"
color4 = "#78824b"
color5 = "#bb7744"
color6 = "#c9a554"
color7 = "#d7c483"
color8 = "#666666"
color9 = "#685742"
color10 = "#5f875f"
color11 = "#b36d43"
color12 = "#78824b"
color13 = "#bb7744"
color14 = "#c9a554"
color15 = "#d7c483"
"##;
        let colors = parse_omarchy_colors(content, "test-theme").unwrap();
        assert_eq!(colors.foreground, "#c2c2b0");
        assert_eq!(colors.background, "#222222");
        assert_eq!(colors.accent, "#78824b");
        assert_eq!(colors.palette[0], "#666666");
        assert_eq!(colors.palette[4], "#78824b");
    }

    #[test]
    fn map_theme() {
        let content = r##"
accent = "#78824b"
foreground = "#c2c2b0"
background = "#222222"
color0 = "#666666"
color1 = "#685742"
color2 = "#5f875f"
color3 = "#b36d43"
color4 = "#78824b"
color5 = "#bb7744"
color6 = "#c9a554"
color7 = "#d7c483"
color8 = "#666666"
color9 = "#685742"
color10 = "#5f875f"
color11 = "#b36d43"
color12 = "#78824b"
color13 = "#bb7744"
color14 = "#c9a554"
color15 = "#d7c483"
"##;
        let colors = parse_omarchy_colors(content, "test").unwrap();
        let theme = map_to_theme(&colors);
        assert_eq!(theme.meta.name, "test");
        assert!(!theme.meta.transparent);
        assert_eq!(theme.elements.active, "#78824b");
        assert_eq!(theme.elements.state_connected, "#5f875f");
    }

    #[test]
    fn glass_variant() {
        let content = r##"
accent = "#78824b"
foreground = "#c2c2b0"
background = "#222222"
color0 = "#666666"
color1 = "#685742"
color2 = "#5f875f"
color3 = "#b36d43"
color4 = "#78824b"
color5 = "#bb7744"
color6 = "#c9a554"
color7 = "#d7c483"
color8 = "#666666"
color9 = "#685742"
color10 = "#5f875f"
color11 = "#b36d43"
color12 = "#78824b"
color13 = "#bb7744"
color14 = "#c9a554"
color15 = "#d7c483"
"##;
        let colors = parse_omarchy_colors(content, "test").unwrap();
        let glass = map_to_glass_theme(&colors);
        assert_eq!(glass.meta.name, "test-glass");
        assert!(glass.meta.transparent);
    }

    #[test]
    fn extract_name() {
        assert_eq!(
            extract_name_from_url("https://github.com/OldJobobo/omarchy-miasma-theme"),
            "omarchy-miasma-theme"
        );
        assert_eq!(
            extract_name_from_url("https://github.com/user/repo/"),
            "repo"
        );
    }

    #[test]
    fn github_url() {
        assert_eq!(
            github_raw_url("https://github.com/OldJobobo/omarchy-miasma-theme", "colors.toml"),
            Some("https://raw.githubusercontent.com/OldJobobo/omarchy-miasma-theme/main/colors.toml".to_string())
        );
    }

    #[test]
    fn darken() {
        assert_eq!(darken_hex("#222222"), "#131313");
        assert_eq!(darken_hex("#000000"), "#000000");
        assert_eq!(darken_hex("#ffffff"), "#f0f0f0");
    }
}
