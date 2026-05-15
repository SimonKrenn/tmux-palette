use crate::fuzzy::multi_fuzzy_score;
use crate::model::{Action, Item, PaletteDef};
use crate::tmux::{tmux, tmux_quote};
use serde_json::json;

pub fn detect_agent(command: &str, title: &str) -> String {
    let direct = [
        "claude",
        "codex",
        "aider",
        "cursor-agent",
        "opencode",
        "gemini",
        "ollama",
    ];
    if direct.contains(&command) {
        return command.to_string();
    }
    if title.starts_with("OC | ") || title.starts_with("OC|") {
        return "opencode".into();
    }
    let trimmed_title = title.trim_start();
    if trimmed_title
        .chars()
        .next()
        .map(|c| "*✳⠂⠐⠁⠉⠙⠹⠸⠼⠴⠦⠧⠇⠏".contains(c))
        .unwrap_or(false)
        && trimmed_title.chars().nth(1) == Some(' ')
    {
        return "claude".into();
    }
    String::new()
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pane {
    pub session: String,
    pub window_index: String,
    pub pane_index: String,
    pub window_name: String,
    pub pane_title: String,
    pub command: String,
    pub path: String,
    pub agent: String,
    pub pane_active: bool,
    pub window_active: bool,
    pub is_current: bool,
    pub target: String,
}

pub fn parse_pane_line(line: &str, current_pane: &str) -> Option<Pane> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 9 {
        return None;
    }
    if parts[0].is_empty() || parts[1].is_empty() || parts[2].is_empty() {
        return None;
    }
    let target = format!("{}:{}.{}", parts[0], parts[1], parts[2]);
    let pane_title = if parts[4].is_empty() {
        format!("pane{}", parts[2])
    } else {
        parts[4].to_string()
    };
    let command = parts[5].to_string();
    Some(Pane {
        session: parts[0].to_string(),
        window_index: parts[1].to_string(),
        pane_index: parts[2].to_string(),
        window_name: if parts[3].is_empty() {
            format!("window{}", parts[1])
        } else {
            parts[3].to_string()
        },
        pane_title: pane_title.clone(),
        command: command.clone(),
        path: parts[6].to_string(),
        agent: detect_agent(&command, &pane_title),
        pane_active: parts[7] == "1",
        window_active: parts[8] == "1",
        is_current: target == current_pane,
        target,
    })
}

pub fn pane_select_action(p: &Pane) -> Action {
    Action::Tmux {
        tmux: format!(
            "select-pane -t {} \\; select-window -t {} \\; switch-client -t {}",
            tmux_quote(&p.target),
            tmux_quote(&format!("{}:{}", p.session, p.window_index)),
            tmux_quote(&p.session)
        ),
    }
}

