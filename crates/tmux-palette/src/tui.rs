use std::{env, io::Write, path::PathBuf, time::Duration};

use anyhow::Context;
use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{self, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Terminal,
};

use crate::{
    actions::dispatch_to_file,
    config,
    fuzzy::default_filter,
    model::{Action, Item, PaletteDef, Theme},
    palettes,
    render::{build_rows, first_selectable, is_selectable, step, Row, RowAction},
    text::{display_width, truncate},
    theme,
};

type TuiTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Clone)]
struct NavState {
    def: PaletteDef,
    name: String,
    selected: usize,
    scroll: usize,
    filter: String,
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<(Self, TuiTerminal)> {
        terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            EnableMouseCapture,
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        let guard = Self;
        let terminal = (|| -> anyhow::Result<TuiTerminal> {
            let backend = CrosstermBackend::new(std::io::stdout());
            let mut terminal = Terminal::new(backend)?;
            terminal.clear()?;
            Ok(terminal)
        })();
        match terminal {
            Ok(terminal) => Ok((guard, terminal)),
            Err(err) => {
                drop(guard);
                Err(err)
            }
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = execute!(
            out,
            DisableMouseCapture,
            cursor::Show,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
        let _ = write!(out, "\x1b]112\x07");
        let _ = out.flush();
        let _ = terminal::disable_raw_mode();
    }
}

fn prev_boundary(s: &str, from: usize) -> usize {
    s[..from].char_indices().last().map(|(i, _)| i).unwrap_or(0)
}

fn next_boundary(s: &str, from: usize) -> usize {
    s[from..]
        .char_indices()
        .nth(1)
        .map(|(i, _)| from + i)
        .unwrap_or(s.len())
}

fn word_back(s: &str, from: usize) -> usize {
    let mut i = from;
    while i > 0 {
        let prev = prev_boundary(s, i);
        if !s[prev..i].chars().all(char::is_whitespace) {
            break;
        }
        i = prev;
    }
    while i > 0 {
        let prev = prev_boundary(s, i);
        if s[prev..i].chars().all(char::is_whitespace) {
            break;
        }
        i = prev;
    }
    i
}

fn word_forward(s: &str, from: usize) -> usize {
    let mut i = from;
    while i < s.len() {
        let next = next_boundary(s, i);
        if !s[i..next].chars().all(char::is_whitespace) {
            break;
        }
        i = next;
    }
    while i < s.len() {
        let next = next_boundary(s, i);
        if s[i..next].chars().all(char::is_whitespace) {
            break;
        }
        i = next;
    }
    i
}

fn shell_size_expr(spec: &str, axis: &str, pad: u16) -> String {
    if let Some(pct) = spec.strip_suffix('%').and_then(|s| s.parse::<u16>().ok()) {
        return format!(
            "$(( $(tmux display-message -p '#{{{axis}}}') * {pct} / 100 - {} ))",
            pad * 2
        );
    }
    spec.parse::<u16>()
        .map(|n| n.saturating_sub(pad * 2).max(1).to_string())
        .unwrap_or_else(|_| spec.to_string())
}

fn apply_user_overrides(items: &[Item]) -> Vec<Item> {
    let shortcuts = config::user_shortcuts();
    let aliases = config::user_aliases();
    items
        .iter()
        .cloned()
        .map(|mut item| {
            if item.shortcut.is_none() {
                item.shortcut = shortcuts.get(&item.title).cloned();
            }
            if let Some(extra) = aliases.get(&item.title) {
                item.aliases.extend(extra.clone());
            }
            item
        })
        .collect()
}

fn clamp_scroll(rows: &[Row], list_height: usize, selected: usize, mut scroll: usize) -> usize {
    if let Some(row_idx) = rows
        .iter()
        .position(|row| matches!(row, Row::Item { item_index, .. } if *item_index == selected))
    {
        if row_idx < scroll {
            scroll = row_idx;
        }
        if row_idx >= scroll + list_height {
            scroll = row_idx - list_height + 1;
        }
    }
    scroll.min(rows.len().saturating_sub(list_height))
}

fn footer_text(selectable_count: usize, empty_text: &str) -> String {
    if selectable_count == 0 {
        return empty_text.to_string();
    }
    let noun = if selectable_count == 1 {
        "command"
    } else {
        "commands"
    };
    format!("enter select   up/down move   {selectable_count} {noun}")
}

fn theme_from_item(item: Option<&Item>) -> Option<Theme> {
    item.and_then(|item| item.data.as_ref())
        .and_then(|data| serde_json::from_value(data.clone()).ok())
}

fn rat_color(hex: &str) -> Color {
    if hex.eq_ignore_ascii_case("default") {
        return Color::Reset;
    }
    let lower = hex.to_ascii_lowercase();
    if let Some(index) = lower
        .strip_prefix("colour")
        .or_else(|| lower.strip_prefix("color"))
        .and_then(|value| value.parse().ok())
    {
        return Color::Indexed(index);
    }
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 {
        return Color::Reset;
    }
    let Ok(r) = u8::from_str_radix(&h[0..2], 16) else {
        return Color::Reset;
    };
    let Ok(g) = u8::from_str_radix(&h[2..4], 16) else {
        return Color::Reset;
    };
    let Ok(b) = u8::from_str_radix(&h[4..6], 16) else {
        return Color::Reset;
    };
    Color::Rgb(r, g, b)
}

#[derive(Clone, Copy)]
struct PaletteStyles {
    panel: Style,
    selected: Style,
    fg: Style,
    selected_fg: Style,
    muted: Style,
    accent: Style,
    bold_fg: Style,
    bold_accent: Style,
}

impl PaletteStyles {
    fn new(theme: &Theme) -> Self {
        let panel_bg = rat_color(&theme.panel);
        let selected_bg = rat_color(&theme.selected);
        let fg = rat_color(&theme.fg);
        let muted = rat_color(&theme.muted);
        let accent = rat_color(&theme.accent);
        Self {
            panel: Style::default().bg(panel_bg),
            selected: Style::default().bg(selected_bg),
            fg: Style::default().fg(fg).bg(panel_bg),
            selected_fg: Style::default().fg(fg).bg(selected_bg),
            muted: Style::default().fg(muted).bg(panel_bg),
            accent: Style::default().fg(accent).bg(panel_bg),
            bold_fg: Style::default()
                .fg(fg)
                .bg(panel_bg)
                .add_modifier(Modifier::BOLD),
            bold_accent: Style::default()
                .fg(accent)
                .bg(panel_bg)
                .add_modifier(Modifier::BOLD),
        }
    }

    fn row_bg(self, active: bool) -> Style {
        if active {
            self.selected
        } else {
            self.panel
        }
    }

    fn fg_for(self, active: bool) -> Style {
        if active {
            self.selected_fg
        } else {
            self.muted
        }
    }

    fn accent_for(self, active: bool) -> Style {
        let bg = if active {
            self.selected.bg.unwrap_or(Color::Reset)
        } else {
            self.panel.bg.unwrap_or(Color::Reset)
        };
        Style::default()
            .fg(self.accent.fg.unwrap_or(Color::Reset))
            .bg(bg)
    }

    fn muted_for(self, active: bool) -> Style {
        let bg = if active {
            self.selected.bg.unwrap_or(Color::Reset)
        } else {
            self.panel.bg.unwrap_or(Color::Reset)
        };
        Style::default()
            .fg(self.muted.fg.unwrap_or(Color::Reset))
            .bg(bg)
    }

    fn bold_fg_for(self, active: bool) -> Style {
        self.fg_for(active).add_modifier(Modifier::BOLD)
    }
}

fn span_width(spans: &[Span<'static>]) -> usize {
    spans
        .iter()
        .map(|span| display_width(span.content.as_ref()))
        .sum()
}

fn data_str<'a>(data: &'a serde_json::Value, key: &str) -> &'a str {
    data.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

fn pane_str<'a>(data: &'a serde_json::Value, key: &str) -> &'a str {
    data.get("pane")
        .and_then(|pane| pane.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

fn pane_bool(data: &serde_json::Value, key: &str) -> bool {
    data.get("pane")
        .and_then(|pane| pane.get(key))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn shorten_path(path: &str) -> String {
    if let Some(home) = env::var_os("HOME").and_then(|home| home.into_string().ok()) {
        if path == home {
            return "~".into();
        }
        if let Some(rest) = path.strip_prefix(&(home + "/")) {
            return format!("~/{rest}");
        }
    }
    path.into()
}

fn default_item_line(
    item: &Item,
    styles: PaletteStyles,
    active: bool,
    width: usize,
) -> Line<'static> {
    let row = styles.row_bg(active);
    let title_style = if active {
        styles.bold_fg_for(true)
    } else {
        styles.muted_for(false)
    };
    let shortcut = item.shortcut.clone().unwrap_or_default();
    let icon_text = item.icon.clone().unwrap_or_else(|| " ".into());
    let mut left = vec![
        Span::styled(if active { "▌" } else { " " }, styles.accent_for(active)),
        Span::styled(" ", row),
    ];
    if let Some(icon) = &item.icon {
        let icon_style = item
            .icon_color
            .as_deref()
            .map(rat_color)
            .map(|fg| Style::default().fg(fg).bg(row.bg.unwrap_or(Color::Reset)))
            .unwrap_or_else(|| styles.accent_for(active));
        left.push(Span::styled(icon.clone(), icon_style));
    } else {
        left.push(Span::styled(" ", row));
    }
    left.push(Span::styled("  ", row));
    left.push(Span::styled(item.title.clone(), title_style));
    if let Some(alias) = item.aliases.first() {
        left.push(Span::styled("  ", row));
        left.push(Span::styled(format!(" {alias} "), styles.fg_for(active)));
    }
    if let Some(description) = &item.description {
        left.push(Span::styled(
            format!(" - {description}"),
            styles.muted_for(active),
        ));
    }
    let shortcut_width = display_width(&shortcut);
    let max_left = width.saturating_sub(shortcut_width + 1);
    if span_width(&left) > max_left {
        let fixed_width = 1 + 1 + display_width(&icon_text) + 2;
        let title_budget = max_left.saturating_sub(fixed_width).max(1);
        left = vec![
            Span::styled(if active { "▌" } else { " " }, styles.accent_for(active)),
            Span::styled(" ", row),
            Span::styled(icon_text, styles.accent_for(active)),
            Span::styled("  ", row),
            Span::styled(truncate(&item.title, title_budget), title_style),
        ];
    }
    let gap = width
        .saturating_sub(span_width(&left) + shortcut_width)
        .max(1);
    left.push(Span::styled(" ".repeat(gap), row));
    if !shortcut.is_empty() {
        left.push(Span::styled(
            shortcut,
            if active {
                styles.accent_for(true)
            } else {
                styles.muted_for(false)
            },
        ));
    }
    Line::from(left).style(row)
}

fn find_pane_line(item: &Item, styles: PaletteStyles, active: bool, width: usize) -> Line<'static> {
    let row = styles.row_bg(active);
    let Some(data) = item.data.as_ref() else {
        return default_item_line(item, styles, active, width);
    };
    match data.get("kind").and_then(|v| v.as_str()) {
        Some("session") => {
            let session = data_str(data, "session");
            let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            let path = data_str(data, "path");
            let mut spans = Vec::new();
            if data.get("isCurrent").and_then(|v| v.as_bool()) == Some(true) {
                spans.push(Span::styled("▶ ", styles.accent_for(active)));
            } else {
                spans.push(Span::styled("  ", row));
            }
            spans.push(Span::styled(
                session.to_string(),
                styles.bold_accent.bg(row.bg.unwrap_or(Color::Reset)),
            ));
            spans.push(Span::styled(
                format!(" ({count})"),
                styles.muted_for(active),
            ));
            if !path.is_empty() {
                spans.push(Span::styled(
                    format!("  {}", shorten_path(path)),
                    styles.muted_for(active),
                ));
            }
            Line::from(spans).style(row)
        }
        Some("window") => {
            let prefix = data_str(data, "treePrefix");
            let name = data_str(data, "windowName");
            Line::from(vec![
                Span::styled(prefix.to_string(), styles.muted_for(active)),
                Span::styled(
                    name.to_string(),
                    if active {
                        styles.bold_fg_for(true)
                    } else {
                        styles.fg
                    },
                ),
            ])
            .style(row)
        }
        Some("pane") => {
            let prefix = data_str(data, "treePrefix");
            let title = pane_str(data, "paneTitle");
            let is_current = pane_bool(data, "isCurrent");
            let pane_active = pane_bool(data, "paneActive");
            let marker_style = if is_current {
                styles.accent_for(active)
            } else if pane_active {
                Style::default()
                    .fg(Color::Rgb(166, 227, 161))
                    .bg(row.bg.unwrap_or(Color::Reset))
            } else {
                styles.muted_for(active)
            };
            let marker = if is_current {
                "▶"
            } else if pane_active {
                "●"
            } else {
                "○"
            };
            let title_style = if active {
                styles.bold_fg_for(true)
            } else if is_current {
                styles.fg
            } else {
                styles.muted
            };
            let agent = pane_str(data, "agent");
            let right = format!(
                "{}.{}",
                pane_str(data, "windowIndex"),
                pane_str(data, "paneIndex")
            );
            let agent_width = if agent.is_empty() {
                0
            } else {
                2 + display_width(agent)
            };
            let fixed_width =
                display_width(prefix) + 1 + 1 + agent_width + 1 + display_width(&right);
            let title_budget = width.saturating_sub(fixed_width).max(1);
            let title = truncate(title, title_budget);
            let mut spans = vec![
                Span::styled(prefix.to_string(), styles.muted_for(active)),
                Span::styled(marker, marker_style),
                Span::styled(" ", row),
                Span::styled(
                    title.clone(),
                    title_style.bg(row.bg.unwrap_or(Color::Reset)),
                ),
            ];
            let mut left_width = display_width(prefix) + 1 + 1 + display_width(&title);
            if !agent.is_empty() {
                spans.push(Span::styled(format!("  {agent}"), styles.muted_for(active)));
                left_width += 2 + display_width(agent);
            }
            spans.push(Span::styled(
                " ".repeat(
                    width
                        .saturating_sub(left_width + display_width(&right))
                        .max(1),
                ),
                row,
            ));
            spans.push(Span::styled(right, styles.muted_for(active)));
            Line::from(spans).style(row)
        }
        _ => default_item_line(item, styles, active, width),
    }
}

fn row_line(
    row: &Row,
    styles: PaletteStyles,
    active: bool,
    width: usize,
    palette_name: &str,
) -> Line<'static> {
    match row {
        Row::Category { category } => Line::from(Span::styled(
            category.clone(),
            styles
                .bold_accent
                .bg(styles.panel.bg.unwrap_or(Color::Reset)),
        ))
        .style(styles.panel),
        Row::Item { item, .. } if palette_name == "find-pane" => {
            find_pane_line(item, styles, active, width)
        }
        Row::Item { item, .. } => default_item_line(item, styles, active, width),
    }
}

