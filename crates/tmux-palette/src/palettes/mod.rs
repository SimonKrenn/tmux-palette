mod commands;
mod find_pane;
mod move_pane;
mod themes;

pub use commands::commands;
pub use find_pane::{detect_agent, filter_tree, find_pane, pane_select_action, parse_pane_line};
pub use move_pane::{move_pane, parse_window_line};
pub use themes::{save_theme, themes};

use crate::model::PaletteDef;

pub fn load_builtin(name: &str) -> Option<PaletteDef> {
    match name {
        "commands" => Some(commands()),
        "find-pane" => Some(find_pane()),
        "move-pane" => Some(move_pane()),
        "themes" => Some(themes()),
        _ => None,
    }
}
