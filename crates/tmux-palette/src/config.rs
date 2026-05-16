use crate::cache::CachedConfig;
use crate::model::{Action, CustomPalette, Item, PaletteDef, Sizing};
use anyhow::Context;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn config_dir() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("tmux-palette")
}

pub(crate) fn load_json<T: serde::de::DeserializeOwned>(name: &str, fallback: T) -> T {
    let path = config_dir().join(name);
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return fallback,
    };
    serde_json::from_str(&raw).unwrap_or(fallback)
}

static USER_SHORTCUTS: CachedConfig<std::collections::HashMap<String, String>> =
    CachedConfig::new("shortcuts.json");
static USER_ALIASES: CachedConfig<std::collections::HashMap<String, Vec<String>>> =
    CachedConfig::new("aliases.json");
static USER_COMMANDS: CachedConfig<Vec<Item>> = CachedConfig::new("commands.json");
static USER_HIDDEN: CachedConfig<std::collections::HashSet<String>> =
    CachedConfig::new("hidden.json");
static USER_SIZING: CachedConfig<Sizing> = CachedConfig::new("sizing.json");

pub fn user_shortcuts() -> std::collections::HashMap<String, String> {
    USER_SHORTCUTS.get()
}

pub fn user_aliases() -> std::collections::HashMap<String, Vec<String>> {
    USER_ALIASES.get()
}

pub fn user_commands() -> Vec<Item> {
    USER_COMMANDS.get()
}

pub fn user_hidden() -> std::collections::HashSet<String> {
    USER_HIDDEN.get()
}

pub fn user_sizing() -> Sizing {
    USER_SIZING.get()
}

pub fn user_palette(name: &str) -> Option<CustomPalette> {
    load_json(&format!("palettes/{name}.json"), None)
}

fn read_user_palette(name: &str) -> anyhow::Result<Option<CustomPalette>> {
    let path = config_dir().join("palettes").join(format!("{name}.json"));
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("Failed to read {}", path.display())),
    };
    let palette = serde_json::from_str::<CustomPalette>(&raw)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(palette))
}

fn parse_plain_text(
    output: &str,
    default_action: Action,
    default_icon: Option<String>,
    default_icon_color: Option<String>,
) -> Vec<Item> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            let (icon, icon_color, title) = match parts.as_slice() {
                [title] => (None, None, (*title).to_string()),
                [icon, title] => (Some((*icon).to_string()), None, (*title).to_string()),
                [icon, color, ..] => (
                    Some((*icon).to_string()),
                    Some((*color).to_string()),
                    parts[2..].join("\t"),
                ),
                _ => (None, None, line.to_string()),
            };
            let mut item = Item::new(title.clone(), substitute_action(&default_action, &title));
            item.icon = icon;
            item.icon_color = icon_color;
            if item.icon.is_none() {
                item.icon = default_icon.clone();
            }
            if item.icon_color.is_none() {
                item.icon_color = default_icon_color.clone();
            }
            item
        })
        .collect()
}

fn substitute_action(action: &Action, value: &str) -> Action {
    match action {
        Action::Shell { shell } => Action::Shell {
            shell: shell.replace("{}", value),
        },
        Action::Tmux { tmux } => Action::Tmux {
            tmux: tmux.replace("{}", value),
        },
        Action::Palette { palette } => Action::Palette {
            palette: palette.replace("{}", value),
        },
        Action::Popup(p) => {
            let mut p = p.clone();
            p.popup = p.popup.replace("{}", value);
            Action::Popup(p)
        }
        Action::ApplyTheme { apply_theme } => Action::ApplyTheme {
            apply_theme: apply_theme.replace("{}", value),
        },
    }
}

fn error_item(title: impl Into<String>, description: impl Into<String>) -> Item {
    let mut item = Item::new(title, Action::Shell { shell: ":".into() });
    item.icon = Some(String::new());
    item.description = Some(description.into());
    item
}

pub fn plugin_items(
    command: &str,
    default_action: Option<Action>,
    default_icon: Option<String>,
    default_icon_color: Option<String>,
) -> Vec<Item> {
    let output = match Command::new("sh").arg("-c").arg(command).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).into_owned()
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr)
                .trim()
                .lines()
                .next()
                .map(str::to_string)
                .unwrap_or_else(|| format!("exit {}", output.status.code().unwrap_or_default()));
            return vec![error_item("Plugin command failed", err)];
        }
        Err(err) => return vec![error_item("Plugin command failed", err.to_string())],
    };

    if let Ok(items) = serde_json::from_str::<Vec<Item>>(&output) {
        return items;
    }
    match default_action {
        Some(action) => parse_plain_text(&output, action, default_icon, default_icon_color),
        None => vec![error_item(
            "Plain-text plugin output but no 'action' template set",
            "Add an 'action' field to the palette JSON (use {} for the line text)",
        )],
    }
}

pub fn load_palette(name: &str) -> Option<PaletteDef> {
    load_palette_result(name).ok().flatten()
}

