use crate::model::{Colors, Theme};

pub fn default_theme() -> Theme {
    Theme {
        bg: "#1e1d40".into(),
        panel: "#2d2b55".into(),
        selected: "#504d7a".into(),
        fg: "#ffffff".into(),
        muted: "#a599e9".into(),
        accent: "#fad000".into(),
    }
}

fn rgb(hex: &str) -> anyhow::Result<(u8, u8, u8)> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 {
        anyhow::bail!("Invalid hex color: {hex}");
    }
    Ok((
        u8::from_str_radix(&h[0..2], 16)?,
        u8::from_str_radix(&h[2..4], 16)?,
        u8::from_str_radix(&h[4..6], 16)?,
    ))
}

fn fg(hex: &str) -> anyhow::Result<String> {
    let (r, g, b) = rgb(hex)?;
    Ok(format!("\x1b[38;2;{r};{g};{b}m"))
}

fn bg(hex: &str) -> anyhow::Result<String> {
    let (r, g, b) = rgb(hex)?;
    Ok(format!("\x1b[48;2;{r};{g};{b}m"))
}

pub fn make_colors(theme: &Theme) -> anyhow::Result<Colors> {
    Ok(Colors {
        bg: bg(&theme.bg)?,
        panel: bg(&theme.panel)?,
        selected: format!("{}{}", bg(&theme.selected)?, fg(&theme.fg)?),
        fg: fg(&theme.fg)?,
        muted: fg(&theme.muted)?,
        accent: fg(&theme.accent)?,
        reset: "\x1b[0m".into(),
        bold: "\x1b[1m".into(),
    })
}
