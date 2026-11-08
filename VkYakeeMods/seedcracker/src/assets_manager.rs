use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::Value;

pub static ASSET_MANAGER: RwLock<Option<AssetManager>> = RwLock::new(None);

pub struct AssetManager {
    base_dir: PathBuf,
    current_locale: String,
    translations: HashMap<String, String>,
}

impl AssetManager {
    pub fn init<P: AsRef<Path>>(base_dir: P, locale: &str) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();
        let mut mgr = Self {
            base_dir,
            current_locale: locale.to_string(),
            translations: HashMap::new(),
        };
        mgr.load_locale(locale);
        mgr
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.current_locale = locale.to_string();
        self.load_locale(locale);
    }

    fn load_locale(&mut self, locale: &str) {
        self.translations.clear();
        let lang_file = self.base_dir.join("assets").join("lang").join(format!("{}.json", locale));
        if lang_file.exists() {
            if let Ok(content) = fs::read_to_string(&lang_file) {
                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                    if let Some(obj) = val.as_object() {
                        for (k, v) in obj {
                            if let Some(s) = v.as_str() {
                                self.translations.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Fallback to Spanish or English embedded if empty
        if self.translations.is_empty() {
            let embedded = if locale.starts_with("es") {
                include_str!("../assets/lang/es_es.json")
            } else {
                include_str!("../assets/lang/en_us.json")
            };
            if let Ok(val) = serde_json::from_str::<Value>(embedded) {
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj {
                        if let Some(s) = v.as_str() {
                            self.translations.insert(k.clone(), s.to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn translate(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn list_examples(&self) -> Vec<PathBuf> {
        let examples_dir = self.base_dir.join("assets").join("examples");
        let mut results = Vec::new();
        if let Ok(entries) = fs::read_dir(examples_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    results.push(path);
                }
            }
        }
        results
    }
}

pub fn tr(key: &str) -> String {
    if let Ok(guard) = ASSET_MANAGER.read() {
        if let Some(ref mgr) = *guard {
            return mgr.translate(key);
        }
    }
    key.to_string()
}