pub fn load_palette_result(name: &str) -> anyhow::Result<Option<PaletteDef>> {
    if name == "commands" {
        let mut items = crate::palettes::commands().items;
        items.extend(user_commands());
        let hidden = user_hidden();
        items.retain(|item| !hidden.contains(&item.title));
        return Ok(Some(PaletteDef {
            title: Some("Commands".into()),
            items,
            grouped: true,
            empty_text: None,
        }));
    }

    if let Some(builtin) = crate::palettes::load_builtin(name) {
        return Ok(Some(builtin));
    }

    let custom = match read_user_palette(name)? {
        Some(custom) => custom,
        None => return Ok(None),
    };
    let mut items = Vec::new();
    let mut all_main = crate::palettes::commands().items;
    all_main.extend(user_commands());
    for title in custom.from {
        if let Some(item) = all_main.iter().find(|item| item.title == title) {
            items.push(item.clone());
        }
    }
    if let Some(category) = custom.from_category.as_deref() {
        items.extend(
            all_main
                .iter()
                .filter(|item| item.category.as_deref() == Some(category))
                .cloned(),
        );
    }
    if let Some(command) = custom.command.as_deref() {
        items.extend(plugin_items(
            command,
            custom.action.clone(),
            custom.icon.clone(),
            custom.icon_color.clone(),
        ));
    }
    items.extend(custom.items);
    Ok(Some(PaletteDef {
        title: Some(custom.title.unwrap_or_else(|| name.to_string())),
        items,
        grouped: custom.grouped.unwrap_or(false),
        empty_text: custom.empty_text,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn with_config<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let old = env::var_os("XDG_CONFIG_HOME");
        env::set_var("XDG_CONFIG_HOME", dir.path());
        f();
        match old {
            Some(v) => env::set_var("XDG_CONFIG_HOME", v),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn loads_commands_and_hidden() {
        with_config(|| {
            let cfg = config_dir();
            fs::create_dir_all(&cfg).unwrap();
            fs::write(
                cfg.join("commands.json"),
                r#"[{"title":"User","action":{"shell":"echo hi"}}]"#,
            )
            .unwrap();
            fs::write(cfg.join("hidden.json"), r#"["Reload Config"]"#).unwrap();
            assert_eq!(user_commands().len(), 1);
            assert!(user_hidden().contains("Reload Config"));
            assert!(!load_palette("commands")
                .unwrap()
                .items
                .iter()
                .any(|i| i.title == "Reload Config"));
        });
    }

    #[test]
    fn loads_custom_palette_sources() {
        with_config(|| {
            let cfg = config_dir().join("palettes");
            fs::create_dir_all(&cfg).unwrap();
            fs::write(cfg.join("demo.json"), r#"{"title":"Demo","from":["Find Pane"],"fromCategory":"Panes","items":[{"title":"Local","action":{"shell":"echo local"}}]}"#).unwrap();
            let palette = load_palette("demo").unwrap();
            assert!(palette.items.iter().any(|i| i.title == "Local"));
            assert!(palette.items.iter().any(|i| i.title == "Find Pane"));
            assert!(palette
                .items
                .iter()
                .any(|i| i.category.as_deref() == Some("Panes")));
        });
    }

    #[test]
    fn loads_command_only_custom_palette_and_uses_name_as_title() {
        with_config(|| {
            let cfg = config_dir().join("palettes");
            fs::create_dir_all(&cfg).unwrap();
            fs::write(
                cfg.join("git-branches.json"),
                r#"{"command":"printf 'main\n'","action":{"tmux":"send-keys 'git checkout {}' Enter"}}"#,
            )
            .unwrap();

            let palette = load_palette_result("git-branches").unwrap().unwrap();
            assert_eq!(palette.title.as_deref(), Some("git-branches"));
            assert_eq!(palette.items[0].title, "main");
        });
    }

    #[test]
    fn custom_palette_parse_errors_are_visible() {
        with_config(|| {
            let cfg = config_dir().join("palettes");
            fs::create_dir_all(&cfg).unwrap();
            fs::write(
                cfg.join("broken.json"),
                r#"{"items":[{"title":"Missing action"}]}"#,
            )
            .unwrap();

            let err = load_palette_result("broken").unwrap_err().to_string();
            assert!(err.contains("Failed to parse"));
            assert!(err.contains("broken.json"));
        });
    }

    #[test]
    fn parses_plain_text_plugin_output() {
        let items = plugin_items(
            "printf 'A\tB\tTitle\nPlain\nA\tB\tTitle\twith\ttabs\n'",
            Some(Action::Shell {
                shell: "open {}".into(),
            }),
            None,
            None,
        );
        assert_eq!(items[0].title, "Title");
        assert_eq!(items[0].icon.as_deref(), Some("A"));
        assert_eq!(items[0].icon_color.as_deref(), Some("B"));
        assert_eq!(items[1].title, "Plain");
        assert_eq!(items[2].title, "Title\twith\ttabs");
    }

    #[test]
    fn plain_text_plugin_without_template_returns_visible_error() {
        let items = plugin_items("printf 'Plain\n'", None, None, None);
        assert_eq!(
            items[0].title,
            "Plain-text plugin output but no 'action' template set"
        );
        assert!(matches!(items[0].action, Action::Shell { ref shell } if shell == ":"));
    }
}
