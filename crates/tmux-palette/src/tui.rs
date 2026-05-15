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

use crate::{
    actions::dispatch_to_file,
    config,
    fuzzy::default_filter,
    model::{Action, Item, PaletteDef, Theme},
    palettes,
    render::{
        build_rows, compose_footer, compose_header, compose_list_body, compose_search,
        first_selectable, is_selectable, render_category, render_default_item, step, Row,
        RowAction,
    },
    theme,
};

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
    fn enter() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(
            std::io::stdout(),
            EnableMouseCapture,
            cursor::Hide,
            cursor::MoveTo(0, 0)
        )?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            std::io::stdout(),
            DisableMouseCapture,
            cursor::Show,
            cursor::SetCursorStyle::DefaultUserShape,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        );
        let _ = write!(std::io::stdout(), "\x1b]112\x07");
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

    fn render(&mut self) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let width = usize::from(width).max(1);
        let height = usize::from(height).max(1);
        let vis = self.visible();
        self.ensure_selectable(&vis);
        let (theme, colors) = self.colors(&vis)?;
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
        let blank = format!("{}{}{}", colors.panel, " ".repeat(width), colors.reset);
        let title = self.current_def.title.as_deref().unwrap_or("Commands");
        let header = compose_header(title, width, pad_x, body_width, &colors);
        self.esc_action = Some((if bordered { 1 } else { 2 }, header.esc_x1, header.esc_x2));
        let (body_lines, row_actions) = compose_list_body(
            &rows,
            self.scroll,
            list_height,
            self.selected,
            body_width,
            pad_x,
            &colors,
            if bordered { 4 } else { 5 },
            |row, is_selected| match row {
                Row::Category { category } => {
                    let row_bg = if is_selected {
                        &colors.selected
                    } else {
                        &colors.panel
                    };
                    render_category(category, &colors, row_bg)
                }
                Row::Item { item, .. } => {
                    if self.current_name == "find-pane" {
                        palettes::render_find_pane_item(item, &colors, is_selected, body_width)
                    } else {
                        render_default_item(item, &colors, is_selected, body_width)
                    }
                }
            },
        );
        self.row_actions = row_actions;
        let selectable_count = vis.iter().filter(|item| is_selectable(Some(item))).count();
        let empty = self
            .current_def
            .empty_text
            .as_deref()
            .unwrap_or("No results");
        let footer = compose_footer(
            &footer_text(selectable_count, empty),
            pad_x,
            body_width,
            &colors,
        );
        let mut lines = vec![
            header.line,
            compose_search(&self.filter, pad_x, body_width, &colors),
            blank.clone(),
        ];
        lines.extend(body_lines);
        lines.push(blank.clone());
        lines.push(footer);
        if !bordered {
            lines.insert(0, blank.clone());
            lines.push(blank);
        }
        let search_row = if bordered { 2 } else { 3 };
        let cursor_col =
            (pad_x + 3 + self.filter_cursor).min(pad_x + 3 + body_width.saturating_sub(2));
        let mut out = std::io::stdout();
        write!(
            out,
            "\x1b[?2026h\x1b[?25l\x1b[H{}\x1b[{};{}H\x1b[5 q\x1b]12;{}\x07\x1b[?25h\x1b[?2026l",
            lines.join("\n"),
            search_row,
            cursor_col,
            theme.accent
        )?;
        out.flush()?;
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
        let _guard = TerminalGuard::enter()?;
        self.render()?;
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
                self.render()?;
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
