use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Action {
    Tmux {
        tmux: String,
    },
    Shell {
        shell: String,
    },
    Palette {
        palette: String,
    },
    Popup(PopupAction),
    #[serde(rename_all = "camelCase")]
    ApplyTheme {
        apply_theme: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PopupAction {
    pub popup: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(rename = "padX", default, skip_serializing_if = "Option::is_none")]
    pub pad_x: Option<u16>,
    #[serde(rename = "padY", default, skip_serializing_if = "Option::is_none")]
    pub pad_y: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "iconColor", default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    pub action: Action,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selectable: Option<bool>,
}

impl Item {
    pub fn new(title: impl Into<String>, action: Action) -> Self {
        Self {
            icon: None,
            icon_color: None,
            title: title.into(),
            description: None,
            shortcut: None,
            category: None,
            aliases: Vec::new(),
            action,
            data: None,
            selectable: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub bg: String,
    pub panel: String,
    pub selected: String,
    pub fg: String,
    pub muted: String,
    pub accent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colors {
    pub bg: String,
    pub panel: String,
    pub selected: String,
    pub fg: String,
    pub muted: String,
    pub accent: String,
    pub reset: String,
    pub bold: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteDef {
    pub title: Option<String>,
    pub items: Vec<Item>,
    pub grouped: bool,
    pub empty_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Sizing {
    pub width: Option<u16>,
    pub max_height: Option<u16>,
    pub pad_x: Option<u16>,
    pub mobile_width: Option<u16>,
    pub border: Option<String>,
    pub body_style: Option<String>,
    pub border_style: Option<String>,
    pub popup_border: Option<String>,
    pub popup_body_style: Option<String>,
    pub popup_border_style: Option<String>,
    pub popup_width: Option<String>,
    pub popup_height: Option<String>,
    pub popup_pad_x: Option<u16>,
    pub popup_pad_y: Option<u16>,
    pub esc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CustomPalette {
    pub title: Option<String>,
    pub items: Vec<Item>,
    pub from: Vec<String>,
    pub from_category: Option<String>,
    pub command: Option<String>,
    pub action: Option<Action>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub grouped: Option<bool>,
    pub empty_text: Option<String>,
}
