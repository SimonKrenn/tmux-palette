use std::{collections::HashMap, fs, sync::LazyLock};

use serde::Deserialize;

use crate::{
    cache::CachedConfig,
    config,
    model::{Colors, Theme},
};

pub const DEFAULT_SLUG: &str = "shades-of-purple";

static BUNDLED_THEME_MAP: LazyLock<HashMap<String, Theme>> = LazyLock::new(|| {
    bundled_themes()
        .into_iter()
        .map(|entry| (entry.slug, entry.theme))
        .collect()
});

static USER_THEMES: CachedConfig<HashMap<String, Theme>> = CachedConfig::new("themes");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeListEntry {
    pub slug: String,
    pub name: String,
    pub theme: Theme,
    pub source: ThemeSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeSource {
    User,
    Bundled,
}

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

pub fn bundled_themes() -> Vec<ThemeListEntry> {
    vec![
        bundled(
            "shades-of-purple",
            "Shades of Purple",
            "#1e1d40",
            "#2d2b55",
            "#504d7a",
            "#ffffff",
            "#a599e9",
            "#fad000",
        ),
        bundled(
            "dracula", "Dracula", "#282a36", "#45495d", "#6a6f8f", "#f8f8f2", "#bdc3d8", "#d6acff",
        ),
        bundled(
            "tokyo-night",
            "Tokyo Night",
            "#1a1b26",
            "#34354b",
            "#53567a",
            "#c0caf5",
            "#99a0bf",
            "#7aa2f7",
        ),
        bundled(
            "catppuccin-mocha",
            "Catppuccin Mocha",
            "#1e1e2e",
            "#383857",
            "#5a5a8b",
            "#cdd6f4",
            "#a6a9b9",
            "#89b4fa",
        ),
        bundled(
            "gruvbox-dark",
            "Gruvbox Dark",
            "#282828",
            "#414141",
            "#646464",
            "#ebdbb2",
            "#b7ada4",
            "#8ec07c",
        ),
        bundled(
            "rose-pine",
            "Rosé Pine",
            "#191724",
            "#3c3857",
            "#645c8f",
            "#e0def4",
            "#b1aebf",
            "#9ccfd8",
        ),
        bundled(
            "nord", "Nord", "#2e3440", "#3f4758", "#5c677f", "#d8dee9", "#abb2c0", "#88c0d0",
        ),
        bundled(
            "solarized-dark",
            "Solarized Dark",
            "#002b36",
            "#00333f",
            "#00485b",
            "#839496",
            "#4a8897",
            "#268bd2",
        ),
        bundled(
            "kanagawa-wave",
            "Kanagawa Wave",
            "#1f1f28",
            "#3a3a4b",
            "#5c5c77",
            "#dcd7ba",
            "#b4aa6c",
            "#7e9cd8",
        ),
        bundled(
            "github-dark",
            "GitHub Dark",
            "#101216",
            "#1e2129",
            "#363c4a",
            "#8b949e",
            "#707a85",
            "#6ca4f8",
        ),
        bundled(
            "one-dark", "One Dark", "#21252b", "#2f353d", "#48505e", "#abb2bf", "#8691a3",
            "#61afef",
        ),
        bundled(
            "ayu-dark", "Ayu Dark", "#0b0e14", "#242e41", "#3f5072", "#bfbdb6", "#98958a",
            "#53bdfa",
        ),
    ]
}

fn bundled(
    slug: &str,
    name: &str,
    bg: &str,
    panel: &str,
    selected: &str,
    fg: &str,
    muted: &str,
    accent: &str,
) -> ThemeListEntry {
    ThemeListEntry {
        slug: slug.into(),
        name: name.into(),
        theme: Theme {
            bg: bg.into(),
            panel: panel.into(),
            selected: selected.into(),
            fg: fg.into(),
            muted: muted.into(),
            accent: accent.into(),
        },
        source: ThemeSource::Bundled,
    }
}

fn is_full_theme(theme: &Theme) -> bool {
    !theme.bg.is_empty()
        && !theme.panel.is_empty()
        && !theme.selected.is_empty()
        && !theme.fg.is_empty()
        && !theme.muted.is_empty()
        && !theme.accent.is_empty()
}

pub fn user_themes() -> HashMap<String, Theme> {
    USER_THEMES.get_with(|| {
        let mut out = HashMap::new();
        let dir = config::config_dir().join("themes");
        let Ok(entries) = fs::read_dir(dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(slug) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let Ok(raw) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(theme) = serde_json::from_str::<Theme>(&raw) else {
                continue;
            };
            if is_full_theme(&theme) {
                out.insert(slug.to_string(), theme);
            }
        }
        out
    })
}

pub fn list_themes() -> Vec<ThemeListEntry> {
    let users = user_themes();
    let mut entries: Vec<ThemeListEntry> = users
        .iter()
        .map(|(slug, theme)| ThemeListEntry {
            slug: slug.clone(),
            name: slug.clone(),
            theme: theme.clone(),
            source: ThemeSource::User,
        })
        .collect();
    entries.extend(
        bundled_themes()
            .into_iter()
            .filter(|entry| !users.contains_key(&entry.slug)),
    );
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

pub fn resolve_theme(theme: Option<&str>) -> anyhow::Result<Theme> {
    let Some(slug) = theme else {
        return Ok(default_theme());
    };
    if let Some(theme) = user_themes().get(slug).cloned() {
        return Ok(theme);
    }
    if let Some(theme) = BUNDLED_THEME_MAP.get(slug).cloned() {
        return Ok(theme);
    }
    anyhow::bail!("Unknown theme: {slug}")
}

#[derive(Debug, Clone, Deserialize, Default)]
struct UserThemeFile {
    name: Option<String>,
    bg: Option<String>,
    panel: Option<String>,
    selected: Option<String>,
    fg: Option<String>,
    muted: Option<String>,
    accent: Option<String>,
}

pub fn resolve_active_theme(declared: Option<&str>) -> anyhow::Result<Theme> {
    let file: Option<UserThemeFile> = config::load_json("theme.json", None);
    if let Some(name) = file.as_ref().and_then(|file| file.name.as_deref()) {
        return resolve_theme(Some(name));
    }
    let mut theme = resolve_theme(declared)?;
    if let Some(file) = file {
        if let Some(bg) = file.bg {
            theme.bg = bg;
        }
        if let Some(panel) = file.panel {
            theme.panel = panel;
        }
        if let Some(selected) = file.selected {
            theme.selected = selected;
        }
        if let Some(fg) = file.fg {
            theme.fg = fg;
        }
        if let Some(muted) = file.muted {
            theme.muted = muted;
        }
        if let Some(accent) = file.accent {
            theme.accent = accent;
        }
    }
    Ok(theme)
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