pub fn filter_tree(items: &[Item], query: &str) -> Vec<Item> {
    let parts: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    if parts.is_empty() {
        return items.to_vec();
    }
    let mut ok_sessions = std::collections::HashSet::new();
    let mut ok_windows = std::collections::HashSet::new();
    let mut ok_panes = std::collections::HashSet::new();
    for item in items {
        if let Some(data) = &item.data {
            if data.get("kind").and_then(|v| v.as_str()) == Some("pane") {
                let p = &data["pane"];
                let haystack = [
                    p.get("session"),
                    p.get("windowName"),
                    p.get("paneTitle"),
                    p.get("command"),
                    p.get("path"),
                    p.get("target"),
                    p.get("agent"),
                ]
                .into_iter()
                .filter_map(|v| v.and_then(|v| v.as_str()))
                .collect::<Vec<_>>()
                .join(" ");
                if multi_fuzzy_score(
                    &haystack,
                    &parts.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                ) > 0
                {
                    let session = p.get("session").and_then(|v| v.as_str()).unwrap_or("");
                    let window_index = p.get("windowIndex").and_then(|v| v.as_str()).unwrap_or("");
                    let target = p.get("target").and_then(|v| v.as_str()).unwrap_or("");
                    ok_panes.insert(target.to_string());
                    ok_sessions.insert(session.to_string());
                    ok_windows.insert(format!("{session}:{window_index}"));
                }
            }
        }
    }
    items
        .iter()
        .filter(|item| {
            if let Some(data) = &item.data {
                return match data.get("kind").and_then(|v| v.as_str()) {
                    Some("session") => ok_sessions
                        .contains(data.get("session").and_then(|v| v.as_str()).unwrap_or("")),
                    Some("window") => ok_windows.contains(&format!(
                        "{}:{}",
                        data.get("session").and_then(|v| v.as_str()).unwrap_or(""),
                        data.get("windowIndex")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                    )),
                    Some("pane") => ok_panes.contains(
                        data.get("pane")
                            .and_then(|v| v.get("target"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    _ => false,
                };
            }
            false
        })
        .cloned()
        .collect()
}

fn build_items() -> Vec<Item> {
    let current_pane = tmux(&[
        "display-message",
        "-p",
        "#{session_name}:#{window_index}.#{pane_index}",
    ]);
    let current_session = current_pane.split(':').next().unwrap_or("").to_string();
    let lines: Vec<String> = tmux(&[
        "list-panes",
        "-a",
        "-F",
        "#{session_name}\t#{window_index}\t#{pane_index}\t#{window_name}\t#{pane_title}\t#{pane_current_command}\t#{pane_current_path}\t#{pane_active}\t#{window_active}",
    ])
    .lines()
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect();
    let mut panes = Vec::new();
    for line in lines {
        if let Some(p) = parse_pane_line(&line, &current_pane) {
            panes.push(p);
        }
    }
    let mut items = Vec::new();
    let mut sessions = Vec::new();
    for p in &panes {
        if !sessions.contains(&p.session) {
            sessions.push(p.session.clone());
        }
    }
    for session in sessions {
        let in_session: Vec<&Pane> = panes.iter().filter(|p| p.session == session).collect();
        let focused = in_session
            .iter()
            .find(|p| p.pane_active && p.window_active)
            .copied()
            .or_else(|| in_session.first().copied());
        let mut session_item = Item::new(
            &session,
            Action::Tmux {
                tmux: format!("switch-client -t {}", tmux_quote(&session)),
            },
        );
        session_item.selectable = Some(false);
        session_item.data = Some(json!({
            "kind": "session",
            "session": session,
            "count": in_session.len(),
            "path": focused.map(|p| p.path.clone()).unwrap_or_default(),
            "isCurrent": session == current_session,
        }));
        items.push(session_item);
        for p in in_session {
            let mut pane_item = Item::new(&p.pane_title, pane_select_action(p));
            pane_item.data = Some(json!({"kind":"pane","pane": p}));
            items.push(pane_item);
        }
    }
    items
}

pub fn find_pane() -> PaletteDef {
    PaletteDef {
        title: Some("Find Pane".into()),
        items: build_items(),
        grouped: false,
        empty_text: Some("No panes".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn detects_agents() {
        assert_eq!(detect_agent("claude", "x"), "claude");
        assert_eq!(detect_agent("sh", "OC | foo"), "opencode");
        assert_eq!(detect_agent("sh", "  ✳ working"), "claude");
    }
    #[test]
    fn parses_pane_line() {
        let p = parse_pane_line("s\t1\t2\tw\tt\tcmd\t/path\t1\t0", "s:1.2").unwrap();
        assert!(p.is_current);
        assert_eq!(p.agent, "");
    }

    #[test]
    fn rejects_incomplete_pane_lines() {
        assert!(parse_pane_line("\t1\t2\tw\tt\tcmd\t/path\t1\t0", "s:1.2").is_none());
    }
    #[test]
    fn selects_with_quoted_targets() {
        let p = Pane {
            session: "s".into(),
            window_index: "1".into(),
            pane_index: "2".into(),
            window_name: "w".into(),
            pane_title: "t".into(),
            command: "cmd".into(),
            path: "p".into(),
            agent: "".into(),
            pane_active: false,
            window_active: false,
            is_current: false,
            target: "s:1.2".into(),
        };
        if let Action::Tmux { tmux } = pane_select_action(&p) {
            assert!(tmux.contains("'s:1.2'"));
        } else {
            panic!();
        }
    }
    #[test]
    fn filters_tree_by_pane_matches() {
        let pane = json!({"kind":"pane","pane":{"session":"s","windowIndex":"1","paneIndex":"2","windowName":"w","paneTitle":"agent","command":"claude","path":"/x","agent":"claude","target":"s:1.2"}});
        let session = json!({"kind":"session","session":"s"});
        let item = Item {
            icon: None,
            icon_color: None,
            title: "x".into(),
            description: None,
            shortcut: None,
            category: None,
            aliases: vec![],
            action: Action::Palette {
                palette: "p".into(),
            },
            data: Some(pane),
            selectable: None,
        };
        let s = Item {
            data: Some(session),
            ..item.clone()
        };
        let filtered = filter_tree(&[s, item], "claude");
        assert_eq!(filtered.len(), 2);
    }
}
