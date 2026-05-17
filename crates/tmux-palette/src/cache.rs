use std::{fs, sync::Mutex, time::SystemTime};

pub struct CachedConfig<T> {
    rel_path: &'static str,
    state: Mutex<Option<(Option<SystemTime>, T)>>,
}

impl<T: Clone + Default + serde::de::DeserializeOwned> CachedConfig<T> {
    pub const fn new(rel_path: &'static str) -> Self {
        Self {
            rel_path,
            state: Mutex::new(None),
        }
    }

    pub fn get(&self) -> T {
        self.get_with(|| crate::config::load_config(self.rel_path, T::default()))
    }

    pub fn get_with<F: FnOnce() -> T>(&self, loader: F) -> T {
        let path = crate::config::config_dir().join(self.rel_path);
        let current_mtime = fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .or_else(|| crate::config::config_file_mtime(self.rel_path));

        let mut guard = self.state.lock().unwrap();

        if let Some((cached_mtime, ref value)) = *guard {
            if cached_mtime == current_mtime {
                return value.clone();
            }
        }

        let value = loader();
        *guard = Some((current_mtime, value.clone()));
        value
    }

    #[cfg(test)]
    pub fn clear(&self) {
        self.state.lock().unwrap().take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ENV_LOCK;
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
    fn cache_returns_default_when_file_missing() {
        with_config(|| {
            static CACHE: CachedConfig<Vec<String>> = CachedConfig::new("missing");
            assert_eq!(CACHE.get(), Vec::<String>::default());
        });
    }

    #[test]
    fn cache_updates_when_file_added() {
        with_config(|| {
            let dir = crate::config::config_dir();
            fs::create_dir_all(&dir).unwrap();
            static CACHE: CachedConfig<Vec<String>> = CachedConfig::new("items");

            assert_eq!(CACHE.get(), Vec::<String>::default());

            fs::write(dir.join("items.json"), r#"["a","b"]"#).unwrap();

            // Drop the cache and re-check
            CACHE.state.lock().unwrap().take();
            assert_eq!(CACHE.get(), vec!["a", "b"]);
        });
    }
}
