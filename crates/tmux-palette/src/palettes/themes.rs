use std::fs;

use crate::{
    model::{Action, Item, PaletteDef},
    theme::{self, ThemeSource},
};

const CUSTOM_THEME_DOCS: &str = "https://github.com/eduwass/tmux-palette#custom-themes";

pub fn save_theme(slug: &str) -> std::io::Result<()> {
    let path = crate::config::config_dir().join("theme.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::json!({ "name": slug })))
}

pub fn themes() -> PaletteDef {
    let mut items: Vec<Item> = theme::list_themes()
        .into_iter()
        .map(|entry| {
            let mut item = Item::new(
                entry.name,
                Action::ApplyTheme {
                    apply_theme: entry.slug.clone(),
                },
            );
            item.icon = Some("●".into());
            item.icon_color = Some(entry.theme.accent.clone());
            item.aliases = vec![entry.slug];
            if entry.source == ThemeSource::User {
                item.description = Some("custom".into());
            }
            item.data = Some(serde_json::to_value(entry.theme).expect("theme serializes"));
            item
        })
        .collect();

    let mut docs = Item::new(
        "Add custom theme...",
        Action::Shell {
            shell: format!("open '{CUSTOM_THEME_DOCS}' || xdg-open '{CUSTOM_THEME_DOCS}'"),
        },
    );
    docs.icon = Some("+".into());
    docs.description = Some("Open setup instructions".into());
    docs.aliases = vec!["custom".into(), "theme".into(), "docs".into()];
    items.push(docs);

    PaletteDef {
        title: Some("Themes".into()),
        items,
        grouped: false,
        empty_text: Some("No themes found".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn with_config<F: FnOnce(&TempDir)>(f: F) {
        let _guard = crate::config::ENV_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let old = env::var_os("XDG_CONFIG_HOME");
        env::set_var("XDG_CONFIG_HOME", dir.path());
        f(&dir);
        match old {
            Some(v) => env::set_var("XDG_CONFIG_HOME", v),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn themes_palette_contains_bundled_and_docs_items() {
        with_config(|_| {
            let palette = themes();
            assert_eq!(palette.title.as_deref(), Some("Themes"));
            assert!(!palette.grouped);
            assert!(palette.items.iter().any(|item| item.title == "Dracula"));
            assert!(palette
                .items
                .iter()
                .any(|item| item.title == "Add custom theme..."));
            let dracula = palette
                .items
                .iter()
                .find(|item| item.title == "Dracula")
                .unwrap();
            assert_eq!(dracula.icon.as_deref(), Some("●"));
            assert!(matches!(dracula.action, Action::ApplyTheme { .. }));
            assert!(dracula.data.is_some());
        });
    }

    #[test]
    fn theme_items_mark_user_themes_as_custom() {
        with_config(|_| {
            let dir = crate::config::config_dir().join("themes");
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("mine.json"),
                r##"{"bg":"#111111","panel":"#222222","selected":"#333333","fg":"#eeeeee","muted":"#999999","accent":"#ff00ff"}"##,
            )
            .unwrap();
            let palette = themes();
            let mine = palette
                .items
                .iter()
                .find(|item| item.title == "mine")
                .unwrap();
            assert_eq!(mine.description.as_deref(), Some("custom"));
            assert_eq!(mine.icon_color.as_deref(), Some("#ff00ff"));
        });
    }

    #[test]
    fn save_theme_writes_active_theme_file() {
        with_config(|_| {
            save_theme("dracula").unwrap();
            let raw = fs::read_to_string(crate::config::config_dir().join("theme.json")).unwrap();
            assert_eq!(raw, "{\"name\":\"dracula\"}\n");
        });
    }
}
