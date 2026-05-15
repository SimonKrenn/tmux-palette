use crate::{
    model::{Colors, Item},
    text::{char_width, display_width, truncate},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    Category { category: String },
    Item { item: Item, item_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowAction {
    pub y: usize,
    pub item_index: usize,
}

pub fn is_selectable(item: Option<&Item>) -> bool {
    item.is_some_and(|item| item.selectable != Some(false))
}

pub fn step(vis: &[Item], from: usize, dir: i8) -> usize {
    if vis.is_empty() {
        return 0;
    }
    let mut i = from;
    for _ in 0..vis.len() {
        if dir > 0 {
            i = (i + 1) % vis.len();
        } else {
            i = (i + vis.len() - 1) % vis.len();
        }
        if is_selectable(vis.get(i)) {
            return i;
        }
    }
    from
}

pub fn first_selectable(vis: &[Item]) -> Option<usize> {
    vis.iter().position(|item| is_selectable(Some(item)))
}

pub fn build_rows(vis: &[Item], grouped: bool, filtered: bool) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut last_cat = String::new();
    for (i, item) in vis.iter().enumerate() {
        if grouped && !filtered {
            if let Some(category) = &item.category {
                if category != &last_cat {
                    rows.push(Row::Category {
                        category: category.clone(),
                    });
                    last_cat = category.clone();
                }
            }
        }
        rows.push(Row::Item {
            item: item.clone(),
            item_index: i,
        });
    }
    rows
}

pub fn render_category(category: &str, colors: &Colors, row_bg: &str) -> String {
    format!(
        "{}{}{}{}{}",
        colors.accent, colors.bold, category, colors.reset, row_bg
    )
}

fn alias_chip(item: &Item, colors: &Colors, row_bg: &str) -> (String, usize) {
    let Some(alias) = item.aliases.first() else {
        return (String::new(), 0);
    };
    (
        format!(
            "  {}{} {} {}{}",
            colors.bg, colors.fg, alias, colors.reset, row_bg
        ),
        2 + 1 + alias.len() + 1,
    )
}

fn description_fragment(item: &Item, colors: &Colors, row_bg: &str) -> (String, usize) {
    let Some(description) = &item.description else {
        return (String::new(), 0);
    };
    (
        format!(
            "{} - {}{}{}",
            colors.muted, description, colors.reset, row_bg
        ),
        3 + description.len(),
    )
}

fn shortcut_fragment(item: &Item, colors: &Colors, active: bool, row_bg: &str) -> (String, String) {
    let text = item.shortcut.clone().unwrap_or_default();
    if text.is_empty() {
        return (String::new(), text);
    }
    let color = if active {
        &colors.accent
    } else {
        &colors.muted
    };
    (format!("{}{}{}{}", color, text, colors.reset, row_bg), text)
}

pub fn render_default_item(
    item: &Item,
    colors: &Colors,
    active: bool,
    body_width: usize,
) -> String {
    let row_bg = if active {
        &colors.selected
    } else {
        &colors.panel
    };
    let marker = if active {
        format!("{}▌{}{}", colors.accent, colors.reset, row_bg)
    } else {
        " ".to_string()
    };
    let icon_glyph = item.icon.as_deref().unwrap_or(" ");
    let icon = if let Some(icon) = &item.icon {
        format!("{}{}{}{}", colors.accent, icon, colors.reset, row_bg)
    } else {
        " ".to_string()
    };
    let title_style = if active {
        format!("{}{}", colors.bold, colors.fg)
    } else {
        colors.muted.clone()
    };
    let title_styled = format!("{}{}{}{}", title_style, item.title, colors.reset, row_bg);
    let (chip, chip_width) = alias_chip(item, colors, row_bg);
    let (desc, desc_width) = description_fragment(item, colors, row_bg);
    let (shortcut, shortcut_text) = shortcut_fragment(item, colors, active, row_bg);
    let left_styled = format!("{marker} {icon}  {title_styled}{chip}{desc}");
    let left_plain_width = 1
        + 1
        + icon_glyph.chars().next().map(char_width).unwrap_or(1)
        + 2
        + display_width(&item.title)
        + chip_width
        + desc_width;
    let gap = body_width
        .saturating_sub(left_plain_width + shortcut_text.len())
        .max(1);
    format!("{}{}{}", left_styled, " ".repeat(gap), shortcut)
}

pub fn compose_list_body<F>(
    rows: &[Row],
    scroll: usize,
    list_height: usize,
    selected: usize,
    body_width: usize,
    pad_x: usize,
    colors: &Colors,
    start_y: usize,
    render_row: F,
) -> (Vec<String>, Vec<RowAction>)
where
    F: Fn(&Row, bool) -> String,
{
    let mut lines = Vec::new();
    let mut row_actions = Vec::new();
    let end = rows.len().min(scroll + list_height);
    for i in scroll..end {
        let row = &rows[i];
        let is_selected = matches!(row, Row::Item { item_index, .. } if *item_index == selected);
        if let Row::Item { item_index, .. } = row {
            row_actions.push(RowAction {
                y: start_y + (i - scroll),
                item_index: *item_index,
            });
        }
        let row_bg = if is_selected {
            &colors.selected
        } else {
            &colors.panel
        };
        let content = render_row(row, is_selected);
        lines.push(format!(
            "{}{}{}{}{}",
            row_bg,
            " ".repeat(pad_x),
            truncate(&content, body_width),
            " ".repeat(pad_x),
            colors.reset
        ));
    }
    let blank = format!(
        "{}{}{}",
        colors.panel,
        " ".repeat(body_width + pad_x * 2),
        colors.reset
    );
    while lines.len() < list_height {
        lines.push(blank.clone());
    }
    (lines, row_actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;

    fn test_colors() -> Colors {
        Colors {
            bg: String::new(),
            panel: String::new(),
            selected: String::new(),
            fg: String::new(),
            muted: String::new(),
            accent: String::new(),
            reset: String::new(),
            bold: String::new(),
        }
    }

    fn items() -> Vec<Item> {
        let action = Action::Shell { shell: ":".into() };
        let mut find = Item::new("Find Pane", action.clone());
        find.category = Some("Panes".into());
        let mut section = Item::new("Section", action.clone());
        section.category = Some("Panes".into());
        section.selectable = Some(false);
        let mut new_window = Item::new("New Window", action);
        new_window.category = Some("Windows".into());
        vec![find, section, new_window]
    }

    #[test]
    fn treats_items_as_selectable_unless_explicitly_disabled() {
        let items = items();
        assert!(is_selectable(items.first()));
        assert!(!is_selectable(items.get(1)));
    }

    #[test]
    fn finds_and_steps_over_non_selectable_items() {
        let items = items();
        assert_eq!(first_selectable(&items), Some(0));
        assert_eq!(step(&items, 0, 1), 2);
        assert_eq!(step(&items, 2, -1), 0);
    }

    #[test]
    fn adds_category_rows_when_grouped_and_unfiltered() {
        let rows = build_rows(&items(), true, false);
        let labels: Vec<String> = rows
            .into_iter()
            .map(|row| match row {
                Row::Category { category } => category,
                Row::Item { item, .. } => item.title,
            })
            .collect();
        assert_eq!(
            labels,
            vec!["Panes", "Find Pane", "Section", "Windows", "New Window"]
        );
    }

    #[test]
    fn omits_category_rows_while_filtering() {
        let rows = build_rows(&items(), true, true);
        assert!(rows.iter().all(|row| matches!(row, Row::Item { .. })));
    }

    #[test]
    fn tracks_row_actions_only_for_item_rows() {
        let rows = build_rows(&items(), true, false);
        let (lines, row_actions) = compose_list_body(
            &rows,
            0,
            3,
            0,
            20,
            1,
            &test_colors(),
            10,
            |row, _| match row {
                Row::Category { category } => category.clone(),
                Row::Item { item, .. } => item.title.clone(),
            },
        );
        assert_eq!(lines.len(), 3);
        assert_eq!(
            row_actions,
            vec![
                RowAction {
                    y: 11,
                    item_index: 0
                },
                RowAction {
                    y: 12,
                    item_index: 1
                }
            ]
        );
    }
}
