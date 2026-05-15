use crate::model::{Action, Item, PaletteDef};

pub fn commands() -> PaletteDef {
    let mut items = Vec::new();

    macro_rules! tmux_item {
        ($icon:expr, $category:expr, $title:expr, $tmux:expr) => {{
            let mut item = Item::new($title, Action::Tmux { tmux: $tmux.into() });
            item.icon = Some($icon.into());
            item.category = Some($category.into());
            item
        }};
        ($icon:expr, $category:expr, $title:expr, $description:expr, $tmux:expr) => {{
            let mut item = tmux_item!($icon, $category, $title, $tmux);
            item.description = Some($description.into());
            item
        }};
    }

    let mut find = Item::new(
        "Find Pane",
        Action::Palette {
            palette: "find-pane".into(),
        },
    );
    find.icon = Some("󰍉".into());
    find.category = Some("Panes".into());
    items.push(find);
    items.push(tmux_item!(
        "",
        "Panes",
        "Split Horizontal",
        "side by side",
        "split-window -h -c '#{pane_current_path}'"
    ));
    items.push(tmux_item!(
        "",
        "Panes",
        "Split Vertical",
        "stacked",
        "split-window -v -c '#{pane_current_path}'"
    ));
    items.push(tmux_item!("󰅖", "Panes", "Close Pane", "kill-pane"));
    items.push(tmux_item!(
        "󰒉",
        "Panes",
        "Close Other Panes",
        "confirm-before -p 'kill all other panes? (y/n)' 'kill-pane -a'"
    ));
    items.push(tmux_item!("󰁔", "Panes", "Next Pane", "select-pane -t +1"));
    items.push(tmux_item!(
        "󰁍",
        "Panes",
        "Previous Pane",
        "select-pane -t -1"
    ));
    items.push(tmux_item!(
        "󰎠",
        "Panes",
        "Display Pane Numbers",
        "display-panes"
    ));
    items.push(tmux_item!("󰓡", "Panes", "Cycle Pane Layout", "next-layout"));
    items.push(tmux_item!("󰁝", "Panes", "Swap Pane Up", "swap-pane -U"));
    items.push(tmux_item!("󰁅", "Panes", "Swap Pane Down", "swap-pane -D"));
    items.push(tmux_item!("󰍉", "Panes", "Zoom / Unzoom", "resize-pane -Z"));
    items.push(tmux_item!(
        "󰆏",
        "Panes",
        "Enter Copy Mode",
        "scrollback / select",
        "copy-mode"
    ));
    items.push(tmux_item!(
        "󰏫",
        "Panes",
        "Rename Pane",
        "command-prompt -I '#{pane_title}' 'select-pane -T \"%1\"'"
    ));
    let mut move_pane = Item::new(
        "Move Pane to...",
        Action::Palette {
            palette: "move-pane".into(),
        },
    );
    move_pane.icon = Some("󰁁".into());
    move_pane.category = Some("Panes".into());
    items.push(move_pane);
    items.push(tmux_item!(
        "󰘖",
        "Panes",
        "Break to New Window",
        "break-pane"
    ));

    items.push(tmux_item!(
        "󰝰",
        "Windows",
        "New Window",
        "new-window -c '#{pane_current_path}'"
    ));
    items.push(tmux_item!("󰁔", "Windows", "Next Window", "next-window"));
    items.push(tmux_item!(
        "󰁍",
        "Windows",
        "Previous Window",
        "previous-window"
    ));
    items.push(tmux_item!("󰋚", "Windows", "Last Window", "last-window"));
    items.push(tmux_item!(
        "󰏫",
        "Windows",
        "Rename Window",
        "command-prompt -I '#W' 'rename-window -- \"%%\"'"
    ));
    items.push(tmux_item!(
        "󰅖",
        "Windows",
        "Close Window",
        "confirm-before -p 'kill window? (y/n)' kill-window"
    ));

    items.push(tmux_item!(
        "󱂬",
        "Sessions",
        "Choose Session",
        "choose-tree -Zs"
    ));
    items.push(tmux_item!("󰐕", "Sessions", "New Session", "command-prompt -p 'New session name:' 'new-session -d -s \"%1\" ; switch-client -t \"%1\"'"));
    items.push(tmux_item!(
        "󰏫",
        "Sessions",
        "Rename Session",
        "command-prompt -I '#S' 'rename-session -- \"%%\"'"
    ));
    items.push(tmux_item!(
        "󰁔",
        "Sessions",
        "Next Session",
        "switch-client -n"
    ));
    items.push(tmux_item!(
        "󰁍",
        "Sessions",
        "Previous Session",
        "switch-client -p"
    ));
    items.push(tmux_item!("󰍃", "Sessions", "Detach", "detach-client"));
    items.push(tmux_item!(
        "󰆴",
        "Sessions",
        "Kill Session",
        "confirm-before -p 'kill session #S? (y/n)' kill-session"
    ));
    items.push(tmux_item!(
        "󰑓",
        "System",
        "Reload Config",
        "source-file ~/.tmux.conf ; display-message 'Config reloaded'"
    ));
    let mut themes = Item::new(
        "Switch Theme...",
        Action::Palette {
            palette: "themes".into(),
        },
    );
    themes.icon = Some("".into());
    themes.category = Some("Appearance".into());
    themes.description = Some("browse + live-preview bundled themes".into());
    items.push(themes);

    PaletteDef {
        title: Some("Commands".into()),
        items,
        grouped: true,
        empty_text: None,
    }
}

pub fn load_builtin(name: &str) -> Option<PaletteDef> {
    match name {
        "commands" => Some(commands()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmux_of(title: &str) -> String {
        let palette = commands();
        let item = palette
            .items
            .iter()
            .find(|item| item.title == title)
            .unwrap();
        match &item.action {
            Action::Tmux { tmux } => tmux.clone(),
            _ => panic!("item {title} has no tmux action"),
        }
    }

    #[test]
    fn new_session_creates_and_switches_to_new_session() {
        let cmd = tmux_of("New Session");
        assert!(cmd.contains("new-session"));
        assert!(cmd.contains("switch-client"));
    }

    #[test]
    fn no_command_prompt_template_reuses_double_percent() {
        let offenders: Vec<String> = commands()
            .items
            .into_iter()
            .filter_map(|item| match item.action {
                Action::Tmux { tmux } if tmux.contains("command-prompt") => {
                    let count = tmux.matches("%%").count();
                    (count > 1).then(|| format!("{}: {}", item.title, tmux))
                }
                _ => None,
            })
            .collect();
        assert!(offenders.is_empty(), "{offenders:?}");
    }
}
