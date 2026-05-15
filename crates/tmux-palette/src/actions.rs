use std::{env, fs};

use crate::{model::Action, theme::default_theme};

fn popup_flags() -> String {
    let theme = default_theme();
    format!("-B -s 'bg={}'", theme.panel)
}

pub fn encode_action(action: &Action) -> Option<String> {
    match action {
        Action::Tmux { tmux } => Some(format!("tmux:{tmux}")),
        Action::Shell { shell } => Some(format!("shell:{shell}")),
        Action::Palette { palette } => {
            let bin = env::var("TMUX_PALETTE_BIN").unwrap_or_else(|_| "tmux-palette".into());
            Some(format!("tmux:run-shell -b '{bin} {palette}'"))
        }
        Action::Popup(popup) => Some(format!(
            "tmux:display-popup -E {} -h 80% -w 80% {}",
            popup_flags(),
            popup.popup
        )),
    }
}

pub fn dispatch_to_file(
    action: &Action,
    cmd_file: Option<&std::path::Path>,
) -> std::io::Result<()> {
    if let (Some(encoded), Some(cmd_file)) = (encode_action(action), cmd_file) {
        fs::write(cmd_file, encoded)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PopupAction;

    #[test]
    fn encodes_tmux_commands_for_wrapper() {
        assert_eq!(
            encode_action(&Action::Tmux {
                tmux: "split-window -h".into()
            })
            .unwrap(),
            "tmux:split-window -h"
        );
    }

    #[test]
    fn encodes_shell_commands_for_wrapper() {
        assert_eq!(
            encode_action(&Action::Shell {
                shell: "echo hi".into()
            })
            .unwrap(),
            "shell:echo hi"
        );
    }

    #[test]
    fn encodes_nested_palette_actions_using_configured_launcher() {
        env::set_var("TMUX_PALETTE_BIN", "/tmp/tmux-palette");
        assert_eq!(
            encode_action(&Action::Palette {
                palette: "themes".into()
            })
            .unwrap(),
            "tmux:run-shell -b '/tmp/tmux-palette themes'"
        );
        env::remove_var("TMUX_PALETTE_BIN");
    }

    #[test]
    fn encodes_popup_actions_with_default_sizing() {
        let encoded = encode_action(&Action::Popup(PopupAction {
            popup: "htop".into(),
            width: None,
            height: None,
            pad_x: None,
            pad_y: None,
            border: None,
        }))
        .unwrap();
        assert!(encoded.starts_with("tmux:display-popup -E -B -s '"));
        assert!(encoded.ends_with(" -h 80% -w 80% htop"));
    }
}
