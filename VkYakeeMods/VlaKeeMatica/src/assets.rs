use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct AssetManager {
    assets_root: PathBuf,
    languages: HashMap<String, HashMap<String, String>>,
    active_language: String,
}

impl AssetManager {
    pub fn find_assets_dir() -> PathBuf {
        let candidates = [
            PathBuf::from("VkYakeeMods/HorizonMatica/assets/litematica"),
            PathBuf::from("HorizonMatica/assets/litematica"),
            PathBuf::from("assets/litematica"),
            PathBuf::from("../assets/litematica"),
            PathBuf::from("/data/data/com.termux/files/home/Mods/starling/VkYakeeMods/HorizonMatica/assets/litematica"),
        ];

        for c in candidates {
            if c.exists() && c.is_dir() {
                return c;
            }
        }

        PathBuf::from("assets/litematica")
    }

    pub fn load_default() -> Self {
        let root = Self::find_assets_dir();
        Self::load_from_path(root)
    }

    pub fn load_from_path(root: impl Into<PathBuf>) -> Self {
        let assets_root = root.into();
        let mut languages = HashMap::new();

        let lang_dir = assets_root.join("lang");
        if let Ok(entries) = fs::read_dir(&lang_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                                languages.insert(stem.to_lowercase(), map);
                            }
                        }
                    }
                }
            }
        }

        Self {
            assets_root,
            languages,
            active_language: "en_us".to_string(),
        }
    }

    pub fn set_language(&mut self, lang_code: &str) -> bool {
        let code = lang_code.to_lowercase();
        if self.languages.contains_key(&code) {
            self.active_language = code;
            true
        } else {
            false
        }
    }

    pub fn get_available_languages(&self) -> Vec<String> {
        let mut list: Vec<_> = self.languages.keys().cloned().collect();
        list.sort();
        list
    }

    pub fn translate(&self, key: &str) -> String {
        if let Some(map) = self.languages.get(&self.active_language) {
            if let Some(val) = map.get(key) {
                return val.clone();
            }
        }

        if let Some(en_map) = self.languages.get("en_us") {
            if let Some(val) = en_map.get(key) {
                return val.clone();
            }
        }

        key.to_string()
    }

    pub fn get_blockstate_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let dir = self.assets_root.join("blockstates");
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                files.push(entry.path());
            }
        }
        files
    }

    pub fn get_model_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let dir = self.assets_root.join("models");
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                files.push(entry.path());
            }
        }
        files
    }

    pub fn get_texture_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let dir = self.assets_root.join("textures");
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                files.push(entry.path());
            }
        }
        files
    }

    pub fn get_icon_path(&self) -> PathBuf {
        self.assets_root.join("icon.png")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assets_loader() {
        let assets = AssetManager::load_default();
        let langs = assets.get_available_languages();
        assert!(!langs.is_empty(), "Languages should be loaded from assets/litematica/lang");
        assert!(langs.contains(&"en_us".to_string()));
        assert!(langs.contains(&"es_es".to_string()));

        let translated_en = assets.translate("litematica.gui.button.area_editor.create_schematic");
        assert_eq!(translated_en, "Save Schematic");
    }
}
