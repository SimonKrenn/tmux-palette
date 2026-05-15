use crate::model::{Action, Item, PaletteDef};
use crate::tmux::{tmux, tmux_quote};

pub fn parse_window_line(line: &str) -> Option<(String, String, String)> {
    let mut parts = line.split('\t');
    let session = parts.next()?.to_string();
    let window_index = parts.next()?.to_string();
    if session.is_empty() || window_index.is_empty() {
        return None;
    }
    let window_name = parts
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("window{window_index}"));
    Some((session, window_index, window_name))
}

fn new_window_items(sessions: &[String], pane_id: &str) -> Vec<Item> {
    sessions
        .iter()
        .map(|session| {
            let mut item = Item::new(
                "New window",
                Action::Tmux {
                    tmux: format!(
                        "break-pane -d -s {} -t {}",
                        tmux_quote(pane_id),
                        tmux_quote(&format!("{session}:"))
                    ),
                },
            );
            item.icon = Some("󰝰".into());
            item.description = Some(format!("in {session}"));
            item
        })
        .collect()
}

fn join_window_items(win_lines: &[String], pane_id: &str, current_window: &str) -> Vec<Item> {
    let mut items = Vec::new();
    for line in win_lines {
        let Some((session, window_index, window_name)) = parse_window_line(line) else {
            continue;
        };
        let target = format!("{session}:{window_index}");
        if target == current_window {
            continue;
        }
        let mut item = Item::new(
            window_name,
            Action::Tmux {
                tmux: format!(
                    "join-pane -d -s {} -t {}",
                    tmux_quote(pane_id),
                    tmux_quote(&target)
                ),
            },
        );
        item.icon = Some("󰖲".into());
        item.description = Some(format!("{session} · {window_index}"));
        items.push(item);
    }
    items
}

pub fn move_pane() -> PaletteDef {
    let pane_id = tmux(&["display-message", "-p", "#{pane_id}"]);
    let current_window = tmux(&["display-message", "-p", "#{session_name}:#{window_index}"]);
    let sessions: Vec<String> = tmux(&["list-sessions", "-F", "#S"])
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let win_lines: Vec<String> = tmux(&[
        "list-windows",
        "-a",
        "-F",
        "#{session_name}\t#{window_index}\t#{window_name}",
    ])
    .lines()
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect();
    let mut items = new_window_items(&sessions, &pane_id);
    items.extend(join_window_items(&win_lines, &pane_id, &current_window));
    PaletteDef {
        title: Some("Move Pane to...".into()),
        items,
        grouped: false,
        empty_text: Some("No targets".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_window_line() {
        assert_eq!(
            parse_window_line("s\t1\tname"),
            Some(("s".into(), "1".into(), "name".into()))
        );
    }
    #[test]
    fn defaults_window_name() {
        assert_eq!(
            parse_window_line("s\t2\t"),
            Some(("s".into(), "2".into(), "window2".into()))
        );
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert_eq!(parse_window_line("\t2\tname"), None);
        assert_eq!(parse_window_line("s\t\tname"), None);
    }
}