fn initial_selected(name: &str, items: &[Item]) -> usize {
    if name == "find-pane" {
        if let Some(idx) = items.iter().position(|item| {
            item.data
                .as_ref()
                .and_then(|data| data.get("pane"))
                .and_then(|pane| pane.get("isCurrent"))
                .and_then(|value| value.as_bool())
                == Some(true)
        }) {
            return idx;
        }
    }
    first_selectable(items).unwrap_or(0)
}

pub struct PaletteRunner {
    current_def: PaletteDef,
    current_name: String,
    items: Vec<Item>,
    selected: usize,
    scroll: usize,
    filter: String,
    filter_cursor: usize,
    row_actions: Vec<RowAction>,
    esc_action: Option<(usize, usize, usize)>,
    stack: Vec<NavState>,
    cmd_file: Option<PathBuf>,
}

impl PaletteRunner {
    pub fn new(def: PaletteDef, initial_name: impl Into<String>) -> Self {
        let current_name = initial_name.into();
        let items = apply_user_overrides(&def.items);
        let selected = initial_selected(&current_name, &items);
        Self {
            current_def: def,
            current_name,
            items,
            selected,
            scroll: 0,
            filter: String::new(),
            filter_cursor: 0,
            row_actions: Vec::new(),
            esc_action: None,
            stack: Vec::new(),
            cmd_file: env::var_os("TMUX_PALETTE_CMD").map(PathBuf::from),
        }
    }

    fn load_def(&mut self, name: String, def: PaletteDef) {
        self.items = apply_user_overrides(&def.items);
        self.selected = initial_selected(&name, &self.items);
        self.scroll = 0;
        self.filter.clear();
        self.filter_cursor = 0;
        self.row_actions.clear();
        self.esc_action = None;
        self.current_name = name;
        self.current_def = def;
    }

    fn visible(&self) -> Vec<Item> {
        let needle = self.filter.trim();
        if needle.is_empty() {
            return self.items.clone();
        }
        if self.current_name == "find-pane" {
            return palettes::filter_tree(&self.items, needle);
        }
        default_filter(&self.items, needle)
    }

    fn ensure_selectable(&mut self, vis: &[Item]) {
        if !is_selectable(vis.get(self.selected)) {
            self.selected = first_selectable(vis).unwrap_or(0);
        }
    }

    fn colors(&self, vis: &[Item]) -> anyhow::Result<(Theme, crate::model::Colors)> {
        let theme = if self.current_name == "themes" {
            theme_from_item(vis.get(self.selected)).unwrap_or(theme::resolve_active_theme(None)?)
        } else {
            theme::resolve_active_theme(None)?
        };
        let colors = theme::make_colors(&theme)?;
        Ok((theme, colors))
    }

    fn render(&mut self, terminal: &mut TuiTerminal) -> anyhow::Result<()> {
        let area = terminal.size()?;
        let width = usize::from(area.width).max(1);
        let height = usize::from(area.height).max(1);
        let vis = self.visible();
        self.ensure_selectable(&vis);
        let (theme, _colors) = self.colors(&vis)?;
        let styles = PaletteStyles::new(&theme);
        let rows = build_rows(&vis, self.current_def.grouped, !self.filter.is_empty());
        let bordered = env::var("TMUX_PALETTE_BORDERED").ok().as_deref() == Some("1");
        let chrome_rows = if bordered { 5 } else { 7 };
        let list_height = height.saturating_sub(chrome_rows).max(1);
        self.scroll = clamp_scroll(&rows, list_height, self.selected, self.scroll);
        let pad_x = env::var("TMUX_PALETTE_PADX")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3usize);
        let body_width = width.saturating_sub(pad_x * 2).max(1);
        let title = self.current_def.title.as_deref().unwrap_or("Commands");
        let selectable_count = vis.iter().filter(|item| is_selectable(Some(item))).count();
        let empty = self
            .current_def
            .empty_text
            .as_deref()
            .unwrap_or("No results");
        let header_y = if bordered { 1 } else { 2 };
        let esc_width = display_width("esc");
        self.esc_action = Some((
            header_y,
            width.saturating_sub(pad_x + esc_width).max(1),
            width.saturating_sub(pad_x).saturating_add(1),
        ));
        let list_y = if bordered { 4 } else { 5 };
        self.row_actions = rows
            .iter()
            .enumerate()
            .skip(self.scroll)
            .take(list_height)
            .filter_map(|(row_idx, row)| match row {
                Row::Item { item_index, .. } => Some(RowAction {
                    y: list_y + (row_idx - self.scroll),
                    item_index: *item_index,
                }),
                Row::Category { .. } => None,
            })
            .collect();
        let filter = self.filter.clone();
        let current_name = self.current_name.clone();
        let selected = self.selected;
        let scroll = self.scroll;
        let footer_text = footer_text(selectable_count, empty);
        let cursor_offset = display_width(&self.filter[..self.filter_cursor]);
        let cursor_col = (pad_x + 2 + cursor_offset).min(pad_x + 2 + body_width.saturating_sub(1));
        let search_row = if bordered { 1 } else { 2 };

        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(Block::default().style(styles.panel), area);
            let rows_layout = if bordered {
                Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(area)
            } else {
                Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(area)
            };
            let (header_area, search_area, list_area, footer_area) = if bordered {
                (
                    rows_layout[0],
                    rows_layout[1],
                    rows_layout[3],
                    rows_layout[5],
                )
            } else {
                (
                    rows_layout[1],
                    rows_layout[2],
                    rows_layout[4],
                    rows_layout[6],
                )
            };
            let header_gap = body_width.saturating_sub(display_width(title) + esc_width);
            let header = Line::from(vec![
                Span::styled(" ".repeat(pad_x), styles.panel),
                Span::styled(title.to_string(), styles.bold_fg),
                Span::styled(" ".repeat(header_gap), styles.panel),
                Span::styled("esc", styles.muted),
                Span::styled(" ".repeat(pad_x), styles.panel),
            ]);
            frame.render_widget(Paragraph::new(header).style(styles.panel), header_area);

            let search_text = if filter.is_empty() {
                Span::styled("Search", styles.muted)
            } else {
                Span::styled(truncate(&filter, body_width.saturating_sub(2)), styles.fg)
            };
            let search = Line::from(vec![
                Span::styled(" ".repeat(pad_x), styles.panel),
                Span::styled("▌", styles.accent),
                Span::styled(" ", styles.panel),
                search_text,
                Span::styled(" ".repeat(body_width), styles.panel),
            ]);
            frame.render_widget(Paragraph::new(search).style(styles.panel), search_area);

            let mut lines: Vec<Line<'static>> = rows
                .iter()
                .skip(scroll)
                .take(usize::from(list_area.height))
                .map(|row| {
                    let active =
                        matches!(row, Row::Item { item_index, .. } if *item_index == selected);
                    let mut line = row_line(row, styles, active, body_width, &current_name);
                    line.spans
                        .insert(0, Span::styled(" ".repeat(pad_x), styles.row_bg(active)));
                    line.spans
                        .push(Span::styled(" ".repeat(pad_x), styles.row_bg(active)));
                    line
                })
                .collect();
            while lines.len() < usize::from(list_area.height) {
                lines.push(Line::from(Span::styled(" ", styles.panel)));
            }
            frame.render_widget(Paragraph::new(lines).style(styles.panel), list_area);

            let footer = Line::from(vec![
                Span::styled(" ".repeat(pad_x), styles.panel),
                Span::styled(truncate(&footer_text, body_width), styles.muted),
                Span::styled(" ".repeat(pad_x), styles.panel),
            ]);
            frame.render_widget(Paragraph::new(footer).style(styles.panel), footer_area);

            frame.set_cursor_position((cursor_col as u16, search_row as u16));
        })?;
        write!(
            terminal.backend_mut(),
            "\x1b[5 q\x1b]12;{}\x07",
            theme.accent
        )?;
        terminal.backend_mut().flush()?;
        Ok(())
    }

    fn navigate_to(&mut self, name: &str) -> anyhow::Result<()> {
        let next =
            config::load_palette(name).with_context(|| format!("Unknown palette: {name}"))?;
        self.stack.push(NavState {
            def: self.current_def.clone(),
            name: self.current_name.clone(),
            selected: self.selected,
            scroll: self.scroll,
            filter: self.filter.clone(),
        });
        self.load_def(name.to_string(), next);
        Ok(())
    }

    fn navigate_back(&mut self) -> bool {
        let Some(prev) = self.stack.pop() else {
            return false;
        };
        self.current_def = prev.def;
        self.current_name = prev.name;
        self.items = apply_user_overrides(&self.current_def.items);
        self.selected = prev.selected;
        self.scroll = prev.scroll;
        self.filter = prev.filter;
        self.filter_cursor = self.filter.len();
        true
    }

    fn build_popup_flags(&self, border_override: Option<&str>) -> anyhow::Result<String> {
        let sizing = config::user_sizing();
        let theme = theme::resolve_active_theme(None)?;
        let popup_border = border_override
            .map(str::to_string)
            .or(sizing.popup_border)
            .unwrap_or_else(|| "none".into());
        let body_style = sizing
            .popup_body_style
            .unwrap_or_else(|| format!("bg={}", theme.panel));
        if popup_border == "none" {
            return Ok(format!("-B -s '{body_style}'"));
        }
        let border_style = sizing
            .popup_border_style
            .unwrap_or_else(|| format!("fg={},bg=default", theme.accent));
        Ok(format!(
            "-b {popup_border} -s '{body_style}' -S '{border_style}'"
        ))
    }

    fn build_popup_relaunch_command(
        &self,
        action: &crate::model::PopupAction,
    ) -> anyhow::Result<String> {
        let sizing = config::user_sizing();
        let pad_x = action.pad_x.or(sizing.popup_pad_x).unwrap_or(0);
        let pad_y = action.pad_y.or(sizing.popup_pad_y).unwrap_or(0);
        let width = action
            .width
            .clone()
            .or(sizing.popup_width)
            .unwrap_or_else(|| "80%".into());
        let height = action
            .height
            .clone()
            .or(sizing.popup_height)
            .unwrap_or_else(|| "80%".into());
        let w_expr = shell_size_expr(&width, "client_width", pad_x);
        let h_expr = shell_size_expr(&height, "client_height", pad_y);
        let bin = env::var("TMUX_PALETTE_BIN").unwrap_or_else(|_| "tmux-palette".into());
        Ok(format!(
            "tmux display-popup -E {} -h {} -w {} {}; tmux run-shell -b '{} {}'",
            self.build_popup_flags(action.border.as_deref())?,
            h_expr,
            w_expr,
            action.popup,
            bin,
            self.current_name
        ))
    }

    fn activate(&mut self, item: &Item) -> anyhow::Result<bool> {
        match &item.action {
            Action::Palette { palette } => {
                self.navigate_to(palette)?;
                Ok(false)
            }
            Action::ApplyTheme { apply_theme } => {
                palettes::save_theme(apply_theme)?;
                Ok(!self.navigate_back())
            }
            Action::Popup(popup) => {
                if let Some(cmd_file) = self.cmd_file.as_deref() {
                    std::fs::write(
                        cmd_file,
                        format!("shell:{}", self.build_popup_relaunch_command(popup)?),
                    )?;
                }
                Ok(true)
            }
            action => {
                dispatch_to_file(action, self.cmd_file.as_deref())?;
                Ok(true)
            }
        }
    }

    fn esc(&mut self) -> bool {
        let mode = config::user_sizing().esc.unwrap_or_else(|| "back".into());
        mode != "back" || !self.navigate_back()
    }

    fn reset_filter_selection(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<bool> {
        let vis = self.visible();
        match key.code {
            KeyCode::Esc => return Ok(self.esc()),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected = step(&vis, self.selected, -1)
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selected = step(&vis, self.selected, 1)
            }
            KeyCode::Enter => {
                if let Some(item) = vis
                    .get(self.selected)
                    .filter(|item| is_selectable(Some(item)))
                    .cloned()
                {
                    return self.activate(&item);
                }
            }
            KeyCode::Up => self.selected = step(&vis, self.selected, -1),
            KeyCode::Down => self.selected = step(&vis, self.selected, 1),
            KeyCode::PageUp => {
                for _ in 0..10 {
                    self.selected = step(&vis, self.selected, -1);
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    self.selected = step(&vis, self.selected, 1);
                }
            }
            KeyCode::Left
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL) =>
            {
                self.filter_cursor = word_back(&self.filter, self.filter_cursor)
            }
            KeyCode::Right
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL) =>
            {
                self.filter_cursor = word_forward(&self.filter, self.filter_cursor)
            }
            KeyCode::Left => self.filter_cursor = prev_boundary(&self.filter, self.filter_cursor),
            KeyCode::Right => self.filter_cursor = next_boundary(&self.filter, self.filter_cursor),
            KeyCode::Home => self.filter_cursor = 0,
            KeyCode::End => self.filter_cursor = self.filter.len(),
            KeyCode::Backspace => {
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL)
                {
                    let start = word_back(&self.filter, self.filter_cursor);
                    self.filter.drain(start..self.filter_cursor);
                    self.filter_cursor = start;
                    self.reset_filter_selection();
                } else if self.filter_cursor > 0 {
                    let prev = prev_boundary(&self.filter, self.filter_cursor);
                    self.filter.drain(prev..self.filter_cursor);
                    self.filter_cursor = prev;
                    self.reset_filter_selection();
                }
            }
            KeyCode::Delete => {
                if self.filter_cursor < self.filter.len() {
                    self.filter.remove(self.filter_cursor);
                    self.reset_filter_selection();
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter.drain(..self.filter_cursor);
                self.filter_cursor = 0;
                self.reset_filter_selection();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let start = word_back(&self.filter, self.filter_cursor);
                self.filter.drain(start..self.filter_cursor);
                self.filter_cursor = start;
                self.reset_filter_selection();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter.truncate(self.filter_cursor);
                self.reset_filter_selection();
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.filter.insert(self.filter_cursor, ch);
                self.filter_cursor += ch.len_utf8();
                self.reset_filter_selection();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> anyhow::Result<bool> {
        let y = usize::from(mouse.row) + 1;
        let x = usize::from(mouse.column) + 1;
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                let vis = self.visible();
                self.selected = step(&vis, self.selected, -1);
            }
            MouseEventKind::ScrollDown => {
                let vis = self.visible();
                self.selected = step(&vis, self.selected, 1);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self
                    .esc_action
                    .is_some_and(|(ey, x1, x2)| y == ey && x >= x1 && x <= x2)
                {
                    return Ok(self.esc());
                }
                if let Some(hit) = self.row_actions.iter().find(|hit| hit.y == y) {
                    let vis = self.visible();
                    if let Some(item) = vis
                        .get(hit.item_index)
                        .filter(|item| is_selectable(Some(item)))
                        .cloned()
                    {
                        self.selected = hit.item_index;
                        return self.activate(&item);
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let (_guard, mut terminal) = TerminalGuard::enter()?;
        self.render(&mut terminal)?;
        loop {
            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key(key)? {
                            return Ok(());
                        }
                    }
                    Event::Mouse(mouse) => {
                        if self.handle_mouse(mouse)? {
                            return Ok(());
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
                self.render(&mut terminal)?;
            }
        }
    }
}

pub fn run_palette(def: PaletteDef, initial_name: impl Into<String>) -> anyhow::Result<()> {
    PaletteRunner::new(def, initial_name).run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    #[test]
    fn footer_reports_empty_text_when_no_selectable_items() {
        assert_eq!(footer_text(0, "No panes"), "No panes");
        assert_eq!(
            footer_text(1, "No panes"),
            "enter select   up/down move   1 command"
        );
    }

    #[test]
    fn runner_initializes_with_first_selectable_item() {
        let mut heading = Item::new("Heading", Action::Shell { shell: ":".into() });
        heading.selectable = Some(false);
        let selectable = Item::new("Selectable", Action::Shell { shell: ":".into() });
        let runner = PaletteRunner::new(
            PaletteDef {
                title: Some("Test".into()),
                items: vec![heading, selectable],
                grouped: false,
                empty_text: None,
            },
            "test",
        );
        assert_eq!(runner.selected, 1);
    }

    #[test]
    fn word_motion_respects_utf8_boundaries() {
        let s = "héllo world";
        assert_eq!(prev_boundary(s, 3), 1);
        assert_eq!(next_boundary(s, 1), 3);
        assert_eq!(word_back(s, s.len()), 7);
        assert_eq!(word_forward(s, 0), 6);
    }
}
